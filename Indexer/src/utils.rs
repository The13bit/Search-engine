use human_regex::{
    any, exactly, one_or_more, or, punctuation, text as htext, whitespace, word_boundary,
    zero_or_more,
};
use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use mongodb::bson::oid::ObjectId;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io::Write,
    path::Path,
};
use stop_words::{get as sget, LANGUAGE};

use crate::models::{Document, Words};

pub fn is_binary_extension(url: &String) -> bool {
    let binary_extensions = [
        ".exe", ".apk", ".dmg", ".pkg", ".deb", ".rpm", ".zip", ".rar", ".7z", ".tar", ".gz",
        ".bz2", ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".jpg", ".jpeg", ".png",
        ".gif", ".bmp", ".svg", ".ico", ".mp3", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".bin",
        ".dll", ".so", ".dylib", ".class", ".jar",
    ];

    let url_lower = url.to_lowercase();
    binary_extensions
        .iter()
        .any(|&ext| url_lower.ends_with(ext))
}
pub fn is_text_content(content_type: &String) -> bool {
    let allowed_types = [
        "text/html",
        "text/plain",
        "text/xml",
        "application/xml",
        "application/xhtml+xml",
        "text/css",
        "text/javascript",
        "application/json",
        "application/ld+json",
    ];

    allowed_types
        .iter()
        .any(|&allowed| content_type.starts_with(allowed))
}

pub fn save_idf(num_doc: usize, global_count: &HashMap<String, i32>) {
    let mut idf: HashMap<String, f32> = HashMap::new();
    let doc_num = num_doc as f32 + 1.0;
    let mut mn = f32::MAX;
    let mut mx = f32::MIN;
    for (word, count) in global_count.iter() {
        idf.entry(word.clone())
            .or_insert(f32::log10(doc_num / (*count as f32 + 1.0)) + 1.0);
    }

    let jsn = serde_json::to_string(&idf).unwrap();

    let path = Path::new("../out.json");
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            panic!("File creation failed:{}", e);
        }
    };

    match file.write(&jsn.as_bytes()) {
        Ok(_) => {
            print!("File creation success")
        }
        Err(_) => {
            panic!("file failed")
        }
    };
}

pub struct Store {
    url: String,
    pub tf_score: HashMap<String, i32>,
}

impl Store {
    pub fn new(url: String, tf_score: HashMap<String, i32>) -> Store {
        Store { url, tf_score }
    }
}

fn clean_corpus(document: String) -> String {
    let words = sget(LANGUAGE::English);
    //println!("{:?}", words);
    // Remove punctuation and lowercase the text to make parsing easier
    let lowercase_doc = document.to_ascii_lowercase();
    let regex_for_punctuation = one_or_more(punctuation());

    //println!("{}", regex_for_punctuation.to_regex());

    let text_without_punctuation = regex_for_punctuation
        .to_regex()
        .replace_all(&lowercase_doc, " ");
    // Make a regex to match stopwords with trailing spaces and punctuation
    let regex_for_stop_words =
        word_boundary() + exactly(1, or(&words)) + word_boundary() + one_or_more(whitespace());
    //println!("{}", regex_for_stop_words.to_regex());
    // Remove stop words
    let clean_text = regex_for_stop_words
        .to_regex()
        .replace_all(&text_without_punctuation, "");
    clean_text.to_string()
}

#[derive(Debug)]
pub struct ExtractedData {
    title: String,
    description: String,
    canonical_url: String,
    summary_text: String,
    full_text: Vec<String>,
}
impl fmt::Display for ExtractedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExtractedData {{")?;
        writeln!(f, "  title: {:?}", self.title)?;
        writeln!(f, "  description: {:?}", self.description)?;
        writeln!(f, "  canonical_url: {:?}", self.canonical_url)?;
        writeln!(f, "  summary_text: \"{}\"", self.summary_text)?;
        writeln!(f, "  full_text: {:?} words", self.full_text.len())?;
        write!(f, "}}")
    }
}


