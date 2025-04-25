use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json;
use std::error::Error;

// File operations example in Rust
// This demonstrates reading, writing, and manipulating files

#[derive(Serialize, Deserialize)]
struct FileStats {
    filename: String,
    lines: usize,
    characters: usize,
    words: usize,
    avg_chars_per_line: f64,
    modified: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Define file paths
    let input_file = "input.txt";
    let output_file = "output.txt";
    let statistics_file = "file_stats.json";
    
    println!("Starting file operations");
    
    // Check if input file exists
    if !Path::new(input_file).exists() {
        println!("Input file not found. Creating sample file.");
        let sample_content = "Line 1: This is a sample text file.\n\
                             Line 2: It contains multiple lines of text.\n\
                             Line 3: We will process this file.\n\
                             Line 4: And generate statistics.\n\
                             Line 5: Then write the results to new files.";
        
        fs::write(input_file, sample_content)?;
        println!("Created sample input file: {}", input_file);
    }
    
    // Read the input file
    println!("Reading file: {}", input_file);
    let mut content = String::new();
    let mut file = File::open(input_file)?;
    file.read_to_string(&mut content)?;
    
    // Process the file content
    let lines: Vec<&str> = content.split('\n').collect();
    println!("File has {} lines", lines.len());
    
    // Calculate statistics
    let total_chars = content.len();
    let total_words = content.split_whitespace().count();
    let avg_chars_per_line = total_chars as f64 / lines.len() as f64;
    
    // Create a modified version of the content
    let modified_content = content.to_uppercase();
    
    // Write the modified content to the output file
    let mut output_file_handle = File::create(output_file)?;
    output_file_handle.write_all(modified_content.as_bytes())?;
    println!("Modified content written to: {}", output_file);
    
    // Create a struct with file statistics
    let stats = FileStats {
        filename: input_file.to_string(),
        lines: lines.len(),
        characters: total_chars,
        words: total_words,
        avg_chars_per_line,
        modified: output_file.to_string(),
    };
    
    // Write the statistics to a JSON file
    let json_stats = serde_json::to_string_pretty(&stats)?;
    let mut stats_file = File::create(statistics_file)?;
    stats_file.write_all(json_stats.as_bytes())?;
    println!("File statistics written to: {}", statistics_file);
    
    // Display the statistics
    println!("File Statistics:");
    println!("  Lines: {}", lines.len());
    println!("  Characters: {}", total_chars);
    println!("  Words: {}", total_words);
    println!("  Avg Chars Per Line: {:.2}", avg_chars_per_line);
    
    Ok(())
}
