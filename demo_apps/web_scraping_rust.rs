// Web scraping example in Rust
// This demonstrates fetching a webpage and extracting data

use reqwest;
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com";
    println!("Fetching {}", url);
    
    // Fetch webpage content
    let response = reqwest::get(url).await?;
    
    // Check if request was successful
    if !response.status().is_success() {
        println!("Error: {}", response.status());
        return Ok(());
    }
    
    // Extract all paragraph text
    let content = response.text().await?;
    let mut paragraphs = Vec::new();
    
    // Simple regex to extract paragraphs
    let re = Regex::new(r"<p>(.*?)</p>")?;
    for cap in re.captures_iter(&content) {
        paragraphs.push(cap[1].to_string());
    }
    
    // Print results
    println!("Found {} paragraphs", paragraphs.len());
    for (i, p) in paragraphs.iter().enumerate().take(3) {
        println!("{}: {}", i, p);
    }
    
    // Save results to file
    let mut file = File::create("results.txt")?;
    write!(file, "{}", paragraphs.join("\n"))?;
    println!("Results saved to results.txt");
    
    Ok(())
}
