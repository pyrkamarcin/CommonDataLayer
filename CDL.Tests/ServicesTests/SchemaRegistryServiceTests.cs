using System;
using System.Collections.Generic;
using System.Text.Json;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;
using System.Linq;
using CDL.Tests.Configuration;
using Microsoft.Extensions.Options;
using System.Threading;

namespace CDL.Tests.ServicesTests
{
    public class SchemaRegistryServiceTests
    {
        private SchemaRegistryService _schemaRegistryService;
        private QueryRouterService _queryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;
        private OnDemandMaterializerService _onDemandMaterializerService;

        private EdgeRegistryService _edgeRegistryService;
        private ConfigurationOptions _options;

        public SchemaRegistryServiceTests(
            SchemaRegistryService schemaRegistryService, 
            QueryRouterService queryService, 
            ITopicProducer<InsertObject> kafkaProducer, 
            Fixture fixture,
            EdgeRegistryService edgeRegistryService,
            OnDemandMaterializerService onDemandMaterializerService,
            IOptions<ConfigurationOptions> options)
        {
            _schemaRegistryService = schemaRegistryService;
            _queryService = queryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
            _edgeRegistryService = edgeRegistryService;
            _onDemandMaterializerService = onDemandMaterializerService;
            _options = options.Value;
        }

        [Fact]
        [Trait("Category","Smoke")]
        public void Heartbeat()
        {
            var results = _schemaRegistryService.Heartbeat().Result;
            Assert.NotNull(results);
            Assert.IsType<Empty>(results);
        }

        [Theory]
        [InlineData("")]
        [InlineData("MyName")]
        [Trait("Category","Smoke")]
        public void AddSchema(string schemaName)
        {           
            var schema = _schemaRegistryService.AddSchema(
                schemaName,
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            Assert.NotNull(schema);
            Assert.IsType<string>(schema.Id_);
            Guid.Parse(schema.Id_);
        }

        [Fact]
        [Trait("Category","Smoke")]
        public void GetSchema()
        {
            var name = _fixture.Create<string>();
            var schema = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage
                ).Result;
            var results = _schemaRegistryService.GetFullSchema(schema.Id_).Result;

            Assert.IsType<FullSchema>(results);
            Assert.Contains(results.Name, name);
        }

