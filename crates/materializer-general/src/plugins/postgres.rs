use std::{
    collections::VecDeque,
    convert::{TryFrom, TryInto},
};

use anyhow::Context;
use bb8_postgres::{
    bb8,
    bb8::{Pool, PooledConnection},
    tokio_postgres::{
        binary_copy::BinaryCopyInWriter,
        types::{ToSql, Type},
        Config,
        NoTls,
    },
    PostgresConnectionManager,
};
use cdl_dto::materialization::{FieldDefinition, FullView, PostgresMaterializerOptions};
use futures::pin_mut;
use itertools::Itertools;
use metrics_utils::{self as metrics, counter};
use rpc::materializer_general::MaterializedView;
use serde_json::Value;
use settings_utils::apps::PostgresSettings;

use super::MaterializerPlugin;
use crate::plugins::models::RowDefinition;

const SCHEMA_COLUMN_PREFIX: &str = "schema_";

// TODO: Move some of those structs to dto crate

pub struct PostgresMaterializer {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

#[derive(Debug)]
struct PsqlView {
    options: PostgresMaterializerOptions,
    rows: Vec<RowDefinition>,
}

#[derive(Debug)]
struct Field {
    sql_name: String,
    name: String,
    json_path: String,
    type_: Type,
}

impl TryFrom<MaterializedView> for PsqlView {
    type Error = anyhow::Error;

    fn try_from(view: MaterializedView) -> Result<Self, Self::Error> {
        let options = serde_json::from_str(&view.options.options)?;

        let rows = view
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<anyhow::Result<_>>()?;

        Ok(PsqlView { options, rows })
    }
}

#[async_trait::async_trait]
impl MaterializerPlugin for PostgresMaterializer {
    fn validate_options(&self, _options: Value) -> anyhow::Result<()> {
        // TODO: Validate if options have proper structure
        // TODO: Validate if table contains only ASCII letters (prevent sql injection)
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(
        &self,
        view: MaterializedView,
        view_definition: FullView,
    ) -> anyhow::Result<()> {
        counter!("cdl.materializer.postgres.upsert-materialized-view", 1);

        let psql_view: PsqlView = view.try_into()?;
        tracing::trace!(?psql_view, "PSQL View");

        let mut conn = self.connect().await?;

        //TODO: Validate if column names are only in ASCII letters and numbers (prevent sql injection)

        if psql_view.rows.is_empty() {
            tracing::warn!("Materialized view is empty, skipping upserting");
        }

        let (copy_stm, insert_stm, create_stm, types, fields) =
            self.build_query(&psql_view, &view_definition).await?;

        let tx = conn.transaction().await?;
        // Temporary table is unique per session
        tx.batch_execute(&create_stm).await?;
        let sink = tx.copy_in(copy_stm.as_str()).await?;
        let writer = BinaryCopyInWriter::new(sink, &types); // Batch insert

        let num_written = self.write(writer, &psql_view, &fields).await?;

        tx.batch_execute(&insert_stm).await?;

        let store_result = tx.commit().await;
        tracing::trace!("PSQL `UPSERT` {:?}", store_result);
        store_result?;

        counter!("cdl.materializer.postgres.store", num_written as u64);

        Ok(())
    }
}

impl PostgresMaterializer {
    async fn write(
        &self,
        writer: BinaryCopyInWriter,
        view: &PsqlView,
        fields: &[Field],
    ) -> anyhow::Result<usize> {
        pin_mut!(writer);

        let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::with_capacity(view.rows.len());
        for m in view.rows.iter() {
            row.clear();
            for object_id in &m.object_ids {
                row.push(object_id);
            }
            for field in fields {
                let f = m.fields.get(&field.name).context("Field not found")?;
                row.push(f.pointer(&field.json_path).context("Subobject not found")?);
            }

            writer.as_mut().write(&row).await?;
        }
        writer.finish().await?;

        Ok(view.rows.len())
    }

