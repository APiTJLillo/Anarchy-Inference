#!/usr/bin/env python3
"""
Data processing example in Python
This demonstrates loading, processing, and analyzing data
"""

import csv
from typing import Dict, List, Any

def main() -> bool:
    # Load data from CSV file
    print("Loading data from file")
    file_path = "sample_data.csv"
    
    try:
        # Read file content
        data = []
        with open(file_path, 'r') as file:
            csv_reader = csv.DictReader(file)
            for row in csv_reader:
                data.append(row)
        
        print(f"Loaded {len(data)} records")
        
        # Calculate statistics
        numeric_column = "value"
        values = [float(row[numeric_column]) for row in data]
        
        count = len(values)
        sum_val = sum(values)
        avg = sum_val / count if count > 0 else 0
        min_val = min(values) if values else 0
        max_val = max(values) if values else 0
        
        # Output results
        print(f"Statistics for {numeric_column}:")
        print(f"  Count: {count}")
        print(f"  Sum: {sum_val}")
        print(f"  Average: {avg}")
        print(f"  Min: {min_val}")
        print(f"  Max: {max_val}")
        
        # Filter data
        filtered = [row for row in data if float(row[numeric_column]) > avg]
        print(f"Found {len(filtered)} records above average")
        
        # Save results
        with open("statistics.csv", 'w', newline='') as file:
            writer = csv.writer(file)
            writer.writerow(["column", "count", "sum", "average", "min", "max"])
            writer.writerow([numeric_column, count, sum_val, avg, min_val, max_val])
        
        print("Results saved to statistics.csv")
        return True
        
    except Exception as e:
        print(f"Error processing data: {e}")
        return False

if __name__ == "__main__":
    main()
