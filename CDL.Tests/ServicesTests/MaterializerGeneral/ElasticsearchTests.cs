using System;
using System.Collections.Generic;
using System.Linq;
using AutoFixture;
using CDL.Tests.Configuration;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using CDL.Tests.Utils;
using EdgeRegistry;
using MassTransit.KafkaIntegration;
using Microsoft.Extensions.Options;
using NJsonSchema;
using SchemaRegistry;
using Xunit;
using Computation = CDL.Tests.ServiceObjects.SchemaService.Computation;
using FieldValueComputation = CDL.Tests.ServiceObjects.SchemaService.FieldValueComputation;
using SchemaType = SchemaRegistry.SchemaType;

namespace CDL.Tests.ServicesTests.MaterializerGeneral
{
    public class ElasticsearchTests
    {

        private readonly EdgeRegistryService _edgeRegistryService;

        private readonly Fixture _fixture;
        private readonly ITopicProducer<InsertObject> _kafkaProducer;
        private readonly ConfigurationOptions _options;
        private readonly SchemaRegistryService _schemaRegistryService;

        public ElasticsearchTests(
            ITopicProducer<InsertObject> kafkaProducer,
            SchemaRegistryService schemaRegistryService,
            EdgeRegistryService edgeRegistryService,
            Fixture fixture,
            IOptions<ConfigurationOptions> options)
        {
            _kafkaProducer = kafkaProducer;
            _schemaRegistryService = schemaRegistryService;
            _edgeRegistryService = edgeRegistryService;
            _fixture = fixture;
            _options = options.Value;
        }

        [Fact]
        public void SingleSchema_NoRelations()
        {
            var indexName = _fixture.Create<string>();
            var objectData = _fixture.Create<Person>();
            var objectId = _fixture.Create<string>();

            var schemaDefinition = JsonSchema.FromType(typeof(Person)).ToJson();

            var schemaId = _schemaRegistryService.AddSchema("test_schema", schemaDefinition, SchemaType.Types.Type.DocumentStorage).Result;

            var materializerFields = new Dictionary<string, object>();
            materializerFields.Add("firstName", new SimpleFiled
            {
                simple = new SimpleItem
                {
                    field_name = "firstName",
                    field_type = "string"
                }
            });

            _schemaRegistryService.AddViewToSchema(schemaId.Id_, "test_view", materializerFields, new List<Relation>(), $"{{ \"index_name\": \"{indexName}\" }}", MaterializerBackend.ElasticSearch);

            _kafkaProducer.Produce(new InsertObject
            {
                objectId = objectId,
                schemaId = schemaId.Id_,
                data = objectData
            });

            var result = RetryHelper.TryFetch(() => ElasticsearchConnector.QueryAll<PersonView1>(new Uri(_options.CDL_ELASTICSEARCH_NODE), indexName), arr => arr.Count > 0).ToArray();

            Assert.Single(result);
            Assert.Equal(objectData.FirstName, result[0].FirstName);
        }

        [Fact]
        public void SingleSchema_WithRelations()
        {

            var indexName = _fixture.Create<string>();
            var object1Data = _fixture.Create<Person>();
            var object1Id = _fixture.Create<string>();
            var object2Data = _fixture.Create<Person>();
            var object2Id = _fixture.Create<string>();

            var schemaDefinition = JsonSchema.FromType(typeof(Person)).ToJson();

            var schemaId = _schemaRegistryService.AddSchema("test_schema1", schemaDefinition, SchemaType.Types.Type.DocumentStorage).Result;

            var relationId = _edgeRegistryService.AddRelation(schemaId.Id_, schemaId.Id_).Result;

            var materializerFields = new Dictionary<string, object>();
            materializerFields.Add("firstName", new SimpleFiled
            {
                simple = new SimpleItem
                {
                    field_name = "firstName",
                    field_type = "string"
                }
            });
            materializerFields.Add("otherPersonFirstName", new ComputedFiled
            {
                computed = new Computed
                {
                    field_type = "string",
                    computation = new Computation
                    {
                        field_value = new FieldValueComputation
                        {
                            schema_id = 1,
                            field_path = "firstName"
                        }
                    }
                }
            });

            _schemaRegistryService.AddViewToSchema(schemaId.Id_, "test_view", materializerFields, new List<Relation>
            {
                new()
                {
                    GlobalId = relationId.RelationId_,
                    LocalId = 1,
                    SearchFor = new SearchFor
                    {
                        SearchFor_ = SearchFor.Types.Direction.Children
                    }
                }
            }, $"{{ \"index_name\": \"{indexName}\" }}", MaterializerBackend.ElasticSearch);

            _kafkaProducer.Produce(new InsertObject
            {
                objectId = object1Id,
                schemaId = schemaId.Id_,
                data = object1Data
            });

            _kafkaProducer.Produce(new InsertObject
            {
                objectId = object2Id,
                schemaId = schemaId.Id_,
                data = object2Data
            });

            var edge = _edgeRegistryService.AddEdges(new List<Edge>
            {
                new()
                {
                    ChildObjectIds =
                    {
                        object2Id
                    },
                    ParentObjectId = object1Id,
                    RelationId = relationId.RelationId_
                }
            }).Result;

            var result = RetryHelper.TryFetch(() => ElasticsearchConnector.QueryAll<PersonView2>(new Uri(_options.CDL_ELASTICSEARCH_NODE), indexName), arr => arr.Count > 0).ToArray();

            Assert.Single(result);
            Assert.Equal(object1Data.FirstName, result[0].FirstName);
            Assert.Equal(object2Data.FirstName, result[0].OtherPersonFirstName);
        }
        
