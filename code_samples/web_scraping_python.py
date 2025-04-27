#!/usr/bin/env python3
"""
Web scraping example in Python
This demonstrates fetching a webpage and extracting data
"""

import requests
import re
from typing import List, Optional

def main() -> bool:
    url = "https://example.com"
    print(f"Fetching {url}")
    
    try:
        # Fetch webpage content
        response = requests.get(url)
        
        # Check if request was successful
        if response.status_code != 200:
            print(f"Error: {response.status_code}")
            return False
        
        # Extract all paragraph text
        content = response.text
        paragraphs = []
        
        # Simple regex to extract paragraphs
        regex = r"<p>(.*?)</p>"
        matches = re.findall(regex, content, re.DOTALL)
        
        for m in matches:
            paragraphs.append(m)
        
        # Print results
        print(f"Found {len(paragraphs)} paragraphs")
        for i, p in enumerate(paragraphs):
            if i < 3:  # Only show first 3
                print(f"{i}: {p}")
        
        # Save results to file
        with open("results.txt", "w") as f:
            f.write("\n".join(paragraphs))
        print("Results saved to results.txt")
        
        return True
    
    except Exception as e:
        print(f"Exception occurred during web scraping: {e}")
        return False

if __name__ == "__main__":
    main()
