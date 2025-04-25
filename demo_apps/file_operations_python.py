#!/usr/bin/env python3
"""
File operations example in Python
This demonstrates reading, writing, and manipulating files
"""

import os
import json

def main():
    # Define file paths
    input_file = "input.txt"
    output_file = "output.txt"
    statistics_file = "file_stats.json"
    
    print("Starting file operations")
    
    try:
        # Check if input file exists
        if not os.path.exists(input_file):
            print("Input file not found. Creating sample file.")
            sample_content = "Line 1: This is a sample text file.\n"
            sample_content += "Line 2: It contains multiple lines of text.\n"
            sample_content += "Line 3: We will process this file.\n"
            sample_content += "Line 4: And generate statistics.\n"
            sample_content += "Line 5: Then write the results to new files."
            
            with open(input_file, 'w') as f:
                f.write(sample_content)
            print(f"Created sample input file: {input_file}")
        
        # Read the input file
        print(f"Reading file: {input_file}")
        with open(input_file, 'r') as f:
            content = f.read()
        
        # Process the file content
        lines = content.split('\n')
        print(f"File has {len(lines)} lines")
        
        # Calculate statistics
        total_chars = len(content)
        total_words = len(content.split())
        avg_chars_per_line = total_chars / len(lines) if lines else 0
        
        # Create a modified version of the content
        modified_content = content.upper()
        
        # Write the modified content to the output file
        with open(output_file, 'w') as f:
            f.write(modified_content)
        print(f"Modified content written to: {output_file}")
        
        # Create a dictionary with file statistics
        stats = {
            "filename": input_file,
            "lines": len(lines),
            "characters": total_chars,
            "words": total_words,
            "avgCharsPerLine": avg_chars_per_line,
            "modified": output_file
        }
        
        # Write the statistics to a JSON file
        with open(statistics_file, 'w') as f:
            json.dump(stats, f, indent=2)
        print(f"File statistics written to: {statistics_file}")
        
        # Display the statistics
        print("File Statistics:")
        print(f"  Lines: {len(lines)}")
        print(f"  Characters: {total_chars}")
        print(f"  Words: {total_words}")
        print(f"  Avg Chars Per Line: {avg_chars_per_line:.2f}")
        
        return True
        
    except Exception as e:
        print(f"Error during file operations: {e}")
        return False

if __name__ == "__main__":
    main()