        [Fact]
        public void MultipleSchema_WithRelations()
        {

            var indexName = _fixture.Create<string>();
            var object1Data = _fixture.Create<Person>();
            var object1Id = _fixture.Create<string>();
            var object2Data = _fixture.Create<Person>();
            var object2Id = _fixture.Create<string>();

            var schemaDefinition = JsonSchema.FromType(typeof(Person)).ToJson();

            var schema1Id = _schemaRegistryService.AddSchema("test_schema1", schemaDefinition, SchemaType.Types.Type.DocumentStorage).Result;
            var schema2Id = _schemaRegistryService.AddSchema("test_schema2", schemaDefinition, SchemaType.Types.Type.DocumentStorage).Result;

            var relationId = _edgeRegistryService.AddRelation(schema2Id.Id_, schema1Id.Id_).Result;

            var materializerFields = new Dictionary<string, object>();
            materializerFields.Add("firstName", new SimpleFiled
            {
                simple = new SimpleItem
                {
                    field_name = "firstName",
                    field_type = "string"
                }
            });
            materializerFields.Add("otherPersonFirstName", new ComputedFiled
            {
                computed = new Computed
                {
                    field_type = "string",
                    computation = new Computation
                    {
                        field_value = new FieldValueComputation
                        {
                            schema_id = 1,
                            field_path = "firstName"
                        }
                    }
                }
            });

            _schemaRegistryService.AddViewToSchema(schema1Id.Id_, "test_view", materializerFields, new List<Relation>
            {
                new()
                {
                    GlobalId = relationId.RelationId_,
                    LocalId = 1,
                    SearchFor = new SearchFor
                    {
                        SearchFor_ = SearchFor.Types.Direction.Children
                    }
                }
            }, $"{{ \"index_name\": \"{indexName}\" }}", MaterializerBackend.ElasticSearch);

            _kafkaProducer.Produce(new InsertObject
            {
                objectId = object1Id,
                schemaId = schema1Id.Id_,
                data = object1Data
            });

            _kafkaProducer.Produce(new InsertObject
            {
                objectId = object2Id,
                schemaId = schema2Id.Id_,
                data = object2Data
            });

            var edge = _edgeRegistryService.AddEdges(new List<Edge>
            {
                new()
                {
                    ChildObjectIds =
                    {
                        object2Id
                    },
                    ParentObjectId = object1Id,
                    RelationId = relationId.RelationId_
                }
            }).Result;

            var result = RetryHelper.TryFetch(() => ElasticsearchConnector.QueryAll<PersonView2>(new Uri(_options.CDL_ELASTICSEARCH_NODE), indexName), arr => arr.Count > 0).ToArray();

            Assert.Single(result);
            Assert.Equal(object1Data.FirstName, result[0].FirstName);
            Assert.Equal(object2Data.FirstName, result[0].OtherPersonFirstName);
        }
        
        public class PersonView1
        {
            public string FirstName { get; set; }
        }

        public class PersonView2
        {
            public string FirstName { get; set; }
            public string OtherPersonFirstName { get; set; }
        }
    }
}