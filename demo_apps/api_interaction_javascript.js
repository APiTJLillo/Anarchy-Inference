// API interaction example in JavaScript
// This demonstrates making API requests and processing JSON responses

const fetch = require('node-fetch');
const fs = require('fs');

async function main() {
  // API endpoint for weather data
  const apiUrl = "https://api.openweathermap.org/data/2.5/weather";
  const apiKey = "sample_key"; // Replace with actual API key in production
  const city = "London";
  
  console.log(`Fetching weather data for ${city}`);
  
  // Construct the full URL with query parameters
  const params = new URLSearchParams({
    q: city,
    appid: apiKey,
    units: "metric"
  });
  
  try {
    // Make the API request
    const response = await fetch(`${apiUrl}?${params}`);
    
    // Check if request was successful
    if (response.status !== 200) {
      console.log(`Error: ${response.status}`);
      return false;
    }
    
    // Parse JSON response
    const data = await response.json();
    
    // Extract relevant information
    const cityName = data.name;
    const weather = data.weather[0];
    const main = data.main;
    
    // Format and display the weather information
    console.log(`Weather in ${cityName}:`);
    console.log(`  Description: ${weather.description}`);
    console.log(`  Temperature: ${main.temp}°C`);
    console.log(`  Feels like: ${main.feels_like}°C`);
    console.log(`  Humidity: ${main.humidity}%`);
    console.log(`  Pressure: ${main.pressure} hPa`);
    
    // Save the data to a file
    const output = `City: ${cityName}\n` +
                   `Description: ${weather.description}\n` +
                   `Temperature: ${main.temp}°C\n` +
                   `Feels like: ${main.feels_like}°C\n` +
                   `Humidity: ${main.humidity}%\n` +
                   `Pressure: ${main.pressure} hPa\n`;
    
    fs.writeFileSync("weather_data.txt", output);
    console.log("Weather data saved to weather_data.txt");
    
    // Also save the raw JSON for further processing
    fs.writeFileSync("weather_data.json", JSON.stringify(data, null, 2));
    
    return true;
    
  } catch (error) {
    console.log(`Exception occurred during API request: ${error.message}`);
    return false;
  }
}

main().catch(console.error);
