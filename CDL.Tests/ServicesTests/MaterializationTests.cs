using System;
using System.Collections.Generic;
using System.Threading;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using Common;
using EdgeRegistry;
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
            var viewFields = new Dictionary<string, object>();
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;
            var relationForView = new List<Relation>();
            
            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = _fixture.Create<UInt32>(),
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });
            
            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("lastName", new SimpleFiled()
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

            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });

            //should have only one row
            while (await res.ResponseStream.MoveNext())
            {
                Assert.True(res.ResponseStream.Current.Fields.Count == 2);
                Assert.Contains(payload_a.FirstName, res.ResponseStream.Current.Fields["firstName"]);
                Assert.Contains(payload_a.LastName, res.ResponseStream.Current.Fields["lastName"]);
            }
        }

        [Fact]
        public async void MaterializerOnDemand_Join_DifferentSchema_SearchForParents()
        {
            var name_a = _fixture.Create<string>();
            var name_b = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name_a, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schema_b = _schemaRegistryService.AddSchema(
                name_b, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_b.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameFromSchemaB", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Parents,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_b.Id_,
                objectId = objectId_b,
                data = payload_b
            });
            
            var newEdge = new Edge()
            {
                RelationId = relation.RelationId_,
                ParentObjectId = objectId_a                
            };
            newEdge.ChildObjectIds.Add(objectId_b);
            await _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });  
            Thread.Sleep(1001);
            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamIsValid = false;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamIsValid = true;
                var currentRow = res.ResponseStream.Current;
                Assert.True(currentRow.Fields.Count == 2, $"Rows:{currentRow.Fields}");
                Assert.Contains(payload_b.FirstName, currentRow.Fields["firstName"]);
                Assert.Contains(payload_a.FirstName, currentRow.Fields["firstNameFromSchemaB"]);
            }
            Assert.True(responseStreamIsValid, "Problem with materialization of view. Response stream is empty or corrupted.");
        }

        [Fact]
        public async void MaterializerOnDemand_Join_DifferentSchema_SearchForChildren()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var schema_b = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_b.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameFromSchemaB", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_b.Id_,
                objectId = objectId_b,
                data = payload_b
            });
            
            var newEdge = new Edge()
            {
                RelationId = relation.RelationId_,
                ParentObjectId = objectId_a                
            };
            newEdge.ChildObjectIds.Add(objectId_b);
            await _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });  
            Thread.Sleep(1001);
            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamIsValid = false;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamIsValid = true;
                var currentRow = res.ResponseStream.Current;
                Assert.True(currentRow.Fields.Count == 2, $"Rows:{currentRow.Fields}");
                Assert.Contains(payload_a.FirstName, currentRow.Fields["firstName"]);
                Assert.Contains(payload_b.FirstName, currentRow.Fields["firstNameFromSchemaB"]);
            }
            Assert.True(responseStreamIsValid, "Problem with materialization. Response stream is empty or corrupted.");
        }

        [Fact]
        public async void MaterializerOnDemand_Join_SameSchema_SearchForParents()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameB", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Parents,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_b,
                data = payload_b
            });
            
            var newEdge = new Edge()
            {
                RelationId = relation.RelationId_,
                ParentObjectId = objectId_a                
            };
            newEdge.ChildObjectIds.Add(objectId_b);
            await _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });  
            Thread.Sleep(1001);
            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamIsValid = false;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamIsValid = true;
                var currentRow = res.ResponseStream.Current;
                Assert.True(currentRow.Fields.Count == 2, $"Rows:{currentRow.Fields}");
                Assert.Contains(payload_b.FirstName, currentRow.Fields["firstName"]);
                Assert.Contains(payload_a.FirstName, currentRow.Fields["firstNameB"]);
            }
            Assert.True(responseStreamIsValid, "Problem with materialization. Response stream is empty or corrupted.");
        }        

        [Fact]
        public async void MaterializerOnDemand_Join_SameSchema_SearchForChildren()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameB", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_b,
                data = payload_b
            });
            
            var newEdge = new Edge()
            {
                RelationId = relation.RelationId_,
                ParentObjectId = objectId_a                
            };
            newEdge.ChildObjectIds.Add(objectId_b);
            await _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });  
            Thread.Sleep(1001);
            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamIsValid = false;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamIsValid = true;
                var currentRow = res.ResponseStream.Current;
                Assert.True(currentRow.Fields.Count == 2, $"Rows:{currentRow.Fields}");
                Assert.Contains(payload_a.FirstName, currentRow.Fields["firstName"]);
                Assert.Contains(payload_b.FirstName, currentRow.Fields["firstNameB"]);
            }
            Assert.True(responseStreamIsValid, "Problem with materialization. Response stream is empty or corrupted.");
        }

        [Fact]
        public async void MaterializerOnDemand_NoEdge_NoResults()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameComputed", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_b,
                data = payload_b
            });
            Thread.Sleep(1001);

            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamShouldBeEmpty = true;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamShouldBeEmpty = false;
            }
            Assert.True(responseStreamShouldBeEmpty, "Response stream is not empty");
        }

        [Fact]
        public async void MaterializerOnDemand_NoOneOfObjects_NoResults()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var payload_a = _fixture.Create<Person>();
            var payload_b = _fixture.Create<Person>();
            var viewName = _fixture.Create<string>(); 
            var viewFields = new Dictionary<string, object>();
            var relationForView = new List<Relation>();
            var schema_a = _schemaRegistryService.AddSchema(
                name, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(schema_a.Id_, schema_a.Id_).Result;

            viewFields.Add("firstName", new SimpleFiled()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "firstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add("firstNameB", new ComputedFiled()
                {
                   computed = new Computed(){
                       computation = new ServiceObjects.SchemaService.Computation(){
                           field_value = new ServiceObjects.SchemaService.FieldValueComputation(){
                               schema_id = 1,
                               field_path = "firstName"
                           }
                       },
                       field_type = "String"
                   }
                });

            relationForView.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children,
                }
            });          
            
            var view = _schemaRegistryService.AddViewToSchema(schema_a.Id_, viewName, viewFields, relationForView).Result;            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema_a.Id_,
                objectId = objectId_a,
                data = payload_a
            });
            
            var newEdge = new Edge()
            {
                RelationId = relation.RelationId_,
                ParentObjectId = objectId_a                
            };
            newEdge.ChildObjectIds.Add(objectId_b);
            await _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });  
            Thread.Sleep(1001);
            var res = _onDemandMaterializerService.Materialize(view.Id_, new List<string>(){
                schema_a.Id_
            });
            
            bool responseStreamShouldBeEmpty = true;
            while (await res.ResponseStream.MoveNext())
            {
                responseStreamShouldBeEmpty = false;
            }
            Assert.True(responseStreamShouldBeEmpty, "Response stream is not empty");
        }
    }
}