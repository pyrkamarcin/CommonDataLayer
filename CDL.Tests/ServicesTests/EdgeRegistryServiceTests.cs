using System;
using System.Collections.Generic;
using System.Threading;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using Common;
using EdgeRegistry;
using Grpc.Core;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;
using static EdgeRegistry.Filter;
using static EdgeRegistry.SimpleFilterSide.Types;
using Empty = EdgeRegistry.Empty;

namespace CDL.Tests.ServicesTests
{
    public class EdgeRegistryServiceTests
    {
        private EdgeRegistryService _edgeRegistryService;
        private SchemaRegistryService _schemaRegistryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;
        
        public EdgeRegistryServiceTests(SchemaRegistryService schemaRegistryService, EdgeRegistryService edgeRegistryService, ITopicProducer<InsertObject> kafkaProducer, Fixture fixture)
        {
            _edgeRegistryService = edgeRegistryService;
            _schemaRegistryService = schemaRegistryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
        }

        [Fact]
        public void AddRelation()
        {
            var parentName = _fixture.Create<string>();
            var childName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(
                parentName, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                childName, 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var results = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            
            Assert.True(results.HasRelationId_);
            Guid.Parse(results.RelationId_);  
        }

        [Fact]
        public void GetRelation_ListRelations()
        {     
            var relationListBefore = _edgeRegistryService.GetRelation(new List<string>()).Result;
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var relationListAfter = _edgeRegistryService.GetRelation(new List<string>()).Result;

            Assert.True(relationListBefore.Items.Count < relationListAfter.Items.Count);
            
            var relations = new List<RelationDetails>(relationListAfter.Items);
            var item = relations.Find( z => z.RelationId == relation.RelationId_);
            Assert.NotNull(item);
            Assert.Equal(parentSchema.Id_, item.ParentSchemaId);
            Assert.True(item.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, item.ChildSchemaId);                   
        }

        [Fact]
        public void GetSchemaByRelation()
        {
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var schemaFromRelation = _edgeRegistryService.GetSchemaByRelation(relation.RelationId_).Result;

            Assert.Equal(parentSchema.Id_, schemaFromRelation.ParentSchemaId);
            Assert.True(schemaFromRelation.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, schemaFromRelation.ChildSchemaId);  
        }

        [Fact]
        public void ResolveTree()
        {
            var objectIdForParentSchema= Guid.NewGuid().ToString(); 
            var objectIdForChildSchema = Guid.NewGuid().ToString();                
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(
                parentSchemaName, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                childSchemaName, 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = objectIdForParentSchema,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = objectIdForChildSchema,
                data = _fixture.Create<Car>(),
            });       

            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = objectIdForParentSchema,                  
                };
            newEdge.ChildObjectIds.Add(objectIdForChildSchema);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    
            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });
            var tree = _edgeRegistryService.ResolveTree(relations).Result;

            Assert.True(tree.Rows.Count == 1);
            var treeObject = tree.Rows[0];
            Assert.True(treeObject.HasBaseObjectId);
            Assert.True(treeObject.BaseObjectId.Equals(objectIdForParentSchema));
            Assert.True(treeObject.RelationObjectIds.Count.Equals(1));
            Assert.True(treeObject.RelationObjectIds[0].Equals(objectIdForChildSchema));
        }



        [Fact]
        public void ResolveTree_FilteringByParent()
        {
            var parentObjectId= Guid.NewGuid().ToString(); 
            var parent2ObjectId= Guid.NewGuid().ToString(); 
            var childObjectId = Guid.NewGuid().ToString();                
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parentObjectId,
                data = _fixture.Create<Person>(),
            });            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parent2ObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = childObjectId,
                data = _fixture.Create<Car>(),
            });       

            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parentObjectId,                  
                };
            newEdge.ChildObjectIds.Add(childObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var newEdge2 = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parent2ObjectId,                  
                };
            newEdge2.ChildObjectIds.Add(childObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge2 });     

            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });
            var simpleFilter = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InParentObjIds}
                };
            simpleFilter.Ids.Add(parentObjectId);
            var filter = new EdgeRegistry.Filter{
                Simple = simpleFilter
            };
            var tree = _edgeRegistryService.ResolveTree(relations, filter).Result;

            Assert.Equal(1,tree.Rows.Count);
            var treeObject = tree.Rows[0];
            Assert.True(treeObject.HasBaseObjectId);
            Assert.True(treeObject.BaseObjectId.Equals(parentObjectId));
            Assert.True(treeObject.RelationObjectIds.Count.Equals(1));
            Assert.True(treeObject.RelationObjectIds[0].Equals(childObjectId));

        }
        [Fact]
        public void ResolveTree_FilteringByChild()
        {
            var parentObjectId= Guid.NewGuid().ToString(); 
            var childObjectId = Guid.NewGuid().ToString();                
            var child2ObjectId = Guid.NewGuid().ToString();                
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parentObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = childObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child2ObjectId,
                data = _fixture.Create<Car>(),
            });       

            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parentObjectId,                  
                };
            newEdge.ChildObjectIds.Add(childObjectId);
            newEdge.ChildObjectIds.Add(child2ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });
            var simpleFilter = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilter.Ids.Add(childObjectId);
            var filter = new EdgeRegistry.Filter{
                Simple = simpleFilter
            };
            var tree = _edgeRegistryService.ResolveTree(relations, filter).Result;

            Assert.Equal(1,tree.Rows.Count);
            var treeObject = tree.Rows[0];
            Assert.True(treeObject.HasBaseObjectId);
            Assert.True(treeObject.BaseObjectId.Equals(parentObjectId));
            Assert.True(treeObject.RelationObjectIds.Count.Equals(1));
            Assert.True(treeObject.RelationObjectIds[0].Equals(childObjectId));
        }
        [Fact]
        public void ResolveTree_ComplexFilterAND()
        {
            var parentObjectId= Guid.NewGuid().ToString(); 
            var parent2ObjectId= Guid.NewGuid().ToString(); 
            var childObjectId = Guid.NewGuid().ToString();                
            var child2ObjectId = Guid.NewGuid().ToString();                
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parentObjectId,
                data = _fixture.Create<Person>(),
            });            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parent2ObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = childObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child2ObjectId,
                data = _fixture.Create<Car>(),
            });         

            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parentObjectId,                  
                };
            newEdge.ChildObjectIds.Add(childObjectId);
            newEdge.ChildObjectIds.Add(child2ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var newEdge2 = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parent2ObjectId,                  
                };
            newEdge2.ChildObjectIds.Add(childObjectId);
            newEdge2.ChildObjectIds.Add(child2ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge2 });    

            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });
            var simpleFilterA = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InParentObjIds}
                };
            simpleFilterA.Ids.Add(parentObjectId);
            var filterA = new EdgeRegistry.Filter{
                Simple = simpleFilterA
            };

            var simpleFilterB = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilterB.Ids.Add(childObjectId);
            var filterB = new EdgeRegistry.Filter{
                Simple = simpleFilterB
            };

            var complexFilter = new EdgeRegistry.ComplexFilter{
                Operator = new LogicOperator(){
                    Operator = LogicOperator.Types.Operator.And
                }
            };
            complexFilter.Operands.Add(filterA);
            complexFilter.Operands.Add(filterB);
            var filter = new EdgeRegistry.Filter{
                Complex = complexFilter
            };
            var tree = _edgeRegistryService.ResolveTree(relations, filter).Result;

            Assert.Equal(1,tree.Rows.Count);
            var treeObject = tree.Rows[0];
            Assert.True(treeObject.HasBaseObjectId);
            Assert.True(treeObject.BaseObjectId.Equals(parentObjectId));
            Assert.True(treeObject.RelationObjectIds.Count.Equals(1));
            Assert.True(treeObject.RelationObjectIds[0].Equals(childObjectId));

        }
        [Fact]
        public void ResolveTree_ComplexFilterOR()
        {
            var parentObjectId= Guid.NewGuid().ToString(); 
            var childObjectId = Guid.NewGuid().ToString();                
            var child2ObjectId = Guid.NewGuid().ToString();                
            var child3ObjectId = Guid.NewGuid().ToString();                
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parentObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = childObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child2ObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child3ObjectId,
                data = _fixture.Create<Car>(),
            });         

            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parentObjectId,                  
                };
            newEdge.ChildObjectIds.Add(childObjectId);
            newEdge.ChildObjectIds.Add(child2ObjectId);
            newEdge.ChildObjectIds.Add(child3ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });
            var simpleFilterA = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilterA.Ids.Add(childObjectId);
            var filterA = new EdgeRegistry.Filter{
                Simple = simpleFilterA
            };

            var simpleFilterB = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilterB.Ids.Add(child2ObjectId);
            var filterB = new EdgeRegistry.Filter{
                Simple = simpleFilterB
            };

            var complexFilter = new EdgeRegistry.ComplexFilter{
                Operator = new LogicOperator(){
                    Operator = LogicOperator.Types.Operator.Or
                }
            };
            complexFilter.Operands.Add(filterA);
            complexFilter.Operands.Add(filterB);
            var filter = new EdgeRegistry.Filter{
                Complex = complexFilter
            };
            var tree = _edgeRegistryService.ResolveTree(relations, filter).Result;

            Assert.Equal(2,tree.Rows.Count);
            var rows =  new List<RelationTreeRow>(tree.Rows);
            var first = rows.Find((x)=>x.RelationObjectIds[0]==childObjectId);
            Assert.NotNull(first);
            Assert.True(first.HasBaseObjectId);
            Assert.True(first.BaseObjectId.Equals(parentObjectId));
            Assert.True(first.RelationObjectIds.Count.Equals(1));
            Assert.True(first.RelationObjectIds[0].Equals(childObjectId));
            var second = rows.Find((x)=>x.RelationObjectIds[0]==child2ObjectId);
            Assert.NotNull(first);
            Assert.True(second.HasBaseObjectId);
            Assert.True(second.BaseObjectId.Equals(parentObjectId));
            Assert.True(second.RelationObjectIds.Count.Equals(1));
            Assert.True(second.RelationObjectIds[0].Equals(child2ObjectId));
        }
        [Fact]
        public void ResolveTree_ComplexFiltering_AND_OR()
        {
            var parentObjectId= Guid.NewGuid().ToString(); 
            var parent2ObjectId= Guid.NewGuid().ToString(); 
            var childObjectId = Guid.NewGuid().ToString();                
            var child2ObjectId = Guid.NewGuid().ToString();                
            var child3ObjectId = Guid.NewGuid().ToString();                
            var parentSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parentObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = parent2ObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = childObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child2ObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = child3ObjectId,
                data = _fixture.Create<Car>(),
            });         

            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parentObjectId,                  
                };
            newEdge.ChildObjectIds.Add(childObjectId);
            newEdge.ChildObjectIds.Add(child2ObjectId);
            newEdge.ChildObjectIds.Add(child3ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var newEdge2 = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = parent2ObjectId,                  
                };
            newEdge2.ChildObjectIds.Add(childObjectId);
            newEdge2.ChildObjectIds.Add(child2ObjectId);
            newEdge2.ChildObjectIds.Add(child3ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge2 });    

            var relations = new List<Relation>();
            relations.Add(new Relation(){
                GlobalId = relation.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            });

            var simpleFilterAParent = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InParentObjIds}
                };
            simpleFilterAParent.Ids.Add(parentObjectId);
            var filterAParent = new EdgeRegistry.Filter{
                Simple = simpleFilterAParent
            };

            var simpleFilterAChild = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilterAChild.Ids.Add(childObjectId);
            var filterAChild = new EdgeRegistry.Filter{
                Simple = simpleFilterAChild
            };
            var complexFilterA = new EdgeRegistry.ComplexFilter{
                Operator = new LogicOperator(){
                    Operator = LogicOperator.Types.Operator.And
                }
            };
            complexFilterA.Operands.Add(filterAParent);
            complexFilterA.Operands.Add(filterAChild);

            var simpleFilterBParent = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InParentObjIds}
                };
            simpleFilterBParent.Ids.Add(parent2ObjectId);
            var filterBParent = new EdgeRegistry.Filter{
                Simple = simpleFilterBParent
            };

            var simpleFilterBChild = new EdgeRegistry.SimpleFilter{
                  Relation = 1,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilterBChild.Ids.Add(child2ObjectId);
            var filterBChild = new EdgeRegistry.Filter{
                Simple = simpleFilterBChild
            };
            var complexFilterB = new EdgeRegistry.ComplexFilter{
                Operator = new LogicOperator(){
                    Operator = LogicOperator.Types.Operator.And
                }
            };
            complexFilterB.Operands.Add(filterBParent);
            complexFilterB.Operands.Add(filterBChild);


            var complexFilter = new EdgeRegistry.ComplexFilter{
                Operator = new LogicOperator(){
                    Operator = LogicOperator.Types.Operator.Or
                }
            };
            complexFilter.Operands.Add(new EdgeRegistry.Filter{
                Complex = complexFilterA
            });
            complexFilter.Operands.Add(new EdgeRegistry.Filter{
                Complex = complexFilterB
            });

            var filter = new EdgeRegistry.Filter{
                Complex = complexFilter
            };
            var tree = _edgeRegistryService.ResolveTree(relations, filter).Result;

            Assert.Equal(2,tree.Rows.Count);
            var rows =  new List<RelationTreeRow>(tree.Rows);
            var first = rows.Find((x)=>x.RelationObjectIds[0]==childObjectId);
            Assert.NotNull(first);
            Assert.True(first.HasBaseObjectId);
            Assert.True(first.BaseObjectId.Equals(parentObjectId));
            Assert.True(first.RelationObjectIds.Count.Equals(1));
            Assert.True(first.RelationObjectIds[0].Equals(childObjectId));
            var second = rows.Find((x)=>x.RelationObjectIds[0]==child2ObjectId);
            Assert.NotNull(second);
            Assert.True(second.HasBaseObjectId);
            Assert.True(second.BaseObjectId.Equals(parent2ObjectId));
            Assert.True(second.RelationObjectIds.Count.Equals(1));
            Assert.True(second.RelationObjectIds[0].Equals(child2ObjectId));
        }

        [Fact]
        public void ResolveTree_FiltersOnSubrelations()
        {

            var ownerObjectId= Guid.NewGuid().ToString(); 
            var carObjectId= Guid.NewGuid().ToString(); 
            var tenantObjectId = Guid.NewGuid().ToString();                
            var tenant2ObjectId = Guid.NewGuid().ToString();                
            var ownerSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var carSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;                
            var tenantSchema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var relationOwner = _edgeRegistryService.AddRelation(carSchema.Id_, ownerSchema.Id_).Result;
            var relationTenant = _edgeRegistryService.AddRelation(tenantSchema.Id_, carSchema.Id_).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = ownerSchema.Id_,
                objectId = ownerObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = carSchema.Id_,
                objectId = carObjectId,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = tenantSchema.Id_,
                objectId = tenantObjectId,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = tenantSchema.Id_,
                objectId = tenant2ObjectId,
                data = _fixture.Create<Person>(),
            });


            var newEdgeOwner = new Edge()
                {
                    RelationId = relationOwner.RelationId_,
                    ParentObjectId = ownerObjectId,                  
                };
            newEdgeOwner.ChildObjectIds.Add(carObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdgeOwner });    

            var newEdgeTenant = new Edge()
                {
                    RelationId = relationTenant.RelationId_,
                    ParentObjectId = carObjectId,                  
                };
            newEdgeTenant.ChildObjectIds.Add(tenantObjectId);
            newEdgeTenant.ChildObjectIds.Add(tenant2ObjectId);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdgeTenant });    

            var relations = new List<Relation>();
            var ownerRelation = new Relation(){
                GlobalId = relationOwner.RelationId_,
                LocalId = 1,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                },
            };
            ownerRelation.Relations.Add(new Relation(){
                GlobalId = relationTenant.RelationId_,
                LocalId = 2,
                SearchFor = new SearchFor(){
                    SearchFor_ = SearchFor.Types.Direction.Children
                }

            });
            relations.Add(ownerRelation);
            
            var simpleFilter = new EdgeRegistry.SimpleFilter{
                  Relation = 2,
                  Side =  new SimpleFilterSide(){Side = Side.InChildObjIds}
                };
            simpleFilter.Ids.Add(tenantObjectId);
            var filter = new EdgeRegistry.Filter{
                Simple = simpleFilter
            };

            var tree = _edgeRegistryService.ResolveTree(relations,filter).Result;

            Assert.Equal(1,tree.Rows.Count);
            var treeObject = tree.Rows[0];
            Assert.True(treeObject.HasBaseObjectId);
            Assert.Equal(ownerObjectId,treeObject.BaseObjectId);
            Assert.Equal(2,treeObject.RelationObjectIds.Count);
            Assert.Equal(carObjectId, treeObject.RelationObjectIds[0]);
            Assert.Equal(tenantObjectId,treeObject.RelationObjectIds[1]);
        }
        [Fact]
        public void GetEdge(){
            var objectIdForParentSchema= Guid.NewGuid().ToString(); 
            var objectIdForChildSchema = Guid.NewGuid().ToString();                
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(
                parentSchemaName, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                childSchemaName, 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = objectIdForParentSchema,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = objectIdForChildSchema,
                data = _fixture.Create<Car>(),
            });       

            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = objectIdForParentSchema,                  
                };
            newEdge.ChildObjectIds.Add(objectIdForChildSchema);
            _edgeRegistryService.AddEdges(new List<Edge>(){ newEdge });    

            var edgeFromService = _edgeRegistryService.GetEdge(objectIdForParentSchema, relation.RelationId_).Result;

            Assert.True(edgeFromService.HasRelationId);
            Assert.True(edgeFromService.HasParentObjectId);
            Assert.True(edgeFromService.RelationId.Equals(relation.RelationId_));
            Assert.True(edgeFromService.ParentObjectId.Equals(objectIdForParentSchema));
            Assert.True(edgeFromService.ChildObjectIds.Count == 1);
            Assert.True(edgeFromService.ChildObjectIds[0].Equals(objectIdForChildSchema));
        }

        [Fact]
        public void GetEdges(){
            var objectIdParent1 = Guid.NewGuid().ToString(); 
            var objectIdChild1 = Guid.NewGuid().ToString(); 
            var objectIdChild2 = Guid.NewGuid().ToString(); 
            var parent1 = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var child1 = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var child2 = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parent1.Id_,
                objectId = objectIdParent1,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = child1.Id_,
                objectId = objectIdChild1,
                data = _fixture.Create<Car>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = child2.Id_,
                objectId = objectIdChild2,
                data = _fixture.Create<Car>(),
            });   

            Thread.Sleep(1000);

            var relation1 = _edgeRegistryService.AddRelation(child1.Id_, parent1.Id_).Result;
            var relation2 = _edgeRegistryService.AddRelation(child2.Id_, parent1.Id_).Result;
            var edge1 = new Edge()
                {
                    RelationId = relation1.RelationId_,
                    ParentObjectId = objectIdParent1,                  
                };
            edge1.ChildObjectIds.Add(objectIdChild1);
            var edge2 = new Edge()
                {
                    RelationId = relation2.RelationId_,
                    ParentObjectId = objectIdParent1,                  
                };
            edge2.ChildObjectIds.Add(objectIdChild2);   
            _edgeRegistryService.AddEdges(new List<Edge>(){ 
                edge1,
                edge2
             });  
            Thread.Sleep(500);  
            var edgeFromService = _edgeRegistryService.GetEdges(objectIdParent1).Result;

            Assert.True(edgeFromService.Relations.Count == 2, "Not all relations were found");
            Assert.True(edgeFromService.Relations.IndexOf(edge1) > -1, 
                $"Relation {edge1.RelationId} not found in edge relations: {edgeFromService.Relations}");
            Assert.True(edgeFromService.Relations.IndexOf(edge2) > -1, 
                $"Relation {edge2.RelationId} not found in edge relations: {edgeFromService.Relations}"); 
        }        

        [Fact]
        public void GetSchemaRelations()
        {          
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(
                parentSchemaName, 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var childSchema = _schemaRegistryService.AddSchema(
                childSchemaName, 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;       
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var schemaRelations = _edgeRegistryService.GetSchemaRelations(parentSchema.Id_).Result;

            Assert.True(schemaRelations.Items.Count == 1);
            var item = schemaRelations.Items[0];

            Assert.Equal(parentSchema.Id_, item.ParentSchemaId);
            Assert.True(item.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, item.ChildSchemaId);
        }

        [Fact]
        public void ValidateRelation_EdgeWithRelation_NoValidationErrors()
        {
            var parent1 = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Person>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;
            var child1 = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<Car>().ToJSONString(), 
                SchemaType.Types.Type.DocumentStorage).Result;                    

            var relation = _edgeRegistryService.AddRelation(child1.Id_, parent1.Id_).Result;
            var results = _edgeRegistryService.ValidateRelation(relation.RelationId_).Result;

            Assert.IsType<Empty>(results);
        }

        [Fact]
        public async void ValidateRelation_NoRelation_ValidationErrors()
        {
            var exception = await Assert.ThrowsAsync<RpcException>(() => _edgeRegistryService.ValidateRelation("1d1cc7a5-9277-48bc-97d3-3d99cfb63002"));
            Assert.Contains("InvalidArgument", exception.Message);
            Assert.Contains("1d1cc7a5-9277-48bc-97d3-3d99cfb63002 does not exist", exception.Message);
        }



        [Fact]
        public void Heartbeat()
        {
            var results = _edgeRegistryService.Heartbeat().Result;
            Assert.IsType<Empty>(results);
        }

    }
}