use std::collections::HashMap;
use mongodb::bson::oid::ObjectId;

mod db;
mod models;

use db::Database;
use models::{Document, Words, TfIdfScore};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    println!("Starting TF-IDF computation...");
    
    let database = Database::new().await;
    
    match compute_tf_idf(&database).await {
        Ok(_) => println!("TF-IDF computation completed successfully!"),
        Err(e) => println!("Error during TF-IDF computation: {}", e),
    }
}

async fn compute_tf_idf(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    // Clear existing TF-IDF scores
    db.delete_tf_idf_scores().await?;
    
    // Get all documents and words from the database
    let documents = db.get_documents_metadata().await?;
    let words = db.get_all_words().await?;
    
    if documents.is_empty() {
        println!("No documents found in the database.");
        return Ok(());
    }
    
    if words.is_empty() {
        println!("No words found in the database.");
        return Ok(());
    }
    
    println!("Processing {} documents and {} words", documents.len(), words.len());
    
    // Build document frequency map (how many documents contain each word)
    let mut document_frequency: HashMap<String, usize> = HashMap::new();
    let mut document_word_counts: HashMap<ObjectId, HashMap<String, i32>> = HashMap::new();
    let mut document_urls: HashMap<ObjectId, String> = HashMap::new();
    
    // Store document URLs for later reference
    for doc in &documents {
        document_urls.insert(doc._id, doc.url.clone());
    }
    
    // Group words by document and count document frequency
    for word in &words {
        // Count how many times each word appears in each document
        document_word_counts
            .entry(word.document)
            .or_insert_with(HashMap::new)
            .insert(word.word.clone(), word.count);
        
        // Count in how many documents each word appears
        *document_frequency.entry(word.word.clone()).or_insert(0) += 1;
    }
    
    let total_documents = documents.len() as f64;
    let mut tf_idf_scores = Vec::new();
    
    // Compute TF-IDF for each word in each document
    for (doc_id, word_counts) in document_word_counts {
        let url = match document_urls.get(&doc_id) {
            Some(url) => url.clone(),
            None => {
                println!("Warning: Document ID {} not found, skipping words for this document", doc_id);
                continue;
            }
        };
        
        // Calculate total words in this document
        let total_words_in_doc: i32 = word_counts.values().sum();
        
        for (word, count) in word_counts {
            // Calculate Term Frequency (TF)
            let tf = (count as f64) / (total_words_in_doc as f64);
            
            // Calculate Inverse Document Frequency (IDF)
            let df = *document_frequency.get(&word).unwrap_or(&1) as f64;
            let idf = (total_documents / df).ln() + 1.0;
            
            // Create TF-IDF score entry
            let tf_idf_score = TfIdfScore::new(word, doc_id, url.clone(), tf, idf);
            tf_idf_scores.push(tf_idf_score);
        }
    }
    
    println!("Computed {} TF-IDF scores", tf_idf_scores.len());
    
    // Insert TF-IDF scores in batches to avoid memory issues
    const BATCH_SIZE: usize = 1000;
    let mut processed = 0;
    
    for batch in tf_idf_scores.chunks(BATCH_SIZE) {
        db.insert_tf_idf_scores(batch.to_vec()).await?;
        processed += batch.len();
        print!("Processed {}/{} TF-IDF scores\r", processed, tf_idf_scores.len());
    }
    
    println!("TF-IDF computation and storage completed!");
    Ok(())
}