pub fn extract_structured_data(text: String,url: String) -> Document {
    let mut og_title = None;
    let mut og_description = None;
    let mut og_url = None;
    let mut meta_title = None;
    let mut meta_description = None;
    let mut paragraphs = Vec::new();
    let mut page_title = None;
    let mut tmp = None;
    // Regex to remove brackets content (similar to BRACKETS_PATTERN)

    let brackets_pattern = (htext("[") + zero_or_more(any()).lazy() + htext("]")).to_regex();
    //println!("{}", brackets_pattern);

    rewrite_str(
        &text,
        RewriteStrSettings {
            element_content_handlers: vec![
                // Extract title tag content
                text!("title", |t| {
                    let text_content = t.as_str().trim();
                    if !text_content.is_empty() {
                        page_title = Some(text_content.to_string());
                    }
                    Ok(())
                }),
                // Extract paragraph content
                text!("p", |t| {
                    let text_content = t.as_str().to_string();
                    if !text_content.is_empty() {
                        paragraphs.push(text_content);
                    }
                    Ok(())
                }),
                // Extract meta tags with property attributes (Open Graph)
                element!("meta[property]", |el| {
                    if let (Some(property), Some(content)) =
                        (el.get_attribute("property"), el.get_attribute("content"))
                    {
                        let content = content.trim().to_string();
                        if !content.is_empty() {
                            match property.as_str() {
                                "og:title" => og_title = Some(content),
                                "og:description" => og_description = Some(content),
                                "og:url" => og_url = Some(content),
                                _ => {}
                            }
                        }
                    }
                    Ok(())
                }),
                // Extract meta tags with name attributes
                element!("meta[name]", |el| {
                    if let (Some(name), Some(content)) =
                        (el.get_attribute("name"), el.get_attribute("content"))
                    {
                        let content = content.trim().to_string();
                        if !content.is_empty() {
                            match name.as_str() {
                                "title" => meta_title = Some(content),
                                "description" => meta_description = Some(content),
                                "url" => tmp = Some(content),
                                _ => {}
                            }
                        }
                    }
                    Ok(())
                }),
            ],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();

    let title = og_title
        .or(meta_title)
        .or(page_title)
        .unwrap_or("".to_string());
    let description = og_description
        .or(meta_description)
        .unwrap_or("".to_string());
    let canonical_url = og_url.or(tmp).unwrap_or("".to_string());

    let page_text = brackets_pattern
        .replace_all(paragraphs.join(" ").as_str(), " ")
        .trim()
        .to_string();

    let words: Vec<&str> = page_text.split_whitespace().collect();
    let summary_text = if words.len() < 500 {
        page_text.clone()
    } else {
        words[..500].join(" ")
    };

    let text: Vec<String> = clean_corpus(page_text)
        .split(" ")
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();

    Document::new(url,title, description, canonical_url, summary_text, text)
}

pub fn create_frequency(data: &Document) -> Vec<Words> {
    let mut count: HashMap<&String, i32> = HashMap::new();
    let corpus = data.get_full_text();
    for i in corpus {
        count.entry(i).and_modify(|x| *x += 1).or_insert(1);
    }
    //adding multiplier to keyword appearing in title
    println!("{}", data.get_title());
    let title: Vec<String> = clean_corpus(data.get_title())
        .split(" ")
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
    println!("{:?}", title);
    for i in &title {
        count.entry(i).and_modify(|x| *x += 50);
    }
    //adding multiplier to keyword appearing in descripton
    println!("{}", data.get_description());
    let descripton: Vec<String> = clean_corpus(data.get_description())
        .split(" ")
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
    println!("{:?}", descripton);
    for i in &descripton {
        count.entry(i).and_modify(|x| *x += 10);
    }

    let mut key_val_pairs: Vec<(&String, i32)> = count.into_iter().collect();

    key_val_pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let top_words = key_val_pairs.into_iter().take(1000);

    println!("{:?}", top_words);
    let mut words_arr:Vec<Words>=Vec::new();
    for (i,j) in top_words{
        words_arr.push(
            Words::new(data.get_id(), i.clone(),j)
        );
    }
    words_arr
}
