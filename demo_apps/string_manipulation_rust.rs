use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use serde::{Serialize, Deserialize};

// String manipulation example in Rust
// This demonstrates various string operations and transformations

#[derive(Serialize, Deserialize)]
struct StringStats {
    text: String,
    length: usize,
    word_count: usize,
    char_frequency: HashMap<char, usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Sample text for manipulation
    let text = "The quick brown fox jumps over the lazy dog";
    println!("Original text: {}", text);
    
    // String length
    let length = text.len();
    println!("Length: {}", length);
    
    // Uppercase and lowercase
    let upper = text.to_uppercase();
    let lower = text.to_lowercase();
    println!("Uppercase: {}", upper);
    println!("Lowercase: {}", lower);
    
    // Substring extraction
    let sub1 = &text[4..9];  // "quick"
    let sub2 = &text[16..19]; // "fox"
    println!("Substring 1: {}", sub1);
    println!("Substring 2: {}", sub2);
    
    // String replacement
    let replaced = text.replace("lazy", "energetic");
    println!("Replaced: {}", replaced);
    
    // String splitting
    let words: Vec<&str> = text.split(' ').collect();
    println!("Word count: {}", words.len());
    println!("Words: {:?}", words);
    
    // String joining
    let joined = words.join("-");
    println!("Joined with hyphens: {}", joined);
    
    // String searching
    let fox_index = text.find("fox").unwrap_or(0);
    let dog_index = text.find("dog").unwrap_or(0);
    println!("'fox' found at index: {}", fox_index);
    println!("'dog' found at index: {}", dog_index);
    
    // Check if string contains substring
    let has_fox = text.contains("fox");
    let has_zebra = text.contains("zebra");
    println!("Contains 'fox': {}", if has_fox { "Yes" } else { "No" });
    println!("Contains 'zebra': {}", if has_zebra { "Yes" } else { "No" });
    
    // String trimming
    let padded_text = format!("   {}   ", text);
    let trimmed = padded_text.trim();
    println!("Padded text: '{}'", padded_text);
    println!("Trimmed text: '{}'", trimmed);
    
    // String reversal
    fn reverse_string(s: &str) -> String {
        s.chars().rev().collect()
    }
    
    let reversed = reverse_string(text);
    println!("Reversed: {}", reversed);
    
    // Count occurrences of a character
    fn count_char(s: &str, c: char) -> usize {
        s.chars().filter(|&ch| ch == c).count()
    }
    
    let e_count = count_char(text, 'e');
    println!("Occurrences of 'e': {}", e_count);
    
    // Generate statistics
    let mut char_frequency = HashMap::new();
    
    // Count frequency of each character
    for c in text.chars() {
        if c != ' ' {
            *char_frequency.entry(c).or_insert(0) += 1;
        }
    }
    
    let stats = StringStats {
        text: text.to_string(),
        length,
        word_count: words.len(),
        char_frequency,
    };
    
    // Save statistics to file
    let json_stats = serde_json::to_string_pretty(&stats)?;
    let mut file = File::create("string_stats.json")?;
    file.write_all(json_stats.as_bytes())?;
    println!("String statistics saved to string_stats.json");
    
    Ok(())
}
