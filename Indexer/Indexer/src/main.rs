mod db;
mod errors;
mod models;
mod utils;
use dotenv::dotenv;
use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    os::windows::process,
    path::Path,
    str,
    sync::Arc,
};

use utils::{is_binary_extension, is_text_content};

use tokio::sync::Semaphore;
use urlencoding::decode;

use db::Database;

use crate::{
    errors::StateEvents,
    utils::{create_frequency, extract_structured_data},
};
fn clean_html(body: String) -> String {
    let mut output = String::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("script", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("svg", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("style", |el| {
                    el.remove();
                    Ok(())
                }),
            ],
            ..Settings::new()
        },
        |out: &[u8]| output.extend(str::from_utf8(out)),
    );
    rewriter.write(body.as_bytes()).unwrap();
    rewriter.end().unwrap();
    //println!("{}",&output[..100]);
    output
}
fn extract_text(text: String) -> Vec<String> {
    let text = text.trim();
    if text.ends_with("</html>") {
        let mut extracted_text = Vec::new();
        rewrite_str(
            text,
            RewriteStrSettings {
                element_content_handlers: vec![text!("*", |t| {
                    let text_content = t.as_str().trim();
                    if !text_content.is_empty() {
                        extracted_text.push(text_content.to_string());
                    }
                    Ok(())
                })],
                ..RewriteStrSettings::new()
            },
        )
        .unwrap();
        extracted_text
    } else {
        text.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }
}

async fn get_data(client: &Client, url: &str) -> String {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let res = client.get(url).headers(headers).send().await;
    let body: String;
    match res {
        Ok(val) => {
            body = val.text().await.unwrap();
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    body
}

async fn check_content_type(client: &Client, url: &str) -> bool {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let head_response = client.head(url).headers(headers.clone()).send().await;
    match head_response {
        Ok(response) => {
            if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                let content_type_str = content_type.to_str().unwrap_or("").to_string();
                if !is_text_content(&content_type_str) {
                    println!(
                        "Skipping non-text content: {} (Content-Type: {})",
                        url, content_type_str
                    );
                    return false;
                }
            }

            // Check content length to avoid very large files
            if let Some(content_length) = response.headers().get("content-length") {
                if let Ok(length_str) = content_length.to_str() {
                    if let Ok(length) = length_str.parse::<u64>() {
                        const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
                        if length > MAX_SIZE {
                            println!("Skipping large file: {} ({} bytes)", url, length);
                            return false;
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to get HEAD response: {}", e);
            return false;
        }
    }
    true
}

async fn process(url: String, db: Arc<Database>) -> StateEvents {
    match db.url_exists(&url).await {
        Ok(exists) => {
            if exists {
                return StateEvents::UrlExists;
            }
        }
        Err(e) => {
            println!("Error checking URL existence: {}", e);
            return StateEvents::UrlError;
        }
    }
    let client = Client::new();

    if is_binary_extension(&url) || !check_content_type(&client, &url).await {
        return StateEvents::InvalidExtension;
    }

    let page = get_data(&client, &url).await;

    let documents = extract_structured_data(page, url);
    //println!("{}",output);
    let words = create_frequency(&documents);

    if db.try_commit(documents, words).await {
        return StateEvents::TransactionSuccess;
    }
    StateEvents::TransactionError
}
async fn process_url(url: String, db: Arc<Database>) {
    match process(decode(url.as_str()).expect("UTF-8").into_owned(), db).await {
        StateEvents::TransactionSuccess => {
            println!("Transaction successful");
        }
        StateEvents::InvalidExtension => {
            println!("Invalid extension or content type, skipping URL");
        }
        StateEvents::TransactionError => {
            println!("Transaction failed");
        }
        StateEvents::UrlExists => {
            println!("URL already exists");
        }
        StateEvents::UrlError => {
            println!("Error checking URL");
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), ()> {
    dotenv().ok().expect("Env error");
    let path = Path::new("../urls.txt");
    let pool = Arc::new(Semaphore::new(10));

    let db = Arc::new(Database::new().await);

    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("{}", e);
        }
    };
    let mut tasks = Vec::new();
    let urls = BufReader::new(file).lines();
    for url in urls.flatten() {
        let db_clone = Arc::clone(&db);
        let pool_clone = Arc::clone(&pool);

        let task = tokio::spawn(async move {
            let _permit = pool_clone.acquire().await.unwrap();
            process_url(url, db_clone).await;
        });
        tasks.push(task);
    }
    for task in tasks {
        let _ = task.await;
    }
    Ok(())
}
