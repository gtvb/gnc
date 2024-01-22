#![allow(unused, dead_code)]

use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::prelude::*;
use std::net::TcpStream;

use crate::event::Event;

pub struct Connection {
    subscription_id: Option<String>,

    relay_url: String,
    conn: TcpStream,
}

struct SubscribeRequest<'a> {
    subscription_id: &'a str,
    filters: Vec<Filter>,
}

impl serde::Serialize for SubscribeRequest<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2 + self.filters.len()))?;

        seq.serialize_element("REQ");
        seq.serialize_element(&self.subscription_id);
        for filter in &self.filters {
            seq.serialize_element(filter);
        }

        seq.end()
    }
}

impl SubscribeRequest<'_> {
    fn new(subscription_id: &str, filters: Vec<Filter>) -> SubscribeRequest {
        SubscribeRequest {
            subscription_id,
            filters
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Filter {
    ids: Vec<String>,
    authors: Vec<String>,
    kinds: Vec<u16>,
    tag_values: Vec<String>,
    since: u64,
    until: u64,
    limit: u64,
}

impl Connection {
    fn new(relay_url: &str) -> Connection {
        let conn = TcpStream::connect(relay_url).expect("could not connect to this address");

        Connection {
            subscription_id: Some(generate_subscription_id(64)),
            relay_url: relay_url.into(),
            conn,
        }
    }

    fn publish(&mut self, event: Event) -> Result<()> {
        let message = json!(["EVENT".to_string(), event]).to_string();

        self.conn.write_all(message.as_bytes())?;

        Ok(())
    }

    fn request(&mut self, filters: Vec<Filter>) -> Result<()> {
        let s_id = self.subscription_id.clone().unwrap();
        let req = SubscribeRequest::new(&s_id, filters);
        let req_str = serde_json::to_string(&req).unwrap();

        self.conn.write_all(req_str.as_bytes())?;

        Ok(())
    }

    fn close_subscription(&mut self) -> Result<()> {
        let message = json!(["CLOSE".to_string(), self.subscription_id]).to_string();

        self.conn.write_all(message.as_bytes())?;
        self.subscription_id = None;

        Ok(())
    }
}

fn generate_subscription_id(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
