use mongodb::{
    bson::doc,
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
    pub async fn delete_documents(&self, url: &str) -> Result<(), Error> {
        let result = self
            .documents
            .delete_one(doc! { "url": url })
            .await;

        match result {
            Ok(res) => {
                println!("Document deleted successfully");
                Ok(())
            }
            Err(e) => {
                println!("Error deleting document: {}", e);
                Err(e)
            }
        }
    }
    pub async fn try_commit(&self, documents: Document, words: Vec<Words>) -> bool {
        let mut document_session = self.documents.client().start_session().await.unwrap();
        let mut word_session = self.words.client().start_session().await.unwrap();
        let url = documents.url.clone();
        document_session.start_transaction().await;
        word_session.start_transaction().await;

        match self
            .documents
            .insert_one(documents)
            .session(&mut document_session)
            .await
        {
            Ok(_) => {},
            Err(e) => {
                document_session.abort_transaction().await;
                return false;
            }
        }
        match self
            .words
            .insert_many(words)
            .session(&mut word_session)
            .await
        {
            Ok(_) => {},
            Err(e) => {
                word_session.abort_transaction().await;
                return false;
            }
        }

        match (
            document_session.commit_transaction().await,
            word_session.commit_transaction().await,
        ) {
            (Ok(_), Ok(_)) => {
                //println!("Both transactions committed successfully");
                true
            }
            (Err(e), _) => {
                println!("Document transaction failed: {} for: {}", e, url);

                false
            }
            (_, Err(e)) => {
                println!("Word transaction failed: {} for: {}", e, url);

                false
            }
        }
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
    pub async fn url_exists(&self, url: &str) -> Result<bool, Error> {
        let count = match self.documents.count_documents(doc! { "url": url }).await {
            Ok(count) => count,
            Err(e) => {
                println!("Error checking URL existence: {}", e);
                return Err(e);
            }
        };
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_check_url() {
        let db = Database::new().await;
        db.insert_document(Document {
            url: "https://docs.rs/url/latest/url/".to_string(),
            ..Document::default()
        })
        .await.unwrap();
        let url = "https://docs.rs/url/latest/url/";
        let exists = db.url_exists(url).await.unwrap();
        assert!(exists);
        db.delete_documents(url).await.unwrap();
    }
    #[tokio::test]
    async fn test_check_url_dne() {
        let db = Database::new().await;
        let url = "asdsda";
        let exists = db.url_exists(url).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_transaction_abort() {
        let db = Database::new().await;

        // Create example document and words
        let test_document = Document {
            url: "https://test-abort-transaction.com".to_string(),
            ..Document::default()
        };

        let test_words = vec![
            Words {
                word: "test".to_string(),
                document: test_document.get_id(),
                ..Words::default()
            },
            Words {
                word: "abort".to_string(),
                document: test_document.get_id(),
                ..Words::default()
            },
        ];

        // Verify the URL doesn't exist before the test
        let exists_before = db.url_exists(&test_document.url).await.unwrap();
        assert!(
            !exists_before,
            "Test URL should not exist before transaction"
        );

        // Start transaction and insert data
        let mut document_session = db.documents.client().start_session().await.unwrap();
        let mut word_session = db.words.client().start_session().await.unwrap();

        document_session.start_transaction().await.unwrap();
        word_session.start_transaction().await.unwrap();

        // Insert document and words within transaction
        let _doc_result = db
            .documents
            .insert_one(&test_document)
            .session(&mut document_session)
            .await;

        let _word_result = db
            .words
            .insert_many(&test_words)
            .session(&mut word_session)
            .await;

        // Abort both transactions
        document_session.abort_transaction().await.unwrap();
        word_session.abort_transaction().await.unwrap();

        // Verify the data was not committed (URL should still not exist)
        let exists_after = db.url_exists(&test_document.url).await.unwrap();
        assert!(
            !exists_after,
            "Test URL should not exist after transaction abort"
        );

        println!("Transaction abort test completed successfully");
    }
}
