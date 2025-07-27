mod utils;
use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    str,
};
use tokenizers::tokenizer::{Result, Tokenizer};

use human_regex::{exactly, one_or_more, or, punctuation, whitespace, word_boundary};
use stop_words::{get as sget, LANGUAGE};
use utils::{is_binary_extension, is_text_content};

use crate::utils::{ extract_structured_data, ExtractedData, Store};
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
fn create_tokenizer() -> Result<Tokenizer> {
    let tokenizer = Tokenizer::from_pretrained("bert-large-cased", None)?;
    Ok(tokenizer)
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
                        url,content_type_str
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

async fn process(url: String) -> Result<ExtractedData> {
    let tokenizer = create_tokenizer().unwrap();
    let client = Client::new();

    if is_binary_extension(&url) || !check_content_type(&client,& url).await {
        return Err("Check Failed".into());
    }

    let page=get_data(&client, &url).await;

    let output = extract_structured_data(page);
    println!("{}",output);
    Ok(output)

}

#[tokio::main]
async fn main() -> Result<()> {
    let path=Path::new("../urls.txt");

    let file=match File::open(path) {
        Ok(file)=>file,
        Err(e)=>{panic!("{}",e);}

        
        
    };
    
    let urls=BufReader::new(file).lines();
    let mut stores: Vec<ExtractedData> = Vec::new();
    let mut cnt=0;
    for url in urls {
        stores.push(process(url.unwrap()).await?);
        cnt+=1;
    }
    
    Ok(())
}
