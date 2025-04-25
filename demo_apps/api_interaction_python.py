#!/usr/bin/env python3
"""
API interaction example in Python
This demonstrates making API requests and processing JSON responses
"""

import requests
import json

def main():
    # API endpoint for weather data
    api_url = "https://api.openweathermap.org/data/2.5/weather"
    api_key = "sample_key"  # Replace with actual API key in production
    city = "London"
    
    print(f"Fetching weather data for {city}")
    
    # Construct the full URL with query parameters
    params = {
        "q": city,
        "appid": api_key,
        "units": "metric"
    }
    
    try:
        # Make the API request
        response = requests.get(api_url, params=params)
        
        # Check if request was successful
        if response.status_code != 200:
            print(f"Error: {response.status_code}")
            return False
        
        # Parse JSON response
        data = response.json()
        
        # Extract relevant information
        city_name = data["name"]
        weather = data["weather"][0]
        main = data["main"]
        
        # Format and display the weather information
        print(f"Weather in {city_name}:")
        print(f"  Description: {weather['description']}")
        print(f"  Temperature: {main['temp']}°C")
        print(f"  Feels like: {main['feels_like']}°C")
        print(f"  Humidity: {main['humidity']}%")
        print(f"  Pressure: {main['pressure']} hPa")
        
        # Save the data to a file
        output = f"City: {city_name}\n"
        output += f"Description: {weather['description']}\n"
        output += f"Temperature: {main['temp']}°C\n"
        output += f"Feels like: {main['feels_like']}°C\n"
        output += f"Humidity: {main['humidity']}%\n"
        output += f"Pressure: {main['pressure']} hPa\n"
        
        with open("weather_data.txt", "w") as f:
            f.write(output)
        print("Weather data saved to weather_data.txt")
        
        # Also save the raw JSON for further processing
        with open("weather_data.json", "w") as f:
            json.dump(data, f, indent=2)
        
        return True
        
    except Exception as e:
        print(f"Exception occurred during API request: {e}")
        return False

if __name__ == "__main__":
    main()
