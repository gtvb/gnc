#![allow(unused, dead_code)]

use std::str::FromStr;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;

use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::event::Event as EventType;
use super::request::{Filter, Request};
use super::response::Response;

pub struct Connection {
    subscription_id: String,

    relay_url: String,
    conn: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Connection {
    pub async fn new(relay_url: &str) -> Result<Connection> {
        let (mut conn, _) = connect_async(relay_url).await?;

        Ok(Connection {
            subscription_id: generate_subscription_id(64),
            relay_url: relay_url.to_string(),
            conn,
        })
    }

    pub async fn publish_event(&mut self, event: EventType) -> Result<Response> {
        let event_json = Request::Event { event };
        let event_str = serde_json::to_string(&event_json).unwrap();

        self.conn.send(Message::Text(event_str)).await?;

        let response = self.conn.next().await.unwrap().unwrap().to_string();
        let response = Response::from_str(&response).unwrap();

        println!("Got response: {:?}", response);

        Ok(response)
    }

    pub async fn subscribe(&mut self, filters: Vec<Filter>) -> Result<()> {
        let request = Request::Req {
            subscription_id: self.subscription_id.clone(),
            filters,
        };
        let request_str = serde_json::to_string(&request).unwrap();

        self.conn.send(Message::Text(request_str)).await?;

        loop {
            let response = match self.conn.next().await {
                Some(Ok(data)) => data.to_string(),
                Some(_) => {
                    println!("Ignoring error variant...");
                    continue;
                }
                None => break,
            };

            let response = Response::from_str(&response).unwrap();
            println!("Got: {:?}", response);
        }

        Ok(())
    }

    pub async fn close(&mut self, event: EventType) -> Result<Response> {
        let event_json = Request::Event { event };
        let event_str = serde_json::to_string(&event_json).unwrap();
        println!("Sending event: {:?}", event_str);

        self.conn.send(Message::Text(event_str)).await?;

        let response = self.conn.next().await.unwrap().unwrap().to_string();
        let response = Response::from_str(&response).unwrap();

        Ok(response)
    }
}

fn generate_subscription_id(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
