use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;
use std::error::Error;

// Data processing example in Rust
// This demonstrates loading, processing, and analyzing data

fn main() -> Result<(), Box<dyn Error>> {
    // Load data from CSV file
    println!("Loading data from file");
    let file_path = "sample_data.csv";
    
    // Read file content
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // Parse header
    let header = lines.next()
        .ok_or("Empty CSV file")??;
    let headers: Vec<&str> = header.split(',').collect();
    
    // Parse data
    let mut data = Vec::new();
    for line_result in lines {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        
        let fields: Vec<&str> = line.split(',').collect();
        let mut row = HashMap::new();
        
        for (i, &column) in headers.iter().enumerate() {
            if i < fields.len() {
                row.insert(column.to_string(), fields[i].to_string());
            }
        }
        
        data.push(row);
    }
    
    println!("Loaded {} records", data.len());
    
    // Calculate statistics
    let numeric_column = "value";
    let mut sum = 0.0;
    let mut min = data[0][numeric_column].parse::<f64>()?;
    let mut max = min;
    
    for row in &data {
        let val = row[numeric_column].parse::<f64>()?;
        sum += val;
        min = if val < min { val } else { min };
        max = if val > max { val } else { max };
    }
    
    let avg = sum / data.len() as f64;
    
    // Output results
    println!("Statistics for {}:", numeric_column);
    println!("  Count: {}", data.len());
    println!("  Sum: {}", sum);
    println!("  Average: {}", avg);
    println!("  Min: {}", min);
    println!("  Max: {}", max);
    
    // Filter data
    let filtered: Vec<&HashMap<String, String>> = data.iter()
        .filter(|row| row[numeric_column].parse::<f64>().unwrap_or(0.0) > avg)
        .collect();
    
    println!("Found {} records above average", filtered.len());
    
    // Save results
    let mut output = File::create("statistics.csv")?;
    writeln!(output, "column,count,sum,average,min,max")?;
    writeln!(output, "{},{},{},{},{},{}", 
             numeric_column, data.len(), sum, avg, min, max)?;
    
    println!("Results saved to statistics.csv");
    
    Ok(())
}
