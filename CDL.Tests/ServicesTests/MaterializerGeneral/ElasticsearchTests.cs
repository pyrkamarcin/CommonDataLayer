using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.Intrinsics.Arm;
using System.Security.Cryptography;
using System.Threading;
using AutoFixture;
using CDL.Tests.Configuration;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using CDL.Tests.Utils;
using MassTransit.KafkaIntegration;
using Microsoft.Extensions.Options;
using NJsonSchema;
using SchemaRegistry;
using Xunit;
using SchemaType = SchemaRegistry.SchemaType;

namespace CDL.Tests.ServicesTests.MaterializerGeneral
{
    public class ElasticsearchTests
    {
        private ITopicProducer<InsertObject> _kafkaProducer;

        private EdgeRegistryService _edgeRegistryService;
        private readonly SchemaRegistryService _schemaRegistryService;

        private readonly Fixture _fixture;
        private ConfigurationOptions _options;

        public ElasticsearchTests(ITopicProducer<InsertObject> kafkaProducer, SchemaRegistryService schemaRegistryService, EdgeRegistryService edgeRegistryService, Fixture fixture, IOptions<ConfigurationOptions> options)
        {
            _kafkaProducer = kafkaProducer;
            _schemaRegistryService = schemaRegistryService;
            _edgeRegistryService = edgeRegistryService;
            _fixture = fixture;
            _options = options.Value;
        }

        [Fact]
        public void SingleSchemaView()
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

            _kafkaProducer.Produce(new InsertObject()
            {
                objectId = objectId,
                schemaId = schemaId.Id_,
                data = objectData,
            });
            
            Thread.Sleep(1000);

            var result = ElasticsearchConnector.QueryAll<Person>(new Uri(_options.CDL_ELASTICSEARCH_NODE), indexName).ToArray();

            Assert.Single(result);
            Assert.Equal(objectData.FirstName, result[0].FirstName);
        }
    }
}