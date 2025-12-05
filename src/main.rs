use async_std::fs;
use async_std::stream::StreamExt;
use encoding_rs::{Encoding, UTF_8};
use chardetng::EncodingDetector;
use std::path::Path;
use regex::RegexBuilder;

async fn read_text_file_async(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(file_path).await?;

    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let detected = detector.guess(None, true);
    let encoding_label = detected.name();

    let encoding = Encoding::for_label(encoding_label.as_bytes()).unwrap_or(UTF_8);

    let (decoded, _, had_errors) = encoding.decode(&bytes);

    if had_errors {
        eprintln!("Warning: There were decoding errors.");
    }

    Ok(decoded.to_string())
}

fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex = String::new();
    for ch in pattern.chars() {
        match ch {
            '%' => regex.push_str(".*"),
            '_' => regex.push('.'),
            '\\' => regex.push_str("\\\\"),
            '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' => {
                regex.push('\\');
                regex.push(ch);
            }
            other => regex.push(other),
        }
    }
    regex
}

async fn search_folder_for_query<P: AsRef<Path>>(
    folder: P,
    query: &str,
    case_insensitive: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let folder_path = folder.as_ref();
    let mut file_paths = Vec::new();
    let mut entries = fs::read_dir(folder_path).await?;

    // Collect all file paths from directory
    while let Some(entry_result) = entries.next().await {
        if let Ok(entry) = entry_result {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Some(path_str) = entry.path().to_str() {
                        file_paths.push(path_str.to_string());
                    }
                }
            }
        }
    }

    let mut matching_files = Vec::new();

    // Detect if this is a wildcard query
    let is_wildcard = query.contains('%') || query.contains('_');

    // Build regex once if wildcard query
    let regex_opt = if is_wildcard {
        let regex_str = wildcard_to_regex(query);
        let re = RegexBuilder::new(&regex_str)
            .case_insensitive(case_insensitive)
            .build()?;
        Some(re)
    } else {
        None
    };

    // For plain substring search, prepare lowercase version if case-insensitive
    let search_query = if !is_wildcard && case_insensitive {
        Some(query.to_lowercase())
    } else {
        None
    };

    // Process files
    for file_path in file_paths {
        match read_text_file_async(&file_path).await {
            Ok(content) => {
                let is_match = if let Some(ref re) = regex_opt {
                    // Use regex for wildcard queries
                    re.is_match(&content)
                } else if let Some(ref q) = search_query {
                    // Use lowercased substring for case-insensitive non-wildcard
                    content.to_lowercase().contains(q)
                } else {
                    // Use direct substring for case-sensitive non-wildcard
                    content.contains(query)
                };

                if is_match {
                    matching_files.push(file_path);
                }
            }
            Err(_) => {
                // Skip files that cannot be read
                continue;
            }
        }
    }

    Ok(matching_files)
}

// context7: async main entry point
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder_path = "D:/Projecten_Thuis/find_my_shit/bac_files";

    // Example: search with wildcard pattern
    let search_query = "%strsql%";

    println!("Searching for '{search_query}' in folder: {folder_path}\n");

    match search_folder_for_query(folder_path, search_query, true).await {
        Ok(matching_files) => {
            if matching_files.is_empty() {
                println!("No files found matching '{search_query}'");
            } else {
                println!("Files matching '{search_query}': ");
                for path in matching_files {
                    println!("  {path}");
                }
            }
        }
        Err(e) => eprintln!("Error during search: {e}"),
    }

    Ok(())
}
