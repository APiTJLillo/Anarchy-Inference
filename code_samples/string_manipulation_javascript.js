// String manipulation example in JavaScript
// This demonstrates various string operations and transformations

const fs = require('fs');

function main() {
  // Sample text for manipulation
  const text = "The quick brown fox jumps over the lazy dog";
  console.log(`Original text: ${text}`);
  
  try {
    // String length
    const length = text.length;
    console.log(`Length: ${length}`);
    
    // Uppercase and lowercase
    const upper = text.toUpperCase();
    const lower = text.toLowerCase();
    console.log(`Uppercase: ${upper}`);
    console.log(`Lowercase: ${lower}`);
    
    // Substring extraction
    const sub1 = text.substring(4, 9);  // "quick"
    const sub2 = text.substring(16, 19); // "fox"
    console.log(`Substring 1: ${sub1}`);
    console.log(`Substring 2: ${sub2}`);
    
    // String replacement
    const replaced = text.replace("lazy", "energetic");
    console.log(`Replaced: ${replaced}`);
    
    // String splitting
    const words = text.split(" ");
    console.log(`Word count: ${words.length}`);
    console.log(`Words: ${JSON.stringify(words)}`);
    
    // String joining
    const joined = words.join("-");
    console.log(`Joined with hyphens: ${joined}`);
    
    // String searching
    const foxIndex = text.indexOf("fox");
    const dogIndex = text.indexOf("dog");
    console.log(`'fox' found at index: ${foxIndex}`);
    console.log(`'dog' found at index: ${dogIndex}`);
    
    // Check if string contains substring
    const hasFox = text.includes("fox");
    const hasZebra = text.includes("zebra");
    console.log(`Contains 'fox': ${hasFox ? "Yes" : "No"}`);
    console.log(`Contains 'zebra': ${hasZebra ? "Yes" : "No"}`);
    
    // String trimming
    const paddedText = "   " + text + "   ";
    const trimmed = paddedText.trim();
    console.log(`Padded text: '${paddedText}'`);
    console.log(`Trimmed text: '${trimmed}'`);
    
    // String reversal
    function reverseString(str) {
      return str.split("").reverse().join("");
    }
    
    const reversed = reverseString(text);
    console.log(`Reversed: ${reversed}`);
    
    // Count occurrences of a character
    function countChar(str, char) {
      return str.split(char).length - 1;
    }
    
    const eCount = countChar(text, "e");
    console.log(`Occurrences of 'e': ${eCount}`);
    
    // Generate statistics
    const stats = {
      text: text,
      length: length,
      wordCount: words.length,
      charFrequency: {}
    };
    
    // Count frequency of each character
    for (const c of text) {
      if (c !== " ") {
        if (!stats.charFrequency[c]) {
          stats.charFrequency[c] = 0;
        }
        stats.charFrequency[c]++;
      }
    }
    
    // Save statistics to file
    fs.writeFileSync("string_stats.json", JSON.stringify(stats, null, 2));
    console.log("String statistics saved to string_stats.json");
    
    return true;
    
  } catch (error) {
    console.log(`Error during string manipulation: ${error.message}`);
    return false;
  }
}

main();
