import { GraphQLClient } from "graphql-request";
import * as Dom from "graphql-request/dist/types.dom";
import gql from "graphql-tag";
export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> &
  { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> &
  { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /** A scalar that can represent any JSON value. */
  JSON: any;
  /** A scalar that can represent any JSON Object value. */
  JSONObject: any;
  UUID: any;
};

export type CdlObject = {
  __typename?: "CdlObject";
  objectId: Scalars["UUID"];
  data: Scalars["JSON"];
};

export type ComplexFilter = {
  __typename?: "ComplexFilter";
  operator: LogicOperator;
  operands: Array<Filter>;
};

export type Computation = {
  __typename?: "Computation";
  operator: ComputationOperator;
  lhs: Computation;
  rhs?: Maybe<Computation>;
};

export type ComputationOperator =
  | RawValueComputation
  | FieldValueComputation
  | EqualsComputation;

export type ComputedFilter = {
  __typename?: "ComputedFilter";
  computation: Computation;
};

/**
 * Schema definition stores information about data structure used to push object to database.
 * Each schema can have only one active definition, under latest version but also contains
 * history for backward compability.
 */
export type Definition = {
  __typename?: "Definition";
  /** Definition is stored as a JSON value and therefore needs to be valid JSON. */
  definition: Scalars["JSON"];
  /** Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist */
  version: Scalars["String"];
};

export type EdgeRelations = {
  __typename?: "EdgeRelations";
  relationId: Scalars["UUID"];
  parentObjectId: Scalars["UUID"];
  childObjectIds: Array<Scalars["UUID"]>;
};

export type EqualsComputation = {
  __typename?: "EqualsComputation";
  placeholder: Scalars["Boolean"];
};

export type FieldValueComputation = {
  __typename?: "FieldValueComputation";
  baseSchema?: Maybe<Scalars["UUID"]>;
  fieldPath: Scalars["String"];
};

/** View's filter */
export type Filter = SimpleFilter | ComplexFilter;

export enum FilterOperator {
  EqualsOp = "EQUALS_OP",
}

export type FilterValue =
  | SchemaFieldFilter
  | ViewPathFilter
  | RawValueFilter
  | ComputedFilter;

/** Schema is the format in which data is to be sent to the Common Data Layer. */
export type FullSchema = {
  __typename?: "FullSchema";
  /** Random UUID assigned on creation */
  id: Scalars["UUID"];
  /** The name is not required to be unique among all schemas (as `id` is the identifier) */
  name: Scalars["String"];
  /** Message queue insert_destination to which data is inserted by data-router. */
  insertDestination: Scalars["String"];
  /** Address of the query service responsible for retrieving data from DB */
  queryAddress: Scalars["String"];
  /** Whether this schema represents documents or timeseries data. */
  type: SchemaType;
  /**
   * Returns schema definition for given version.
   * Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
   * querying for "=2.1.0" will return "2.1.0" if exist
   */
  definition: Definition;
  /**
   * All definitions connected to this schema.
   * Each schema can have only one active definition, under latest version but also contains history for backward compability.
   */
  definitions: Array<Definition>;
  /** All views belonging to this schema. */
  views: Array<View>;
};

/** Schema is the format in which data is to be sent to the Common Data Layer. */
export type FullSchemaDefinitionArgs = {
  versionReq: Scalars["String"];
};

/** A view under a schema. */
export type FullView = {
  __typename?: "FullView";
  /** The ID of the view. */
  id: Scalars["UUID"];
  /** The ID of the base schema. */
  baseSchemaId: Scalars["UUID"];
  /** The name of the view. */
  name: Scalars["String"];
  /** The address of the materializer this view caches data in. */
  materializerAddress: Scalars["String"];
  /** Materializer's options encoded in JSON */
  materializerOptions: Scalars["JSON"];
  /** The fields that this view maps with. */
  fields: Scalars["JSON"];
  /** The relations that this view has. */
  relations: Array<Relation>;
};

export type InputMessage = {
  /** Object ID */
  objectId: Scalars["UUID"];
  /** Schema ID */
  schemaId: Scalars["UUID"];
  /** JSON-encoded payload */
  payload: Scalars["JSON"];
};

export enum LogicOperator {
  And = "AND",
  Or = "OR",
}

export type MaterializedView = {
  __typename?: "MaterializedView";
  /** Source view's UUID */
  id: Scalars["UUID"];
  /** Materialized objects */
  rows: Array<RowDefinition>;
};

export type MutationRoot = {
  __typename?: "MutationRoot";
  addSchema: FullSchema;
  addSchemaDefinition: Definition;
  addView: View;
  updateView: FullView;
  updateSchema: FullSchema;
  insertMessage: Scalars["Boolean"];
  insertBatch: Scalars["Boolean"];
  /** Add new relation, return generated `relation_id` */
  addRelation: Scalars["UUID"];
  /** Add new object-object edges */
  addEdges: Scalars["Boolean"];
};

export type MutationRootAddSchemaArgs = {
  new: NewSchema;
};

export type MutationRootAddSchemaDefinitionArgs = {
  schemaId: Scalars["UUID"];
  newVersion: NewVersion;
};

export type MutationRootAddViewArgs = {
  schemaId: Scalars["UUID"];
  viewId?: Maybe<Scalars["UUID"]>;
  newView: NewView;
};

export type MutationRootUpdateViewArgs = {
  id: Scalars["UUID"];
  update: ViewUpdate;
};

export type MutationRootUpdateSchemaArgs = {
  id: Scalars["UUID"];
  update: UpdateSchema;
};

export type MutationRootInsertMessageArgs = {
  message: InputMessage;
};

export type MutationRootInsertBatchArgs = {
  messages: Array<InputMessage>;
};

export type MutationRootAddRelationArgs = {
  relationId?: Maybe<Scalars["UUID"]>;
  parentSchemaId: Scalars["UUID"];
  childSchemaId: Scalars["UUID"];
};

export type MutationRootAddEdgesArgs = {
  relations: Array<ObjectRelations>;
};

/** Relation between a view's schemas */
export type NewRelation = {
  /** Relation ID stored in Edge Registry */
  globalId: Scalars["UUID"];
  /** Unique in view definition */
  localId: Scalars["Int"];
  /** Looking at relation which direction is important. */
  searchFor: SearchFor;
  /** Subrelations */
  relations: Array<NewRelation>;
};

/**
 * Input object which creates new schema and new definition. Each schema has to
 * contain at least one definition, which can be later overriden.
 */
export type NewSchema = {
  /** The name is not required to be unique among all schemas (as `id` is the identifier) */
  name: Scalars["String"];
  /** Address of the query service responsible for retrieving data from DB */
  queryAddress: Scalars["String"];
  /** Destination to which data is inserted by data-router. */
  insertDestination: Scalars["String"];
  /** Definition is stored as a JSON value and therefore needs to be valid JSON. */
  definition: Scalars["JSON"];
  /** Whether the schema stores documents or timeseries data. */
  type: SchemaType;
};

/** Input object which creates new version of existing schema. */
export type NewVersion = {
  /**
   * Schema is following semantic versioning, querying for "2.1.0" will
   * return "2.1.1" if it exists. When updating, new version has to be higher
   * than highest stored version in DB for given schema.
   */
  version: Scalars["String"];
  /** Definition is stored as a JSON value and therefore needs to be valid JSON. */
  definition: Scalars["JSON"];
};

/** A new view under a schema. */
export type NewView = {
  /** The name of the view. */
  name: Scalars["String"];
  /** The address of the materializer this view caches data in. */
  materializerAddress: Scalars["String"];
  /** Materializer's options encoded in JSON */
  materializerOptions: Scalars["JSON"];
  /** The fields that this view maps with. */
  fields: Scalars["JSON"];
  /** Filters to the fields */
  filters?: Maybe<Scalars["JSON"]>;
  /** The relations that this view has. */
  relations: Array<NewRelation>;
};

export type ObjectRelations = {
  /** Object's schema relations */
  relationId: Scalars["UUID"];
  /** Relation parent */
  parentObjectId: Scalars["UUID"];
  /** Relation children */
  childObjectIds: Array<Scalars["UUID"]>;
};

export type OnDemandViewRequest = {
  /** View's UUID */
  viewId: Scalars["UUID"];
  /** Schemas with objects. This collection is treated like a hash-map with `schemaId` as a key, therefore `schemaId` should be unique per request. */
  schemas: Array<Schema>;
};

export type QueryRoot = {
  __typename?: "QueryRoot";
  /** Return single schema for given id */
  schema: FullSchema;
  /** Return all schemas in database */
  schemas: Array<FullSchema>;
  /** Return single view for given id */
  view: FullView;
  /** Return a single object from the query router */
  object: CdlObject;
  /** Return a map of objects selected by ID from the query router */
  objects: Array<CdlObject>;
  /** Return a map of all objects (keyed by ID) in a schema from the query router */
  schemaObjects: Array<CdlObject>;
  /** Return schema `parent` is in `relation_id` relation with */
  relation?: Maybe<Scalars["UUID"]>;
  /** Return all relations `parent` is in */
  schemaRelations: Array<SchemaRelation>;
  /** List all relations between schemas stored in database */
  allRelations: Array<SchemaRelation>;
  /** Return all objects that `parent` object is in `relation_id` relation with */
  edge: Array<Scalars["UUID"]>;
  /** Return all relations that `parent` object is in */
  edges: Array<EdgeRelations>;
  /** On demand materialized view */
  onDemandView: MaterializedView;
};

export type QueryRootSchemaArgs = {
  id: Scalars["UUID"];
};

export type QueryRootViewArgs = {
  id: Scalars["UUID"];
};

export type QueryRootObjectArgs = {
  objectId: Scalars["UUID"];
  schemaId: Scalars["UUID"];
};

export type QueryRootObjectsArgs = {
  objectIds: Array<Scalars["UUID"]>;
  schemaId: Scalars["UUID"];
};

export type QueryRootSchemaObjectsArgs = {
  schemaId: Scalars["UUID"];
};

export type QueryRootRelationArgs = {
  relationId: Scalars["UUID"];
  parentSchemaId: Scalars["UUID"];
};

export type QueryRootSchemaRelationsArgs = {
  parentSchemaId: Scalars["UUID"];
};

export type QueryRootEdgeArgs = {
  relationId: Scalars["UUID"];
  parentObjectId: Scalars["UUID"];
};

export type QueryRootEdgesArgs = {
  parentObjectId: Scalars["UUID"];
};

export type QueryRootOnDemandViewArgs = {
  request: OnDemandViewRequest;
};

export type RawValueComputation = {
  __typename?: "RawValueComputation";
  value: Scalars["JSON"];
};

export type RawValueFilter = {
  __typename?: "RawValueFilter";
  value: Scalars["JSON"];
};

/** Relation between a view's schemas */
export type Relation = {
  __typename?: "Relation";
  /** Relation ID stored in Edge Registry */
  globalId: Scalars["UUID"];
  /** Unique in view definition */
  localId: Scalars["Int"];
  /** Looking at relation which direction is important. */
  searchFor: SearchFor;
  /** Subrelations */
  relations: Array<Relation>;
};

export type Report = {
  __typename?: "Report";
  /** Application which generated the report */
  application: Scalars["String"];
  /** Output plugin in command service */
  outputPlugin?: Maybe<Scalars["String"]>;
  /** Success/Failure */
  description: Scalars["String"];
  /** Object id */
  objectId: Scalars["UUID"];
  /** JSON encoded payload */
  payload: Scalars["JSON"];
};

export type RowDefinition = {
  __typename?: "RowDefinition";
  /** Object's UUID */
  objectId: Scalars["UUID"];
  /** Materialized fields */
  fields: Scalars["JSONObject"];
};

export type Schema = {
  /** Schema's UUID */
  id: Scalars["UUID"];
  /** List of the object IDs */
  objectIds: Array<Scalars["UUID"]>;
};

export type SchemaFieldFilter = {
  __typename?: "SchemaFieldFilter";
  schemaId: Scalars["Int"];
  fieldPath: Scalars["String"];
};

export type SchemaRelation = {
  __typename?: "SchemaRelation";
  relationId: Scalars["UUID"];
  childSchemaId: Scalars["UUID"];
  parentSchemaId: Scalars["UUID"];
};

export enum SchemaType {
  DocumentStorage = "DOCUMENT_STORAGE",
  Timeseries = "TIMESERIES",
}

export enum SearchFor {
  Parents = "PARENTS",
  Children = "CHILDREN",
}

export type SimpleFilter = {
  __typename?: "SimpleFilter";
  operator: FilterOperator;
  lhs: FilterValue;
  rhs?: Maybe<FilterValue>;
};

export type SubscriptionRoot = {
  __typename?: "SubscriptionRoot";
  reports: Report;
};

/**
 * Input object which updates fields in schema. All fields are optional,
 * therefore one may update only `topic` or `queryAddress` or all of them.
 */
export type UpdateSchema = {
  /** The name is not required to be unique among all schemas (as `id` is the identifier) */
  name?: Maybe<Scalars["String"]>;
  /** Address of the query service responsible for retrieving data from DB */
  queryAddress?: Maybe<Scalars["String"]>;
  /** Destination to which data is inserted by data-router. */
  insertDestination?: Maybe<Scalars["String"]>;
  /** Whether the schema stores documents or timeseries data. */
  type?: Maybe<SchemaType>;
};

/** A view under a schema. */
export type View = {
  __typename?: "View";
  /** The ID of the view. */
  id: Scalars["UUID"];
  /** The name of the view. */
  name: Scalars["String"];
  /** The address of the materializer this view caches data in. */
  materializerAddress: Scalars["String"];
  /** Materializer's options encoded in JSON */
  materializerOptions: Scalars["JSON"];
  /** The fields that this view maps with. */
  fields: Scalars["JSON"];
  /** The relations that this view has. */
  relations: Array<Relation>;
  /** Filters used to narrow source objects. */
  filters?: Maybe<Filter>;
};

export type ViewPathFilter = {
  __typename?: "ViewPathFilter";
  fieldPath: Scalars["String"];
};

/** An update to a view. Only the provided properties are updated. */
export type ViewUpdate = {
  /** The name of the view. */
  name?: Maybe<Scalars["String"]>;
  /** The address of the materializer this view caches data in. */
  materializerAddress?: Maybe<Scalars["String"]>;
  /** Materializer's options encoded in JSON */
  materializerOptions?: Maybe<Scalars["JSON"]>;
  /** The fields that this view maps with. */
  fields?: Maybe<Scalars["JSON"]>;
  /** Filters to the fields */
  filters?: Maybe<Scalars["JSON"]>;
  /** Should filters be updated if not present */
  cleanFilters?: Scalars["Boolean"];
  /** The relations that this view has. */
  relations?: Maybe<Array<NewRelation>>;
};

export type AllSchemasQueryVariables = Exact<{ [key: string]: never }>;

export type AllSchemasQuery = { __typename?: "QueryRoot" } & {
  schemas: Array<
    { __typename?: "FullSchema" } & Pick<
      FullSchema,
      "id" | "name" | "type" | "queryAddress" | "insertDestination"
    > & {
        definitions: Array<
          { __typename?: "Definition" } & Pick<
            Definition,
            "version" | "definition"
          >
        >;
      }
  >;
};

export const AllSchemasDocument = gql`
  query AllSchemas {
    schemas {
      id
      name
      type
      definitions {
        version
        definition
      }
      queryAddress
      insertDestination
    }
  }
`;

export type SdkFunctionWrapper = <T>(
  action: (requestHeaders?: Record<string, string>) => Promise<T>,
  operationName: string
) => Promise<T>;

const defaultWrapper: SdkFunctionWrapper = (action, _operationName) => action();

export function getSdk(
  client: GraphQLClient,
  withWrapper: SdkFunctionWrapper = defaultWrapper
) {
  return {
    AllSchemas(
      variables?: AllSchemasQueryVariables,
      requestHeaders?: Dom.RequestInit["headers"]
    ): Promise<AllSchemasQuery> {
      return withWrapper(
        (wrappedRequestHeaders) =>
          client.request<AllSchemasQuery>(AllSchemasDocument, variables, {
            ...requestHeaders,
            ...wrappedRequestHeaders,
          }),
        "AllSchemas"
      );
    },
  };
}
export type Sdk = ReturnType<typeof getSdk>;
