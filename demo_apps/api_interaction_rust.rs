use reqwest;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::error::Error;

// API interaction example in Rust
// This demonstrates making API requests and processing JSON responses

#[derive(Serialize, Deserialize, Debug)]
struct WeatherResponse {
    name: String,
    weather: Vec<Weather>,
    main: Main,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: i32,
    pressure: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // API endpoint for weather data
    let api_url = "https://api.openweathermap.org/data/2.5/weather";
    let api_key = "sample_key"; // Replace with actual API key in production
    let city = "London";
    
    println!("Fetching weather data for {}", city);
    
    // Construct the full URL with query parameters
    let url = format!("{}?q={}&appid={}&units=metric", api_url, city, api_key);
    
    // Make the API request
    let response = reqwest::get(&url).await?;
    
    // Check if request was successful
    if !response.status().is_success() {
        println!("Error: {}", response.status());
        return Ok(());
    }
    
    // Parse JSON response
    let data: WeatherResponse = response.json().await?;
    
    // Extract relevant information
    let city_name = &data.name;
    let weather = &data.weather[0];
    let main = &data.main;
    
    // Format and display the weather information
    println!("Weather in {}:", city_name);
    println!("  Description: {}", weather.description);
    println!("  Temperature: {}°C", main.temp);
    println!("  Feels like: {}°C", main.feels_like);
    println!("  Humidity: {}%", main.humidity);
    println!("  Pressure: {} hPa", main.pressure);
    
    // Save the data to a file
    let output = format!(
        "City: {}\nDescription: {}\nTemperature: {}°C\nFeels like: {}°C\nHumidity: {}%\nPressure: {} hPa\n",
        city_name, weather.description, main.temp, main.feels_like, main.humidity, main.pressure
    );
    
    let mut file = File::create("weather_data.txt")?;
    file.write_all(output.as_bytes())?;
    println!("Weather data saved to weather_data.txt");
    
    // Also save the raw JSON for further processing
    let json_string = serde_json::to_string_pretty(&data)?;
    let mut json_file = File::create("weather_data.json")?;
    json_file.write_all(json_string.as_bytes())?;
    
    Ok(())
}
