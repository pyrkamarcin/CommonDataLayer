using System;
using System.Collections.Generic;
using System.Threading;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using Grpc.Core;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class MaterializationTests
    {
        private OnDemandMaterializerService _onDemandMaterializerService;
        private SchemaRegistryService _schemaRegistryService;
        private QueryRouterService _queryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;
        private EdgeRegistryService _edgeRegistryService;

        public MaterializationTests(
            OnDemandMaterializerService onDemandMaterializerService,
            SchemaRegistryService schemaRegistryService, 
            QueryRouterService queryService, 
            ITopicProducer<InsertObject> kafkaProducer, 
            Fixture fixture,
            EdgeRegistryService edgeRegistryService)
        {
            _onDemandMaterializerService = onDemandMaterializerService;
            _schemaRegistryService = schemaRegistryService;
            _queryService = queryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
            _edgeRegistryService = edgeRegistryService;
        }

        [Fact]
        public void OnDemandMaterializerServiceHeartbeat()
        {
            var results = _onDemandMaterializerService.Heartbeat().Result;
            Assert.NotNull(results);
            Assert.IsType<MaterializerOndemand.Empty>(results);
        }

        [Fact]
        public async void MaterializerOnDemand()
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

            var res = _onDemandMaterializerService.Materialize(view.Id_, schema_a.Id_, new List<string>(){
                objectId_a,
            });

            //should have only one row
            while (await res.ResponseStream.MoveNext())
            {
                Assert.True(res.ResponseStream.Current.Fields.Count == 2);
                Assert.Contains(payload_a.FirstName, res.ResponseStream.Current.Fields["firstName"]);
                Assert.Contains(payload_a.LastName, res.ResponseStream.Current.Fields["lastName"]);
            }
        }
        
    }
}