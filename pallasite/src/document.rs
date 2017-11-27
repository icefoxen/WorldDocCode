use chrono::prelude::*;
use cid::Cid;

use identity::Identity;

#[derive(Clone, PartialEq, Eq)]
pub struct Document {
    contents: Vec<Part>,
    character_encoding: Encoding,
    title: Option<String>,
    date: Option<DateTime<Utc>>, // Must be in UTC
    local_date: Option<FixedOffset>, // Offset of author's timezone from UTC
    author: Option<String>,
    author_id: Option<Identity>,
    previous_revisions: Option<Vec<Cid>>,
    subject: Option<String>, // Like an email subject... is this the same as "title"?
    // categories/tags (with vocabularies?)
    in_response_to: Option<Cid>,
    reply_to: Option<Identity>,
    language: Option<String>,
}

/// Possible character encodings.
/// Just UTF-8 for now.
#[derive(Clone, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Part {
    Body(Vec<Segment>),
    Section {
        level: u32, 
        contents: Vec<Segment>
    },
}

/// Describe structure of the contained text
#[derive(Clone, PartialEq, Eq)]
pub enum Segment {
    Para(Elements),
    Abstract(Elements),
    Table {
        header: Vec<Elements>,
        body: Vec<Vec<Segment>>,
        footer: Vec<Elements>,
    },
    Figure {
        caption: Vec<Elements>,
        source: Cid, // May get fancier someday
    },
    List {
        type_: ListType,
        elements: Vec<Segment>,
    },
    Code {
        language: Option<String>,
        contents: String,
    },
    Quote(Box<Segment>),
}

#[derive(Clone, PartialEq, Eq)]
pub enum ListType {
    Bulleted,
    Numbered,
}

// #[derive(Clone, PartialEq, Eq)]
pub type Elements = Vec<Element>;

/// Describe properties of the contained text
#[derive(Clone, PartialEq, Eq)]
pub enum Element {
    Text(String),
    Strong(Elements),
    Emphasized(Elements),
    Footnote(Elements),
    Xref {
        contents: Elements, 
        target: Cid,
    },
    Subscript(Elements),
    Superscript(Elements),
    Insertion(Elements),
    Deletion(Elements),
    Preformatted(Elements),
    Comment(String),
    Anchor(String),
}



/*
struct UpdateRequest {
    username: Identity,
    timestamp: DateTime,
    target: Location,
    new_target: CID,
    signature: Signature,
}
*/