// Data processing example in JavaScript
// This demonstrates loading, processing, and analyzing data

const fs = require('fs');
const { parse, stringify } = require('csv-parse/sync');

function main() {
  // Load data from CSV file
  console.log("Loading data from file");
  const filePath = "sample_data.csv";
  
  try {
    // Read file content
    const content = fs.readFileSync(filePath, 'utf8');
    
    // Parse CSV data
    const records = parse(content, {
      columns: true,
      skip_empty_lines: true
    });
    
    console.log(`Loaded ${records.length} records`);
    
    // Calculate statistics
    const numericColumn = "value";
    let sum = 0;
    let min = parseFloat(records[0][numericColumn]);
    let max = min;
    
    records.forEach(row => {
      const val = parseFloat(row[numericColumn]);
      sum += val;
      min = val < min ? val : min;
      max = val > max ? val : max;
    });
    
    const avg = sum / records.length;
    
    // Output results
    console.log(`Statistics for ${numericColumn}:`);
    console.log(`  Count: ${records.length}`);
    console.log(`  Sum: ${sum}`);
    console.log(`  Average: ${avg}`);
    console.log(`  Min: ${min}`);
    console.log(`  Max: ${max}`);
    
    // Filter data
    const filtered = records.filter(row => parseFloat(row[numericColumn]) > avg);
    console.log(`Found ${filtered.length} records above average`);
    
    // Save results
    const result = stringify([
      ["column", "count", "sum", "average", "min", "max"],
      [numericColumn, records.length, sum, avg, min, max]
    ]);
    
    fs.writeFileSync("statistics.csv", result);
    console.log("Results saved to statistics.csv");
    
    return true;
  } catch (error) {
    console.log(`Error processing data: ${error.message}`);
    return false;
  }
}

main();
