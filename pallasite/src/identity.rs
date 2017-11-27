use std::time::Duration;
use chrono::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Pubkey {
    username: Identity,
    algorithm: Algorithm,
    public_key: Key,
    created: DateTime<Utc>,
    expires: Option<DateTime<Utc>>,
    ttl: Option<Duration>,
    // An optional signature, so you can sign a new key with the previous one.
    // TODO: Specify a bit better exactly what goes into this and how it is connected.
    // Do we want a specific reference to the previous key as well?
    signature: Option<Signature>,
}

/// A user identity, such as icefox@alopex.li
#[derive(Clone, PartialEq, Eq)]
pub struct Identity {
    username: String,
    authority: String,
}

/// A base64 encoded string of a key
#[derive(Clone, PartialEq, Eq)]
pub struct Key(String);

/// A base64 encoded signature for the message
#[derive(Clone, PartialEq, Eq)]
pub struct Signature(String);

#[derive(Clone, PartialEq, Eq)]
pub enum Algorithm {
    Ed25519,
    // Maybe others later
}

#[derive(Clone, PartialEq, Eq)]
pub struct PubkeyRequest {
    username: String,
    query: Option<Query>
}

#[derive(Clone, PartialEq, Eq)]
pub enum Query {
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
}