        [Fact]
        [Trait("Category","Smoke")]
        public void CheckViewAddedToSchema()
        {
            var name = _fixture.Create<string>();
            var viewName = _fixture.Create<string>();            
            var schema = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var viewFields = new List<Simple>();
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "FirstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "LastName",
                        field_type = "String" 
                    }
                });
            var view = _schemaRegistryService.AddViewToSchema(schema.Id_, viewName, viewFields, new List<Relation>()).Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;
                     
            Assert.NotNull(view.Id_);
            Assert.IsType<string>(view.Id_);
            Assert.NotNull(viewDetails);
            Assert.IsType<FullView>(viewDetails);
            var viewUUID = Guid.Parse(view.Id_);   
            
            var schemaWithView = _schemaRegistryService.GetFullSchema(schema.Id_).Result;
            Assert.True(schemaWithView.Views.Count == 1);
            
            var viewObject = schemaWithView.Views[0];
            Assert.Equal(viewName, viewObject.Name);
            Assert.IsType<string>(viewObject.Id);
            Assert.NotEqual("{\"Name\": \"Name\" }", viewObject.MaterializerOptions);
            Assert.NotNull(viewDetails);
            Assert.IsType<FullView>(viewDetails);
        }

        [Theory]
        [InlineData("")]
        [InlineData("{field:value}")]
        [Trait("Category","Regression")]
        public void AddViewToSchema_wrongMaterializerOptions(string materializerOptions)
        {
            var name = _fixture.Create<string>();
            string exceptionMsg = string.Empty;
            var schema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            
            try
            {
                _schemaRegistryService.AddViewToSchema(schema.Id_, name, new List<Simple>(), new List<Relation>(), materializerOptions);
            }
            catch (Exception e)
            {
                exceptionMsg = e.Message;
            }

            Assert.NotNull(exceptionMsg);
            Assert.Contains("JSON error", exceptionMsg);
            Assert.Contains("StatusCode=\"Internal\"", exceptionMsg);
        }



        [Theory]
        [InlineData("")]
        [InlineData("{field:value}")]
        [Trait("Category","Regression")]
        public void AddSchema_WrongDefinition(string definition)
        {
            string exceptionMsg = string.Empty;
            try
            {
                var results = _schemaRegistryService.AddSchema(
                    string.Empty, 
                    definition, 
                    SchemaType.Types.Type.DocumentStorage).Result;
            }
            catch (Exception e)
            {
                exceptionMsg = e.Message;
            }

            Assert.NotNull(exceptionMsg);
            Assert.Contains("StatusCode=\"InvalidArgument\"", exceptionMsg);
            Assert.Contains("Invalid JSON provided", exceptionMsg);

        }

        [Fact]
        [Trait("Category","Smoke")]
        public void AddObjectToSchema()
        {
            var schema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            
            var schemaObjectsBefore = _queryService.GetAllObjectsFromSchema(schema.Id_).ExecuteWithRetryPolicy().Result;
            Assert.Equal(System.Net.HttpStatusCode.OK, schemaObjectsBefore.StatusCode);
            Assert.Equal("{}", schemaObjectsBefore.Content);
            var objectId = Guid.NewGuid().ToString();            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema.Id_,
                objectId = objectId,                
                data = _fixture.Create<GeneralObject>(),
            });

            var schemaObjectsAfter = _queryService.GetAllObjectsFromSchema(schema.Id_).ExecuteWithRetryPolicy(m => m.Content.Equals("{}")).Result;
            Assert.Equal(System.Net.HttpStatusCode.OK, schemaObjectsAfter.StatusCode);
            Assert.Contains(objectId, schemaObjectsAfter.Content);
        }

        
        [Fact]
        [Trait("Category","Smoke")]
        public void GetViewsByRelation()
        {
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage
                ).Result;
            var schema_b = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage
                ).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_b.Id_).Result;
            var relationForView = new List<Relation>();
            
            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = _fixture.Create<UInt32>(),
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });

            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, _fixture.Create<string>(), new List<Simple>(), relationForView);
            var allViewByRelations = _schemaRegistryService.GetAllViewsByRelation(relation.RelationId_).Result;

            Assert.True(allViewByRelations.Views.Count == 1);
            Assert.True(allViewByRelations.Views[0].BaseSchemaId.Equals(schema_a.Id_));
        } 

        [Fact]
        [Trait("Category","Regression")]
        public void GetAllViewsOfSchema()
        {
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
           
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;
            var relationForView = new List<Relation>();
            
            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = _fixture.Create<UInt32>(),
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });

            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, _fixture.Create<string>(), new List<Simple>(), relationForView);
            var allViewByRelations = _schemaRegistryService.GetAllViewsOfSchema(schema_a.Id_).Result;

            Assert.True(allViewByRelations.Views.Count == 1);
            Assert.True(allViewByRelations.Views[0].BaseSchemaId.Equals(schema_a.Id_));
        } 

        [Fact]
        public void UpdateSchema()
        {
            var name = _fixture.Create<string>();
            var definition = _fixture.Create<Car>().ToJSONString();
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(),
                definition,
                SchemaType.Types.Type.DocumentStorage).Result;

            _schemaRegistryService.UpdateSchema(schema_a.Id_, name, definition, SchemaType.Types.Type.DocumentStorage);

            var updatedSchema = _schemaRegistryService.GetSchema(schema_a.Id_).Result;

            Assert.Equal(name, updatedSchema.Name);
        }

        [Fact]
        [Trait("Category","Smoke")]
        public async void UpdateView()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>();            
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var viewFields = new List<Simple>();
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;
            var relationForView = new List<Relation>();
            
            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = _fixture.Create<UInt32>(),
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });
            
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "lastName",
                        field_type = "String" 
                    }
                });
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, new List<Relation>()).Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;
            var schemaWithView = _schemaRegistryService.GetFullSchema(schema_a.Id_).Result;
            Assert.True(schemaWithView.Views.Count == 1);
            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            }).Wait();
            Thread.Sleep(1000);
            

            var materializedViewBeforeUpdate = _onDemandMaterializerService.Materialize(view.Id_, schema_a.Id_, new List<string>(){
                objectId_a,
            });

            while (await materializedViewBeforeUpdate.ResponseStream.MoveNext(new System.Threading.CancellationToken()))
            {
                Assert.True(materializedViewBeforeUpdate.ResponseStream.Current.Fields.Count == 2);
                Assert.Contains(payload_a.FirstName, materializedViewBeforeUpdate.ResponseStream.Current.Fields["firstName"]);
                Assert.Contains(payload_a.LastName, materializedViewBeforeUpdate.ResponseStream.Current.Fields["lastName"]);
            }

            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "birthday",
                        field_type = "String" 
                    }
                });
            
            _schemaRegistryService.UpdateViewAsync(
                view.Id_, 
                viewDetails.Name, 
                true, 
                false,
                viewFields,
                new List<Relation>()).Wait();
            Thread.Sleep(1000);
            
            var viewDetailsAfterUpdate = _schemaRegistryService.GetView(view.Id_).Result;

            var materializedViewAfterUpdate = _onDemandMaterializerService.Materialize(view.Id_, schema_a.Id_, new List<string>(){
                objectId_a,
            });

            while (await materializedViewAfterUpdate.ResponseStream.MoveNext(new System.Threading.CancellationToken()))
            {

                Assert.Equal(materializedViewAfterUpdate.ResponseStream.Current.Fields.Count, 3);
                Assert.Contains(payload_a.FirstName, materializedViewAfterUpdate.ResponseStream.Current.Fields["firstName"]);
                Assert.Contains(payload_a.LastName, materializedViewAfterUpdate.ResponseStream.Current.Fields["lastName"]);
                Assert.Contains(payload_a.Birthday.ToString("s"), materializedViewAfterUpdate.ResponseStream.Current.Fields["birthday"]);
            }            
        }

        [Fact]
        [Trait("Category","Regression")]
        public void GetSchemaMetadata()
        {
            var name = _fixture.Create<string>();            
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schemaMetadata = _schemaRegistryService.GetSchema(schema_a.Id_).Result;
            Assert.Contains(_options.CDL_SCHEMA_REGISTRY_DESTINATION, schemaMetadata.InsertDestination);
            Assert.Contains(name, schemaMetadata.Name);
            Assert.Contains(_options.CDL_QUERY_SERVICE_ADDRESS, schemaMetadata.QueryAddress);
            Assert.Contains("DocumentStorage", schemaMetadata.SchemaType.SchemaType_.ToString());
        }

        [Fact]
        [Trait("Category","Regression")]
        public void GetSchemaDefinition()
        {
            var schemaDefinition = _fixture.Create<Car>();
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                schemaDefinition.ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schemaDefinitionSR= _schemaRegistryService.GetSchema(schema_a.Id_).Result;
            var schemaDefinitionFromResponse = JsonSerializer.Deserialize<Car>(schemaDefinitionSR.Definition.ToStringUtf8());

            Assert.True(schemaDefinition.Age == schemaDefinitionFromResponse.Age);
            Assert.True(schemaDefinition.Color == schemaDefinitionFromResponse.Color);
            Assert.True(schemaDefinition.Id == schemaDefinitionFromResponse.Id);
            Assert.True(schemaDefinition.Licence == schemaDefinitionFromResponse.Licence);
            Assert.True(schemaDefinition.Model == schemaDefinitionFromResponse.Model);
            Assert.True(schemaDefinition.Make == schemaDefinitionFromResponse.Make);

        }

        [Fact]
        [Trait("Category","Regression")]
        public void GetAllSchemas()
        {
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schema_b = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schema_c = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;

            var allSchemas = _schemaRegistryService.GetAllSchemas().Result;

            Assert.NotNull(allSchemas.Schemas_.Single(x => x.Id.Equals(schema_a.Id_)));
            Assert.NotNull(allSchemas.Schemas_.Single(x => x.Id.Equals(schema_b.Id_)));
            Assert.NotNull(allSchemas.Schemas_.Single(x => x.Id.Equals(schema_c.Id_)));
            
        }

        [Fact(DisplayName="Get base schema")]
        [Trait("Category","Regression")]
        public async void GetBaseSchemaOfView()
        {
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var viewFields = new List<Simple>();
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;
            var relationForView = new List<Relation>();
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, _fixture.Create<string>(), viewFields, new List<Relation>()).Result;
            var baseSchemaOfView = await _schemaRegistryService.GetBaseSchemaOfViewAsync(view.Id_).Result;
            Assert.Equal(schema_a.Id_, baseSchemaOfView.Id);
        }

        [Fact]
        public void GetAllFullSchemas()
        {
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var viewFields = new List<Simple>();
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;
            var relationForView = new List<Relation>();
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, _fixture.Create<string>(), viewFields, new List<Relation>()).Result;
            var fullSchemas = _schemaRegistryService.GetAllFullSchemas().Result;
            
            Assert.NotNull(fullSchemas.Schemas.Single(x => x.Id.Equals(schema_a.Id_)));
        }
        
        [Fact(Skip = "Test not available.")]
        public void ValidateValue()
        {
            var payload_a = _fixture.Create<GeneralObject>();
            var schema_a = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var objectId = Guid.NewGuid().ToString();            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId,                
                data = payload_a
            });

            var validValue = _schemaRegistryService.ValidateValue(schema_a.Id_, "{\"firstNameAWES\":\"firstNameAWES\"}").Result;
        }

        [Fact(Skip = "Test not available")]
        public void WatchAllSchemaUpdatesAsync()
        {
            throw new NotImplementedException();
        }

        
    }
}
