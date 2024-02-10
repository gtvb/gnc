use std::str::FromStr;

use super::event::Event as EventType;
use serde_json::Value;

#[derive(Debug)]
pub enum Response {
    Event {
        subscription_id: String,
        event: EventType,
    },
    Ok {
        event_id: String,
        accepted: bool,
        message: String,
    },
    Eose {
        subscription_id: String,
    },
    Closed {
        subscription_id: String,
        message: String,
    },
    Notice {
        message: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseResponseError;

/// Implementing `FromStr` is easier than implementing the `Deserialize` trait,
/// since this is what we're going to receive from the connections.
impl FromStr for Response {
    type Err = ParseResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: Value = serde_json::from_str(s).unwrap();
        let mut array = val.as_array().unwrap().iter();

        match array.next().unwrap().as_str().unwrap() {
            "EVENT" => {
                let subscription_id = array.next().unwrap().as_str().unwrap().to_string();
                let event_obj = array.next().unwrap().as_object().unwrap();

                let event = EventType::new(
                    event_obj["pubkey"].as_str().unwrap(),
                    event_obj["kind"].as_u64().unwrap(),
                    event_obj["content"].to_string(),
                );

                Ok(Response::Event {
                    subscription_id,
                    event,
                })
            }
            "OK" => {
                let event_id = array.next().unwrap().as_str().unwrap().to_string();
                let accepted = array.next().unwrap().as_bool().unwrap();
                let message = array.next().unwrap().as_str().unwrap().to_string();

                Ok(Response::Ok {
                    event_id,
                    accepted,
                    message,
                })
            }
            "EOSE" => {
                let subscription_id = array.next().unwrap().as_str().unwrap().to_string();

                Ok(Response::Eose { subscription_id })
            }
            "CLOSED" => {
                let subscription_id = array.next().unwrap().as_str().unwrap().to_string();
                let message = array.next().unwrap().as_str().unwrap().to_string();

                Ok(Response::Closed {
                    subscription_id,
                    message,
                })
            }
            "NOTICE" => {
                let message = array.next().unwrap().as_str().unwrap().to_string();

                Ok(Response::Notice { message })
            }
            str => panic!("Invalid response string {}", str),
        }
    }
}
