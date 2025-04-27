#!/usr/bin/env python3
"""
Sample Data Generator for Anarchy Inference Benchmark Framework

This script generates sample data files needed for the benchmark tests:
- sample_data.csv: For data processing examples
- input.txt: For file operations examples
- weather_data.json: Mock response for API interaction examples

Usage:
  python3 generate_sample_data.py
"""

import csv
import json
import random
import os
from pathlib import Path

def generate_sample_csv():
    """Generate a sample CSV file with random data"""
    print("Generating sample_data.csv...")
    
    # Create data directory if it doesn't exist
    os.makedirs("data", exist_ok=True)
    
    # Define headers and prepare data
    headers = ["id", "name", "value", "category"]
    rows = []
    
    # Generate 20 random records
    categories = ["A", "B", "C", "D"]
    for i in range(1, 21):
        rows.append({
            "id": i,
            "name": f"Item {i}",
            "value": round(random.uniform(10, 100), 2),
            "category": random.choice(categories)
        })
    
    # Write to CSV file
    with open("data/sample_data.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=headers)
        writer.writeheader()
        writer.writerows(rows)
    
    print(f"Created data/sample_data.csv with {len(rows)} records")

def generate_input_text():
    """Generate a sample text file for file operations examples"""
    print("Generating input.txt...")
    
    content = "Line 1: This is a sample text file.\n"
    content += "Line 2: It contains multiple lines of text.\n"
    content += "Line 3: We will process this file.\n"
    content += "Line 4: And generate statistics.\n"
    content += "Line 5: Then write the results to new files."
    
    with open("data/input.txt", "w") as f:
        f.write(content)
    
    print("Created data/input.txt")

def generate_weather_data():
    """Generate a mock weather API response"""
    print("Generating weather_data.json...")
    
    weather_data = {
        "coord": {
            "lon": -0.1257,
            "lat": 51.5085
        },
        "weather": [
            {
                "id": 800,
                "main": "Clear",
                "description": "clear sky",
                "icon": "01d"
            }
        ],
        "base": "stations",
        "main": {
            "temp": 18.5,
            "feels_like": 17.8,
            "temp_min": 16.2,
            "temp_max": 20.1,
            "pressure": 1012,
            "humidity": 65
        },
        "visibility": 10000,
        "wind": {
            "speed": 3.6,
            "deg": 250
        },
        "clouds": {
            "all": 0
        },
        "dt": 1619352000,
        "sys": {
            "type": 2,
            "id": 2019646,
            "country": "GB",
            "sunrise": 1619325935,
            "sunset": 1619378329
        },
        "timezone": 3600,
        "id": 2643743,
        "name": "London",
        "cod": 200
    }
    
    with open("data/weather_data.json", "w") as f:
        json.dump(weather_data, f, indent=2)
    
    print("Created data/weather_data.json")

def create_readme():
    """Create a README file for the benchmark framework"""
    print("Creating README.md...")
    
    content = """# Anarchy Inference Benchmark Framework

This directory contains the benchmark framework for comparing token efficiency between Anarchy Inference and other programming languages.

## Structure

- `benchmark_framework.py`: Main benchmark script
- `code_samples/`: Code samples in different languages
  - `web_scraping_*.{ai,py,js,rs}`: Web scraping examples
  - `data_processing_*.{ai,py,js,rs}`: Data processing examples
  - `api_interaction_*.{ai,py,js,rs}`: API interaction examples
  - `file_operations_*.{ai,py,js,rs}`: File operations examples
  - `string_manipulation_*.{ai,py,js,rs}`: String manipulation examples
- `data/`: Sample data files for benchmarks
- `demo_apps/`: Demonstration applications showcasing real-world use cases

## Running the Benchmark

1. Install dependencies:
   ```
   pip install openai psutil matplotlib
   ```

2. Configure Azure OpenAI credentials in `config.json`:
   ```json
   {
     "azure_openai": {
       "api_type": "azure",
       "api_version": "2023-05-15",
       "endpoint": "YOUR_ENDPOINT",
       "api_key": "YOUR_API_KEY"
     }
   }
   ```

3. Run the benchmark:
   ```
   python benchmark_framework.py --config config.json
   ```

4. View results in the `benchmark_results` directory.

## Demonstration Applications

The `demo_apps` directory contains real-world applications implemented in Anarchy Inference:

- `web_scraper_demo.ai`: News scraper with sentiment analysis
- `data_analyzer_demo.ai`: Data analysis and visualization tool
- `api_client_demo.ai`: API client with caching and rate limiting

These applications showcase the token efficiency and capabilities of Anarchy Inference in practical scenarios.
"""
    
    with open("README.md", "w") as f:
        f.write(content)
    
    print("Created README.md")

def main():
    """Main function to generate all sample data"""
    print("Generating sample data for Anarchy Inference Benchmark Framework...")
    
    # Create data directory
    os.makedirs("data", exist_ok=True)
    
    # Generate all sample files
    generate_sample_csv()
    generate_input_text()
    generate_weather_data()
    create_readme()
    
    print("\nSample data generation complete!")

if __name__ == "__main__":
    main()
