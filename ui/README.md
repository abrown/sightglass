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

### Configure

After the Docker containers are running but before ingesting any data, run:

```
config/configure.sh
```

This script that will configure the Kibana and ElasticSearch instances with helpful dashboards,
field mappings, etc. before ingesting the data. This step is not mandatory, but without it, the user
is responsible to set all of this up. Note that any changes to the Kibana visualizations can be
saved to the `config` directory by running `config/export-dashboards.sh`.

### Ingest data

See the `sightglass-cli upload` command for details on ingesting measurement and fingerprint data.

### Stop

```
docker-compose down
```

### Clean up

To remove all data stored in the system:

1. Stop the containers (see above).
2. Remove the containers (e.g., `docker rm -f $(docker ps -a -q)`, though this will remove all
   active containers).
3. Remove the volumes related to this project:

   ```
   docker volume rm $(docker volume ls -q | grep ui_data)
   ```
