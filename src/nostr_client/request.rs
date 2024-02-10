use super::event::Event as EventType;
use serde::{ser::SerializeSeq, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    ids: Vec<String>,
    authors: Vec<String>,
    kinds: Vec<u64>,
    tag_values: Vec<String>,
    since: u64,
    until: u64,
    limit: u64,
}

impl Filter {
    pub fn new(
        ids: Vec<String>,
        authors: Vec<String>,
        kinds: Vec<u64>,
        tag_values: Vec<String>,
        since: u64,
        until: u64,
        limit: u64,
    ) -> Self {
        Self {
            ids,
            authors,
            kinds,
            tag_values,
            since,
            until,
            limit,
        }
    }
}

pub enum Request {
    Event {
        event: EventType,
    },
    Req {
        subscription_id: String,
        filters: Vec<Filter>,
    },
    Close {
        subscription_id: String,
    },
}

/// The protocol specifies that the requests are formatted in a little bit 
/// of a different fashion, by using JSON arrays, so we implement the 
/// `Serialize` trait instead od deriving it on the struct directly
impl serde::Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Request::Event { event } => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                // let event_str = serde_json::to_string(&event).unwrap();

                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;

                seq.end()
            }
            Request::Req {
                subscription_id,
                filters,
            } => {
                let mut seq = serializer.serialize_seq(Some(2 + filters.len()))?;

                seq.serialize_element("REQ")?;
                seq.serialize_element(subscription_id)?;
                for filter in filters {
                    seq.serialize_element(filter)?;
                }

                seq.end()
            }
            Request::Close { subscription_id } => {
                let mut seq = serializer.serialize_seq(Some(2))?;

                seq.serialize_element("CLOSE")?;
                seq.serialize_element(subscription_id)?;

                seq.end()
            }
        }
    }
}
