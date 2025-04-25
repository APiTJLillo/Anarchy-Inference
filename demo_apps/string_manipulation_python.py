#!/usr/bin/env python3
"""
String manipulation example in Python
This demonstrates various string operations and transformations
"""

import json

def main():
    # Sample text for manipulation
    text = "The quick brown fox jumps over the lazy dog"
    print(f"Original text: {text}")
    
    try:
        # String length
        length = len(text)
        print(f"Length: {length}")
        
        # Uppercase and lowercase
        upper = text.upper()
        lower = text.lower()
        print(f"Uppercase: {upper}")
        print(f"Lowercase: {lower}")
        
        # Substring extraction
        sub1 = text[4:9]  # "quick"
        sub2 = text[16:19]  # "fox"
        print(f"Substring 1: {sub1}")
        print(f"Substring 2: {sub2}")
        
        # String replacement
        replaced = text.replace("lazy", "energetic")
        print(f"Replaced: {replaced}")
        
        # String splitting
        words = text.split(" ")
        print(f"Word count: {len(words)}")
        print(f"Words: {words}")
        
        # String joining
        joined = "-".join(words)
        print(f"Joined with hyphens: {joined}")
        
        # String searching
        fox_index = text.find("fox")
        dog_index = text.find("dog")
        print(f"'fox' found at index: {fox_index}")
        print(f"'dog' found at index: {dog_index}")
        
        # Check if string contains substring
        has_fox = "fox" in text
        has_zebra = "zebra" in text
        print(f"Contains 'fox': {'Yes' if has_fox else 'No'}")
        print(f"Contains 'zebra': {'Yes' if has_zebra else 'No'}")
        
        # String trimming
        padded_text = "   " + text + "   "
        trimmed = padded_text.strip()
        print(f"Padded text: '{padded_text}'")
        print(f"Trimmed text: '{trimmed}'")
        
        # String reversal
        def reverse_string(s):
            return s[::-1]
        
        reversed_text = reverse_string(text)
        print(f"Reversed: {reversed_text}")
        
        # Count occurrences of a character
        def count_char(s, char):
            return s.count(char)
        
        e_count = count_char(text, "e")
        print(f"Occurrences of 'e': {e_count}")
        
        # Generate statistics
        stats = {
            "text": text,
            "length": length,
            "wordCount": len(words),
            "charFrequency": {}
        }
        
        # Count frequency of each character
        for c in text:
            if c != " ":
                if c not in stats["charFrequency"]:
                    stats["charFrequency"][c] = 0
                stats["charFrequency"][c] += 1
        
        # Save statistics to file
        with open("string_stats.json", "w") as f:
            json.dump(stats, f, indent=2)
        print("String statistics saved to string_stats.json")
        
        return True
        
    except Exception as e:
        print(f"Error during string manipulation: {e}")
        return False

if __name__ == "__main__":
    main()
