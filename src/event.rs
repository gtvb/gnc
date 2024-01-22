#![allow(unused, dead_code)]

use std::{
    str::{self, FromStr},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use serde_json::json;

use secp256k1::{hashes::sha256, PublicKey};
use secp256k1::{Message, Secp256k1, SecretKey};

#[derive(Serialize, Deserialize)]
struct UserData {
    name: String,
    about: String,
    picture: String
}

#[derive(Serialize, Deserialize)]
enum DataType {
    Str(String),
    User(UserData)
}

pub struct Category {
    kind: u16,
    data_type: DataType
}

pub struct Event {
    id: Option<String>,
    sig: Option<String>,
    pubkey: Option<String>,

    category: Category,
    created_at: u64,

    tags: Vec<Vec<String>>,
}

impl serde::Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(6))?;

        seq.serialize_element(&0);
        seq.serialize_element(&self.pubkey);
        seq.serialize_element(&self.created_at);
        seq.serialize_element(&self.category.kind);
        seq.serialize_element(&self.tags);
        seq.serialize_element(&self.category.data_type);

        seq.end()
    }
}

impl Event {
    pub fn new(category: Category) -> Event {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to obtain unix time")
            .as_secs();
        let tags = Vec::new();

        Event {
            id: None,
            sig: None,
            pubkey: None,
            category,
            created_at,
            tags
        }
    }

    // TODO: better error handling here 
    pub fn setup(&mut self, public_key: &str, private_key: &str) -> Result<()> {
        self.pubkey = Some(public_key.into());

        self.generate_id()?;
        self.sign(public_key, private_key)?;
        Ok(())
    }

    pub fn add_tag(&mut self, tag_key: &str, tag_value: &str, relay_url: Option<&str>) {
        if let Some(url) = relay_url {
            self.tags
                .push(vec![tag_key.into(), tag_value.into(), url.into()]);
            return;
        }

        self.tags.push(vec![tag_key.into(), tag_value.into()]);
    }

    fn generate_id(&mut self) -> Result<()> {
        let event_json = serde_json::to_string(&self)?;
        let id = Message::from_hashed_data::<sha256::Hash>(event_json.as_bytes());

        self.id = Some(id.to_string());

        Ok(())
    }

    fn sign(&mut self, public_key: &str, private_key: &str) -> Result<()> {
        if self.id.is_none() {
            return Err(anyhow!("cannot sign the event structure without an id"));
        }

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_str(private_key)?;
        let public_key = PublicKey::from_str(public_key)?;

        let id = self.id.clone().unwrap();
        let id_as_msg = Message::from_digest_slice(&id.as_bytes())?;
        let sig = secp.sign_ecdsa(&id_as_msg, &secret_key);

        // Is this necessary?
        // assert!(secp.verify_ecdsa(&id_as_msg, &sig, &public_key).is_ok());

        self.sig = Some(sig.to_string());

        Ok(())
    }
}
