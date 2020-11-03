
# syntax=docker/dockerfile:experimental

FROM crystallang/crystal 
WORKDIR /usr/src/cdl/
COPY shard.yml shard.lock ./
RUN shards install
COPY sample_json/ sample_json/
COPY scripts/upload_to_rabbitmq.cr ./
RUN crystal build upload_to_rabbitmq.cr
CMD ["./upload_to_rabbitmq"]
