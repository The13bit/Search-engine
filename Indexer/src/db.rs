use mongodb::{
    action::InsertOne,
    bson::{self, oid::ObjectId},
    error::Error,
    results::{InsertManyResult, InsertOneResult},
    Client, Collection,
};

use crate::models::{Document, Words};
pub struct Database {
    words: Collection<Words>,
    documents: Collection<Document>,
}

impl Database {
    pub async fn new() -> Database {
        let url = match dotenv::var("MONGODB_URI") {
            Ok(url) => url,
            Err(_) => "mongodb://localhost:27017".to_string(),
        };

        let client = match Client::with_uri_str(url).await {
            Ok(client) => {
                println!("DB connected succesfully");
                client
            }
            Err(e) => {
                panic!("{}", e);
            }
        };
        let db = client.database("SearchEngine");

        let words: Collection<Words> = db.collection("words");
        let documents: Collection<Document> = db.collection("documents");

        Database { words, documents }
    }
    pub async fn insert_words(&self, words: Vec<Words>) -> Result<InsertManyResult, Error> {
        let result = match self.words.insert_many(words).await {
            Ok(res) => {
                println!("Insertion Successful");
                res
            }
            Err(e) => {
                panic!("word Insertion failed:{}", e);
            }
        };

        Ok(result)
    }
    pub async fn insert_document(&self, document: Document) -> Result<InsertOneResult, Error> {
        let result = match self.documents.insert_one(document).await {
            Ok(res) => {
                println!("Insertion Successful");
                res
            }
            Err(e) => {
                panic!("Document Insertion failed:{}", e);
            }
        };
        Ok(result)
    }
    pub async fn insert_word(&self, word: Words) -> Result<InsertOneResult, Error> {
        let result = match self.words.insert_one(word).await {
            Ok(res) => {
                println!("Insertion Successful");
                res
            }
            Err(e) => {
                panic!("word Insertion failed:{}", e);
            }
        };
        Ok(result)
    }
}
