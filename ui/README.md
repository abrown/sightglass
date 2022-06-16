# UI

This directory configures a UI for analyzing Sightglass data using the ELK stack (ElasticSearch, Logstash, Kibana; now renamed to the Elastic stack). Since building a UI from scratch is time-consuming work and doing so flexibly, allowing many types of visualizations and comparisons, is difficult, we simply provide Kibana, an industry-standard tool for data wrangling and visualization. To set up the ELK stack, we follow the official instructions: [Running the Elastic Stack on Docker](https://www.elastic.co/guide/en/elastic-stack-get-started/current/get-started-docker.html#run-stack-docker).

### Pre-requisites

This requires a working `docker-compose` installation.

### Run

```
docker-compose up -d
```

Navigate in your browser to http://localhost:5601 to start using Kibana; the raw Elasticsearch
endpoints are available at http://localhost:9200.

You may need to increase the `mmapfs` limits for ElasticSearch to initialize without error:

```
sysctl -w vm.max_map_count=262144
```

### Stop

```
docker-compose down
```

### TODO

 - Set up index patterns for `measurements`, `engines`, `machines`, `benchmarks`
 - Set up some initial charts
 - Set up initial mapping to link IDs to their stored record
