using MassTransit.Futures.Contracts;

namespace CDL.Tests.Configuration
{
    public class ConfigurationOptions
    {
        public string CDL_EDGE_REGISTRY_ADDRESS { get; set; } 
        public string CDL_MATERIALIZER_GENERAL_POSTGRESQL_ADDRESS { get; set; } 
        public string CDL_MATERIALIZER_GENERAL_ELASTICSEARCH_ADDRESS { get; set; } 
        public string CDL_MATERIALIZER_ONDEMAND_ADDRESS { get; set; } 
        public string CDL_QUERY_ROUTER_ADDRESS { get; set; } 
        public string CDL_QUERY_SERVICE_ADDRESS { get; set; } 
        public string CDL_SCHEMA_REGISTRY_ADDRESS { get; set; }  
        public string CDL_KAFKA_BROKER { get; set; }
        public string CDL_KAFKA_DATA_INPUT_TOPIC { get; set; }
        public string CDL_KAFKA_EDGE_INPUT_TOPIC { get; set; }
        public string CDL_SCHEMA_REGISTRY_DESTINATION { get; set; }
        public string CDL_ELASTICSEARCH_NODE { get; set; }
    }
}