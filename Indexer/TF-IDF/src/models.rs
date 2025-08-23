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
pub struct TfIdfScore {
    pub _id: ObjectId,
    pub word: String,
    pub document_id: ObjectId,
    pub url: String,
    pub tf: f64,        // Term frequency
    pub idf: f64,       // Inverse document frequency  
    pub tf_idf: f64,    // TF-IDF score
}

impl TfIdfScore {
    pub fn new(word: String, document_id: ObjectId, url: String, tf: f64, idf: f64) -> Self {
        let tf_idf = tf * idf;
        TfIdfScore {
            _id: ObjectId::new(),
            word,
            document_id,
            url,
            tf,
            idf,
            tf_idf,
        }
    }
}

impl Default for TfIdfScore {
    fn default() -> Self {
        TfIdfScore {
            _id: ObjectId::new(),
            word: "default".to_string(),
            document_id: ObjectId::new(),
            url: "default".to_string(),
            tf: 0.0,
            idf: 0.0,
            tf_idf: 0.0,
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

// ...existing code...

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentMetadata {
    pub _id: ObjectId,
    pub url: String,
    pub title: String,
    pub description: String,
    // Exclude large fields: canonical_url, summary_text, full_text
}

impl DocumentMetadata {
    pub fn new(_id: ObjectId, url: String, title: String, description: String) -> Self {
        DocumentMetadata {
            _id,
            url,
            title,
            description,
        }
    }
}

// ...existing code...

