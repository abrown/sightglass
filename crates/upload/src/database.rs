use std::{collections::HashMap, slice::SliceIndex};

use anyhow::Result;
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::Value;
pub struct Database {
    url: String,
    dryrun: bool,
}

impl Database {
    pub fn new(url: String, dryrun: bool) -> Self {
        Self {
            url: url.trim_end_matches("/").to_string(),
            dryrun,
        }
    }

    /// Use the ElasticSearch [Get
    /// API](https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-get.html) to
    /// verify if a document exists in `index` with the given `id`.
    pub fn exists(&self, index: &str, id: &str) -> Result<bool> {
        let url = format!("{}/{}/_doc/{}", self.url, index, id);
        if self.dryrun {
            Ok(true)
        } else {
            let client = Client::new();
            let response = client.head(url).send()?;
            Ok(response.status().is_success())
        }
    }

    /// Use the ElasticSearch [Index
    /// API](https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-index_.html) to
    /// add a document to `index`, optionally with the given `id`.
    pub fn create<T>(&self, index: &str, object: &T, id: Option<&str>) -> Result<String>
    where
        T: Serialize,
    {
        let url = if let Some(id) = id {
            format!("{}/{}/_doc/{}", self.url, index, id)
        } else {
            format!("{}/{}/_doc", self.url, index)
        };

        let body = serde_json::to_string(object)?;

        if self.dryrun {
            Ok(if let Some(id) = id {
                id.to_string()
            } else {
                "dryrun-id".to_string()
            })
        } else {
            let client = Client::new();
            let response = client.post(url).body(body).send()?;
            let response: HashMap<String, Value> = serde_json::from_slice(&response.bytes()?)?;
            let id = response.get("_id").unwrap().as_str().unwrap().to_string();
            Ok(id)
        }
    }
}
