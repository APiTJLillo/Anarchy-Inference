// Web scraping example in JavaScript
// This demonstrates fetching a webpage and extracting data

const fetch = require('node-fetch');

async function main() {
  const url = "https://example.com";
  console.log(`Fetching ${url}`);
  
  try {
    // Fetch webpage content
    const response = await fetch(url);
    
    // Check if request was successful
    if (response.status !== 200) {
      console.log(`Error: ${response.status}`);
      return false;
    }
    
    // Extract all paragraph text
    const content = await response.text();
    const paragraphs = [];
    
    // Simple regex to extract paragraphs
    const regex = /<p>(.*?)<\/p>/gs;
    let matches;
    while ((matches = regex.exec(content)) !== null) {
      paragraphs.push(matches[1]);
    }
    
    // Print results
    console.log(`Found ${paragraphs.length} paragraphs`);
    paragraphs.slice(0, 3).forEach((p, i) => {
      console.log(`${i}: ${p}`);
    });
    
    // Save results to file
    const fs = require('fs');
    fs.writeFileSync("results.txt", paragraphs.join("\n"));
    console.log("Results saved to results.txt");
    
    return true;
  } catch (error) {
    console.log(`Exception occurred during web scraping: ${error}`);
    return false;
  }
}

main().then(result => {
  process.exit(result ? 0 : 1);
});