    async fn build_query(
        &self,
        view: &PsqlView,
        definition: &FullView,
    ) -> anyhow::Result<(String, String, String, Vec<Type>, Vec<Field>)> {
        let table = &view.options.table;

        let pk_fields = get_pk_field_list(definition);
        let fields = get_field_list(definition);

        let types = pk_fields
            .iter()
            .chain(fields.iter())
            .map(|x| x.type_.clone())
            .collect();

        let columns = fields.iter().map(|x| x.sql_name.to_owned()).join(", ");
        let pk_columns = pk_fields.iter().map(|x| x.sql_name.to_owned()).join(", ");
        let copy_stm = format!(
            "COPY upserts ({}, {}) FROM STDIN BINARY",
            pk_columns, columns
        );
        // TODO: For now each column is stored as a JSON field.
        // Later we can introduce some kind of type infering mechanism here, so each field in
        // Materialized view would be stored with better column type.
        // We can achieve that by matching `Value` variants (leaving `JSON` for complex objects like arrays or structs).

        let pk_columns = pk_fields.iter().map(|f| f.sql_name.to_owned()).join(", ");
        let pk_columns_with_types = pk_fields
            .iter()
            .map(|f| format!("{} UUID NOT NULL", f.sql_name))
            .join(", ");
        let columns_with_types = fields
            .iter()
            .map(|f| format!("{} JSON NOT NULL", f.sql_name))
            .join(", ");
        let create_stm = format!(
            "CREATE TABLE IF NOT EXISTS {table} ( \
                    {pk_columns_with_types},\
                    {columns_with_types},\
                    PRIMARY KEY ({pk_columns}) \
                 );\
                 CREATE TEMP TABLE upserts ON COMMIT DROP \
                 AS TABLE {table} WITH NO DATA;",
            table = table,
            columns_with_types = columns_with_types,
            pk_columns_with_types = pk_columns_with_types,
            pk_columns = pk_columns
        );

        let update_columns = fields
            .iter()
            .map(|f| format!("{field} = EXCLUDED.{field}", field = f.sql_name))
            .join(", ");

        let insert_stm = format!(
            "INSERT INTO {} \
        SELECT * FROM upserts \
        ON CONFLICT ({}) DO UPDATE SET {}",
            table, pk_columns, update_columns
        );

        tracing::debug!(
            ?copy_stm,
            ?insert_stm,
            ?create_stm,
            ?types,
            ?fields,
            "Build query"
        );
        Ok((copy_stm, insert_stm, create_stm, types, fields))
    }

    pub async fn new(args: &PostgresSettings) -> anyhow::Result<Self> {
        let mut pg_config = Config::new();
        pg_config
            .user(&args.username)
            .password(&args.password)
            .host(&args.host)
            .port(args.port)
            .dbname(&args.dbname);

        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .await?;

        Ok(Self {
            pool,
            schema: args.schema.clone(),
        })
    }
}

fn get_pk_field_list(definition: &FullView) -> Vec<Field> {
    let mut fields = vec![Field {
        json_path: "".to_owned(),
        name: format!("{}{}", SCHEMA_COLUMN_PREFIX, 0),
        sql_name: format!("{}{}", SCHEMA_COLUMN_PREFIX, 0),
        type_: Type::UUID,
    }];

    for relation in &definition.relations {
        let name = format!("{}{}", SCHEMA_COLUMN_PREFIX, relation.local_id);
        fields.push(Field {
            json_path: "".to_owned(),
            name: name.clone(),
            sql_name: name,
            type_: Type::UUID,
        })
    }
    fields
}

fn get_field_list(definition: &FullView) -> Vec<Field> {
    #[derive(Debug)]
    struct PartialFieldDefinition<'a> {
        sql_name: String,
        field_name: String,
        json_path: Vec<String>,
        definition: &'a FieldDefinition,
    }

    let mut fields = vec![];

    let mut fields_to_process = definition
        .fields
        .iter()
        .map(|x| PartialFieldDefinition {
            json_path: vec!["".to_owned()],
            sql_name: x.0.to_owned(),
            field_name: x.0.to_owned(),
            definition: x.1,
        })
        .collect::<VecDeque<_>>();

    while let Some(field) = fields_to_process.pop_front() {
        match field.definition {
            FieldDefinition::Simple { .. } | FieldDefinition::Computed { .. } => {
                fields.push(Field {
                    sql_name: field.sql_name,
                    name: field.field_name,
                    json_path: field.json_path.join("/"),
                    type_: Type::JSON,
                });
            }
            FieldDefinition::SubObject { fields, .. } => {
                fields_to_process = fields
                    .iter()
                    .map(|x| PartialFieldDefinition {
                        definition: x.1,
                        sql_name: format!("{}_{}", field.sql_name, x.0),
                        field_name: field.field_name.clone(),
                        json_path: {
                            let mut vec = field.json_path.clone();
                            vec.push(x.0.clone());
                            vec
                        },
                    })
                    .chain(fields_to_process.into_iter())
                    .collect()
            }
        }
    }

    fields
}

impl PostgresMaterializer {
    async fn set_schema(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    ) -> anyhow::Result<()> {
        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await?;

        Ok(())
    }

    async fn connect(
        &self,
    ) -> anyhow::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        let conn = self.pool.get().await?;

        self.set_schema(&conn).await?;

        Ok(conn)
    }
}
