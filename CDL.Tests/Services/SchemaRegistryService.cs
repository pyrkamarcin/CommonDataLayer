using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Threading.Tasks;
using CDL.Tests.Configuration;
using CDL.Tests.ServiceObjects.SchemaService;
using Common;
using Google.Protobuf;
using Grpc.Core;
using Microsoft.Extensions.Options;
using SchemaRegistry;
using static SchemaRegistry.SchemaRegistry;

namespace CDL.Tests.Services
{
    public enum MaterializerBackend
    {
        Postgres,
        ElasticSearch,
    }

    public static class MaterializerBackendExtension
    {
        public static string Address(this MaterializerBackend backend, ConfigurationOptions options)
        {
            return backend switch
            {
                MaterializerBackend.Postgres => options.CDL_MATERIALIZER_GENERAL_POSTGRESQL_ADDRESS,
                MaterializerBackend.ElasticSearch => options.CDL_MATERIALIZER_GENERAL_ELASTICSEARCH_ADDRESS,
                _ => throw new ArgumentOutOfRangeException(nameof(backend), backend, null)
            };
        }
    }
    
    public class SchemaRegistryService
    {
        private SchemaRegistryClient _client;
        private ConfigurationOptions _options;
        public SchemaRegistryService(SchemaRegistryClient client, IOptions<ConfigurationOptions> options)
        {
            _client = client;
            _options = options.Value;
        }

        public Task<Id> AddSchema(string name, string definition, SchemaType.Types.Type type)
        {
            var schema = new NewSchema()
            {
                InsertDestination = _options.CDL_SCHEMA_REGISTRY_DESTINATION,
                Name = name,
                QueryAddress = _options.CDL_QUERY_SERVICE_ADDRESS,
                SchemaType = new SchemaType(){
                    SchemaType_ = type
                },
                Definition = ByteString.CopyFromUtf8(definition)
            };
            
            return Task.FromResult(_client.AddSchema(schema));
        }

        public Task<Empty> UpdateSchema(string schemaId, string name, string definition, SchemaType.Types.Type type)
        {
            var newUpdateSchema = new SchemaUpdate()
            {
                Id = schemaId,
                InsertDestination = _options.CDL_SCHEMA_REGISTRY_DESTINATION,
                QueryAddress = _options.CDL_QUERY_SERVICE_ADDRESS,
                Name = name,
                SchemaType = new SchemaType(){
                    SchemaType_ = type
                },
                Definition = ByteString.CopyFromUtf8(definition)
            };

            return Task.FromResult(_client.UpdateSchema(newUpdateSchema));
        }

        public async Task<AsyncUnaryCall<Empty>> UpdateSchemaAsync(string schemaId, string name, string definition, SchemaType.Types.Type type)
        {
            var newUpdateSchema = new SchemaUpdate()
            {
                Id = schemaId,
                InsertDestination = _options.CDL_SCHEMA_REGISTRY_DESTINATION,
                QueryAddress = _options.CDL_QUERY_SERVICE_ADDRESS,
                Name = name,
                SchemaType = new SchemaType(){
                    SchemaType_ = type
                },
                Definition = ByteString.CopyFromUtf8(definition)
            };

            return await Task.FromResult(_client.UpdateSchemaAsync(newUpdateSchema));
        }

        public Task<Empty> UpdateView(string viewId, string name, bool updateFields, bool updateRelations, IDictionary<string, object> viewFields, IList<Relation> relations, string materializerOptions = "{}", MaterializerBackend materializerBackend = MaterializerBackend.Postgres)
        {
            var view = new ViewUpdate()
            {
                Id = viewId,
                MaterializerAddress = materializerBackend.Address(_options),
                MaterializerOptions = materializerOptions,
                Name = name,
                UpdateFields = updateFields,
                UpdateRelations = updateRelations
            };

            if (view.UpdateFields)
            {
                foreach (var field in viewFields)
                {
                    view.Fields.Add(field.Key, JsonSerializer.Serialize(field.Value));
                }
            }

            if (view.UpdateRelations)
            {
                foreach (var item in relations)
                {
                    view.Relations.Add(item);
                } 
            }

            return Task.FromResult(_client.UpdateView(view));
        }

        public async Task UpdateViewAsync(string viewId, string name, bool updateFields, bool updateRelations, IDictionary<string, object> viewFields, IList<Relation> relations, string materializerOptions = "{}", MaterializerBackend materializerBackend = MaterializerBackend.Postgres)
        {
            var view = new ViewUpdate()
            {
                Id = viewId,
                MaterializerAddress = materializerBackend.Address(_options),
                MaterializerOptions = materializerOptions,
                Name = name,
                UpdateFields = updateFields,
                UpdateRelations = updateRelations
            };
            
            if (view.UpdateFields)
            {
                foreach (var field in viewFields)
                {
                    view.Fields.Add(field.Key, JsonSerializer.Serialize(field.Value));
                }
            }

            if (view.UpdateRelations)
            {
                foreach (var item in relations)
                {
                    view.Relations.Add(item);
                } 
            }
            await Task.FromResult(_client.UpdateViewAsync(view));
        }

        public Task<Schema> GetSchema(string schemaId)
        {
            return Task.FromResult(_client.GetSchema(new Id()
            {
                Id_ = schemaId
            }));
        }

        public async Task<AsyncUnaryCall<Schema>> GetSchemaAsync(string schemaId)
        {
            return await Task.FromResult(_client.GetSchemaAsync(new Id()
            {
                Id_ = schemaId,
            }));
        }

