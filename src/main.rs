use serde::Deserialize;
use std::error::Error as StdError;
use std::fs;
use std::io::{self, Write};
use prettytable::{Table, Row, Cell, format};
use crossterm::{execute, terminal::{self, ClearType}, style::{self, Color, StyledContent, Stylize}};
use std::io::stdout;

#[derive(Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
    weather: Vec<WeatherCondition>,
}

#[derive(Deserialize)]
struct Main {
    temp: f64,
    pressure: i32,
    humidity: i32,
}

#[derive(Deserialize)]
struct WeatherCondition {
    description: String,
}

const CONFIG_FILE: &str = "config.txt";

fn get_config() -> (String, String) {
    // Check if the config file exists
    if let Ok(config) = fs::read_to_string(CONFIG_FILE) {
        let mut lines = config.lines();
        let api_key = lines.next().unwrap_or("").to_string();
        let city = lines.next().unwrap_or("").to_string();
        return (api_key, city);
    }

    // If the config file does not exist, prompt the user for the API key and their region/city
    let mut api_key = String::new();
    print!("Please enter your OpenWeatherMap API key: ");
    io::stdout().flush().unwrap(); // check out if the prompt is displayed immediately
    io::stdin().read_line(&mut api_key).expect("Failed to read line");

    let mut city = String::new();
    print!("Please enter your preferred city, district, or state: ");
    io::stdout().flush().unwrap(); // check out if the prompt is displayed immediately x2
    io::stdin().read_line(&mut city).expect("Failed to read line");

    // Save the API key and city to the config file
    fs::write(CONFIG_FILE, format!("{}\n{}", api_key.trim(), city.trim())).expect("Unable to write to config file");

    (api_key.trim().to_string(), city.trim().to_string())
}

fn get_weather_icon(description: &str) -> &str {
    match description.to_lowercase().as_str() {
        "clear sky" => "☀️", // Sunny
        "few clouds" => "🌤️", // Partly cloudy
        "scattered clouds" => "🌥️", // Partly cloudy
        "broken clouds" => "☁️", // Cloudy
        "shower rain" => "🌧️", // Rainy
        "rain" => "🌧️", // Rainy
        "thunderstorm" => "⛈️", // Thunderstorm
        "snow" => "❄️", // Snowy
        "mist" | "haze" => "🌫️", // Haze
        "fog" => "🌫️", // Fog
        "dust" => "🌪️", // Dust
        "sand" => "🌪️", // Sand
        "tornado" => "🌪️", // Tornado
        _ => "🌈", // Default icon for unknown conditions
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Clear the terminal
    execute!(stdout(), terminal::Clear(ClearType::All))?;

    // Get the API key from the config
    let (api_key, default_city) = get_config();

    // Collect command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let city = if args.len() > 1 {
        args[1..].join(" ") // Join all arguments after the first one to handle multi-word city names
    } else {
        default_city
    };

    // Construct the API URL
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    // Make the request
    let response = reqwest::get(&url).await?;

    if response.status().is_success() {
        // Attempt to deserialize the response
        let weather_response: WeatherResponse = response.json().await?;

        // Get the weather icon based on the description
        let weather_icon = get_weather_icon(&weather_response.weather[0].description);

        // Create a fancy table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        
        // Add rows with styled content
        table.add_row(Row::new(vec![
            Cell::new("🌆 City").style_spec("c"),
            Cell::new(&weather_response.name).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("🌡️ Temperature (°C)").style_spec("c"),
            Cell::new(&weather_response.main.temp.to_string()).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("💧 Humidity (%)").style_spec("c"),
            Cell::new(&weather_response.main.humidity.to_string()).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("🌬️ Pressure (hPa)").style_spec("c"),
            Cell::new(&weather_response.main.pressure.to_string()).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("🌤️ Condition").style_spec("c"),
            Cell::new(format!("{} {}", weather_icon, weather_response.weather[0].description).as_str()).style_spec("c"),
        ]));

        // Print the table with a styled header
        println!();
        let title_style = style::style("Weather Information").with(Color::Blue).bold();
        println!("{}", title_style);
        println!();

        // Print the table
        table.printstd();
    } else {
        // Print the error message if the request was not successful
        let error_message: serde_json::Value = response.json().await?;
        println!("❌ Error: {}", error_message);
    }

    Ok(())
}