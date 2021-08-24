use anyhow::{Context, Result};
use cdl_dto::materialization::Relation;
use serde_json::Value;

pub fn get_sub_object(value: &Value, mut path: std::str::Split<char>) -> Result<Value> {
    match path.next() {
        Some(next) => {
            let object = value
                .as_object()
                .with_context(|| format!("Expected `{}` to be a JSON object", next))?;
            let sub_object = object
                .get(next)
                .with_context(|| format!("Field `{}` is missing", next))?;

            get_sub_object(sub_object, path)
        }
        None => Ok(value.clone()),
    }
}

pub fn flat_relation(rel: &Relation) -> Vec<&Relation> {
    Some(rel)
        .into_iter()
        .chain(rel.relations.iter().flat_map(flat_relation))
        .collect()
}
