using System;
using System.Collections.Generic;
using Nest;

namespace CDL.Tests.Utils
{
    public static class ElasticsearchConnector
    {
        public static IReadOnlyCollection<TDocument> QueryAll<TDocument>(Uri nodeUrl, string index) where TDocument : class
        {
            var settings = new ConnectionSettings(nodeUrl);
            var client = new ElasticClient(settings);

            var res = client.Search<TDocument>(s => s.Index(index)).Documents;

            return res;
        }
    }
}