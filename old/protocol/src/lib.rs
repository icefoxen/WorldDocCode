extern crate ring;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rustc_serialize;
extern crate untrusted;
extern crate base64;
extern crate chrono;

use ring::{signature};


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ValidationError {
    UnknownUser(String),
    MalformedSignature,
    InvalidSignature,
}

use chrono::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, RustcDecodable, RustcEncodable, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub user: String,
    pub utc: DateTime<Utc>,
    pub signature: String,
    pub new_contents: String,
}

impl UpdateMessage {
    pub fn signed_message(keypair: &signature::Ed25519KeyPair, user: &str, msg: &str) -> UpdateMessage {
        // TODO: Sign the digest of the contents rather than the contents itself,
        // apparently: https://en.wikipedia.org/wiki/Digital_signature#How_they_work
        // Also include timestamp and target server in the signature to prevent replay attacks.
        let aggregated_message = String::from(user) + " " + msg;
        let message_bytes = aggregated_message.as_bytes();
        let sig = keypair.sign(message_bytes);
        let base64_sig = base64::encode(sig.as_ref());
        UpdateMessage {
            user: user.to_string(),
            utc: Utc::now(),
            signature: base64_sig,
            new_contents: msg.to_string(),
        }
    }

    pub fn verify_signature(&self, pubkey_bytes: &[u8]) -> Result<(), ValidationError> {
        let aggregated_message = String::from(self.user.as_str()) + " " + self.new_contents.as_ref();
        let message_bytes = aggregated_message.as_bytes();
        let sig_bytes = base64::decode(&self.signature)
            .map_err(|_decode_error| ValidationError::MalformedSignature)?;
        let pubkey = untrusted::Input::from(pubkey_bytes);
        let msg = untrusted::Input::from(message_bytes);
        let sig = untrusted::Input::from(&sig_bytes);
        signature::verify(&signature::ED25519, pubkey, msg, sig)
            .map_err(|_err| ValidationError::InvalidSignature)
    }
}