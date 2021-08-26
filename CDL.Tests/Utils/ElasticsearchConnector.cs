using System;
using System.Collections.Generic;
using System.Linq;
using Nest;
using Newtonsoft.Json;

namespace CDL.Tests.Utils
{
    public static class ElasticsearchConnector
    {
        public static IReadOnlyCollection<TDocument> QueryAll<TDocument>(Uri nodeUrl, string index) where TDocument : class
        {
            var settings = new ConnectionSettings(nodeUrl);
            var client = new ElasticClient(settings);

            var res = client.Search<TDocument>(s => s.Index(index).MatchAll());
            
            throw new Exception($"{res.Documents.Count}; {JsonConvert.ToString(res.Total)}; {index}");

            return res.Documents;
        }
    }
}
