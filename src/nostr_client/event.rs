#![allow(unused, dead_code)]

use std::{
    str::{self, FromStr},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use serde_json::json;

use secp256k1::{
    hashes::{hex::DisplayHex, sha256},
    All, Message, Parity, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey,
};

#[derive(Debug)]
pub struct PubKeyWrapper {
    key: XOnlyPublicKey,
    parity: Parity,
}

impl serde::Serialize for PubKeyWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.key.serialize()))
    }
}

#[derive(Debug, Serialize)]
pub struct Event {
    id: Option<String>,
    sig: Option<String>,
    pubkey: PubKeyWrapper,

    pub kind: u64,
    pub content: String,
    pub created_at: u64,

    pub tags: Vec<Vec<String>>,
}

impl Event {
    pub fn new(pubkey: &str, kind: u64, content: String) -> Event {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to obtain unix time")
            .as_secs();
        let tags = Vec::new();
        let (key, parity) = PublicKey::from_str(pubkey).unwrap().x_only_public_key();

        Event {
            id: None,
            sig: None,
            pubkey: PubKeyWrapper { key, parity },
            kind,
            content,
            created_at,
            tags,
        }
    }

    /// Generates a new id for the event, and then signs it with the secret key
    pub fn setup(&mut self, secret_key: &str, secp: &Secp256k1<All>) -> Result<()> {
        self.generate_id()?;
        self.sign(secret_key, &secp)?;

        Ok(())
    }

    /// Allows the user to add a new tag to the event
    pub fn add_tag(&mut self, tag_key: &str, tag_value: &str, relay_url: Option<&str>) {
        if let Some(url) = relay_url {
            self.tags
                .push(vec![tag_key.into(), tag_value.into(), url.into()]);
            return;
        }

        self.tags.push(vec![tag_key.into(), tag_value.into()]);
    }

    /// Generates a 32-byte hex-encoded sha256 of the serialized event data, 
    /// then updates the event id
    fn generate_id(&mut self) -> Result<()> {
        let id = Message::from_hashed_data::<sha256::Hash>(self.serialize_for_id().as_bytes());
        self.id = Some(id.to_string());

        Ok(())
    }

    // Uses the secret key alongside the secp engine to correctly sign the 
    // event data, that is represented by the `id` field.
    fn sign(&mut self, secret_key: &str, secp: &Secp256k1<All>) -> Result<()> {
        if self.id.is_none() {
            return Err(anyhow!("cannot sign the event structure without an id"));
        }

        let id = self.id.clone().unwrap();
        let id_as_msg = Message::from_digest_slice(&hex::decode(id).unwrap())?;

        let secret_key = SecretKey::from_str(secret_key)?;
        let keypair = secret_key.keypair(&secp);
        let sig = secp.sign_schnorr(&id_as_msg, &keypair);

        self.sig = Some(sig.to_string());

        Ok(())
    }

    /// The protocol specifies this format for serializing the `Event` type 
    /// for id generation
    fn serialize_for_id(&self) -> String {
        json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ])
        .to_string()
    }
}
