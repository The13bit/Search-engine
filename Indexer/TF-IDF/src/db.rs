use futures::{stream::StreamExt, TryStreamExt};
use mongodb::{
    action::Find,
    bson::doc,
    error::Error,
    options::FindOptions,
    results::{InsertManyResult, InsertOneResult},
    Client, Collection, Cursor,
};

use crate::models::{Document, DocumentMetadata, TfIdfScore, Words};
pub struct Database {
    words: Collection<Words>,
    documents: Collection<Document>,
    tf_idf_scores: Collection<TfIdfScore>,
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
        let tf_idf_scores: Collection<TfIdfScore> = db.collection("tf_idf_scores");

        Database {
            words,
            documents,
            tf_idf_scores,
        }
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
        let result = self.documents.delete_one(doc! { "url": url }).await;

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
            Ok(_) => {}
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
            Ok(_) => {}
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

    pub async fn insert_tf_idf_scores(
        &self,
        scores: Vec<TfIdfScore>,
    ) -> Result<InsertManyResult, Error> {
        let result = match self.tf_idf_scores.insert_many(scores).await {
            Ok(res) => {
                //println!("TF-IDF scores insertion successful");
                res
            }
            Err(e) => {
                panic!("TF-IDF scores insertion failed: {}", e);
            }
        };
        Ok(result)
    }

    pub async fn get_all_documents(&self) -> Result<Vec<Document>, Error> {
        let mut cursor = self.documents.find(doc! {}).await?;
        let mut documents = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => documents.push(document),
                Err(e) => {
                    println!("Error reading document: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(documents)
    }

    pub async fn get_all_words(&self) -> Result<Vec<Words>, Error> {
        let mut cursor = self.words.find(doc! {}).await?;
        let mut words = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(word) => words.push(word),
                Err(e) => {
                    println!("Error reading words: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(words)
    }

    pub async fn delete_tf_idf_scores(&self) -> Result<(), Error> {
        match self.tf_idf_scores.delete_many(doc! {}).await {
            Ok(_) => {
                println!("All TF-IDF scores deleted successfully");
                Ok(())
            }
            Err(e) => {
                println!("Error deleting TF-IDF scores: {}", e);
                Err(e)
            }
        }
    }
    // In your db.rs file, add a method like:
    pub async fn get_documents_metadata(
        &self,
    ) -> Result<Vec<DocumentMetadata>, mongodb::error::Error> {
        let projection = doc! {
            "_id": 1,
            "url": 1,
            "title": 1,
            "description": 1,
           
        };

        let mut cursor = self.documents.clone_with_type::<DocumentMetadata>().find(doc! {}).projection(projection).await?;

        let metadata: Vec<DocumentMetadata> = cursor.try_collect().await?;
        
        Ok(metadata)
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
        .await
        .unwrap();
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
