using System;
using System.Threading.Tasks;
using Polly;

namespace CDL.Tests.Utils
{
    public class RetryHelper
    {
        public static Task<T> TryFetch<T>(Func<T> request, Func<T, bool> validator, int retries = 10)
        {
            return Task.FromResult(Policy.HandleResult(validator).WaitAndRetry(retries, i => TimeSpan.FromSeconds(i)).Execute(request));
        }
    }
}