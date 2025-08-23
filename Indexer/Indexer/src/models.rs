use std::fmt;

use mongodb::bson::{oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Words {
    pub _id: ObjectId,
    pub document: ObjectId,
    pub word: String,
    pub count: i32,
}
impl Default for Words {
    fn default() -> Self {
        Words {
            _id: ObjectId::new(),
            document: ObjectId::new(),
            word: "default".to_string(),
            count: 0,
        }
    }
    
}

impl Words {
    pub fn new(document: ObjectId, word: String, count: i32) -> Self {
        Words {
            _id: ObjectId::new(),
            document,
            word,
            count,
        }
    }
    

   
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Document {
    pub _id: ObjectId,
    pub url: String,
     pub title: String,
     pub description: String,
    pub canonical_url: String,
    pub summary_text: String,
     pub full_text: Vec<String>,
}

impl  Default for Document {
    fn default() -> Self {
        Document {
            _id: ObjectId::new(),
            url: "Testing Test ting 124".to_string(),
            title: "Testing Title".to_string(),
            description: "Testing Description".to_string(),
            canonical_url: "https://example.com/testing".to_string(),
            summary_text: "Testing Summary".to_string(),
            full_text: vec!["Testing full text".to_string()],
        }
    }
    
}
impl Document {
    pub fn new(url: String, title: String, description: String, canonical_url: String, summary_text: String, full_text: Vec<String>) -> Self {
        Document {
            _id: ObjectId::new(),
            url,
            title,
            description,
            canonical_url,
            summary_text,
            full_text,
        }
    }
     pub fn get_full_text(&self) -> &Vec<String> {
        &self.full_text
    }
    pub fn get_title(&self) -> String {
        self.title.clone()
    }
    pub fn get_description(&self) -> String {
        self.description.clone()
    }
    pub fn get_id(&self)->ObjectId{
        self._id
    }
    
}
impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Document {{")?;
        writeln!(f, "  title: {:?}", self.title)?;
        writeln!(f, "  description: {:?}", self.description)?;
        writeln!(f, "  canonical_url: {:?}", self.canonical_url)?;
        writeln!(f, "  summary_text: \"{}\"", self.summary_text)?;
        writeln!(f, "  full_text: {:?} words", self.full_text.len())?;
        write!(f, "}}")
    }
}

