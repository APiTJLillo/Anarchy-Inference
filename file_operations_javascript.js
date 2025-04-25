// File operations example in JavaScript
// This demonstrates reading, writing, and manipulating files

const fs = require('fs');
const path = require('path');

function main() {
  // Define file paths
  const inputFile = "input.txt";
  const outputFile = "output.txt";
  const statisticsFile = "file_stats.json";
  
  console.log("Starting file operations");
  
  try {
    // Check if input file exists
    if (!fs.existsSync(inputFile)) {
      console.log("Input file not found. Creating sample file.");
      const sampleContent = "Line 1: This is a sample text file.\n" +
                           "Line 2: It contains multiple lines of text.\n" +
                           "Line 3: We will process this file.\n" +
                           "Line 4: And generate statistics.\n" +
                           "Line 5: Then write the results to new files.";
      
      fs.writeFileSync(inputFile, sampleContent);
      console.log(`Created sample input file: ${inputFile}`);
    }
    
    // Read the input file
    console.log(`Reading file: ${inputFile}`);
    const content = fs.readFileSync(inputFile, 'utf8');
    
    // Process the file content
    const lines = content.split('\n');
    console.log(`File has ${lines.length} lines`);
    
    // Calculate statistics
    const totalChars = content.length;
    const totalWords = content.split(/\s+/).length;
    const avgCharsPerLine = totalChars / lines.length;
    
    // Create a modified version of the content
    const modifiedContent = content.toUpperCase();
    
    // Write the modified content to the output file
    fs.writeFileSync(outputFile, modifiedContent);
    console.log(`Modified content written to: ${outputFile}`);
    
    // Create an object with file statistics
    const stats = {
      filename: inputFile,
      lines: lines.length,
      characters: totalChars,
      words: totalWords,
      avgCharsPerLine: avgCharsPerLine,
      modified: outputFile
    };
    
    // Write the statistics to a JSON file
    fs.writeFileSync(statisticsFile, JSON.stringify(stats, null, 2));
    console.log(`File statistics written to: ${statisticsFile}`);
    
    // Display the statistics
    console.log("File Statistics:");
    console.log(`  Lines: ${lines.length}`);
    console.log(`  Characters: ${totalChars}`);
    console.log(`  Words: ${totalWords}`);
    console.log(`  Avg Chars Per Line: ${avgCharsPerLine.toFixed(2)}`);
    
    return true;
    
  } catch (error) {
    console.log(`Error during file operations: ${error.message}`);
    return false;
  }
}

main();
