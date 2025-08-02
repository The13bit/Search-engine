use std::{f32::consts::PI, fmt};

use mongodb::{bson::{self, oid::ObjectId}, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Words {
    pub _id: ObjectId,
    pub document: ObjectId,
    pub word: String,
    pub count: i32,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub _id: ObjectId,
    pub url: String,
    pub title: String,
    pub description: String,
    pub canonical_url: String,
    pub summary_text: String,
    pub full_text: Vec<String>,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct IdfCount {
    pub _id: ObjectId,
    pub word: String,
    pub url: String,
    pub weight: f64,
}

impl IdfCount {
    pub fn new(word: String, url: String, weight: f64) -> Self {
        IdfCount {
            _id: ObjectId::new(),
            word,
            url,
            weight,
        }
    }

    
}