        public Task<FullView> GetView(string viewId)
        {
            return Task.FromResult(_client.GetView(new Id()
            {
                Id_ = viewId
            }));
        }

        public async Task<AsyncUnaryCall<FullView>> GetViewAsync(string viewId)
        {
            return await Task.FromResult(_client.GetViewAsync(new Id()
            {
                Id_ = viewId
            }));
        }

        public Task<Schemas> GetAllSchemas()
        {
            return Task.FromResult(_client.GetAllSchemas(new Empty()));
        }

        public async Task<AsyncUnaryCall<Schemas>> GetAllSchemasAsync()
        {
            return await Task.FromResult(_client.GetAllSchemasAsync(new Empty()));
        }

        public Task<SchemaViews> GetAllViewsByRelation(string relationId)
        {
            var id = new Id(){
                Id_ = relationId
            };
            
            return Task.FromResult(_client.GetAllViewsByRelation(id));
        }

        public async Task<AsyncUnaryCall<SchemaViews>> GetAllViewsByRelationAsync(string relationId)
        {
            var id = new Id(){
                Id_ = relationId
            };
            
            return await Task.FromResult(_client.GetAllViewsByRelationAsync(id));
        }

        public Task<SchemaViews> GetAllViewsOfSchema(string schemaId)
        {
            var id = new Id(){
                Id_ = schemaId
            };

            return Task.FromResult(_client.GetAllViewsOfSchema(id));
        }

        public async Task<AsyncUnaryCall<SchemaViews>> GetAllViewsOfSchemaAsync(string schemaId)
        {
            var id = new Id(){
                Id_ = schemaId
            };

            return await Task.FromResult(_client.GetAllViewsOfSchemaAsync(id));
        }

        public Task<Schema> GetBaseSchemaOfView(string viewId)
        {
            var id = new Id(){
                Id_ = viewId
            };
            return Task.FromResult(_client.GetBaseSchemaOfView(id));
        }

        public async Task<AsyncUnaryCall<Schema>> GetBaseSchemaOfViewAsync(string viewId)
        {
            var id = new Id(){
                Id_ = viewId
            };
            return await Task.FromResult(_client.GetBaseSchemaOfViewAsync(id));
        }

        public Task<FullSchema> GetFullSchema(string schemaId)
        {
            return Task.FromResult(_client.GetFullSchema(new Id()
            {
                Id_ = schemaId
            }));
        }

        public async Task<AsyncUnaryCall<FullSchema>> GetFullSchemaAsync(string schemaId)
        {
            return await Task.FromResult(_client.GetFullSchemaAsync(new Id()
            {
                Id_ = schemaId
            }));
        }   

        public Task<FullSchemas> GetAllFullSchemas()
        {
            return Task.FromResult(_client.GetAllFullSchemas(new Empty()));
        }

        public Task<AsyncUnaryCall<FullSchemas>> GetAllFullSchemasAsync()
        {
            return Task.FromResult(_client.GetAllFullSchemasAsync(new Empty()));
        }

        public Task<Id> AddViewToSchema(string schemaId, string name, IDictionary<string, object> materializerFields, IList<Relation> relations, string materializerOptions = "{}", MaterializerBackend materializerBackend = MaterializerBackend.Postgres)
        {
            
            var view = new NewView()
            {
                BaseSchemaId = schemaId,
                Name = name,
                MaterializerAddress = materializerBackend.Address(_options),
                MaterializerOptions = materializerOptions,
            };

            foreach (var field in materializerFields)
            {
                view.Fields.Add(field.Key, JsonSerializer.Serialize(field.Value));
            }

            foreach (var item in relations)
            {
                view.Relations.Add(item);
            }            
            
            return Task.FromResult(_client.AddViewToSchema(view));
        }

        public async Task<AsyncUnaryCall<Id>> AddViewToSchemaAsync(string schemaId, string name, string materializerOptions = "{}", MaterializerBackend materializerBackend = MaterializerBackend.Postgres)
        {
            var view = new NewView()
            {
                BaseSchemaId = schemaId,
                Name = name,
                MaterializerAddress = materializerBackend.Address(_options),
                MaterializerOptions = materializerOptions,
            };

            return await Task.FromResult(_client.AddViewToSchemaAsync(view));
        }

        public Task<Errors> ValidateValue(string schemaId, string valueToCheck)
        {
            var valueToValidate = new ValueToValidate()
            {
                SchemaId = schemaId,
                Value = ByteString.CopyFromUtf8(valueToCheck)
            };
            return Task.FromResult(_client.ValidateValue(valueToValidate));
        }

        public async Task<AsyncUnaryCall<Errors>> ValidateValueAsync(string schemaId, string valueToCheck)
        {
            var valueToValidate = new ValueToValidate()
            {
                SchemaId = schemaId,
                Value = ByteString.CopyFromUtf8(valueToCheck)
            };
            return await Task.FromResult(_client.ValidateValueAsync(valueToValidate));
        }

        public async Task<AsyncServerStreamingCall<Schema>> WatchAllSchemaUpdatesAsync(CallOptions options)
        {
            return await Task.FromResult(_client.WatchAllSchemaUpdates(new Empty(), options));
        }

        public Task<Empty> Heartbeat()
        {
            return Task.FromResult(_client.Heartbeat(new Empty()));
        }
        public async Task<AsyncUnaryCall<Empty>> HeartbeatAsync()
        {
            return await Task.FromResult(_client.HeartbeatAsync(new Empty()));
        }
    }
}
