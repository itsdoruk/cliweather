use serde::Deserialize;
use std::error::Error as StdError;
use std::fs;
use std::io::{self, Write};
use prettytable::{Table, Row, Cell, format};

#[derive(Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Deserialize)]
struct Main {
    temp: f64,
    pressure: i32,
    humidity: i32,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
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

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        
        table.add_row(Row::new(vec![
            Cell::new("city").style_spec("c"),
            Cell::new(&weather_response.name).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("temperature (°C)").style_spec("c"),
            Cell::new(&weather_response.main.temp.to_string()).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("pressure (hPa)").style_spec("c"),
            Cell::new(&weather_response.main.pressure.to_string()).style_spec("c"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("humidity (%)").style_spec("c"),
            Cell::new(&weather_response.main.humidity.to_string()).style_spec("c"),
        ]));

        // Print the table
        table.printstd();
    } else {
        // Print the error message if the request was not successful
        let error_message: serde_json::Value = response.json().await?;
        println!("error: {}", error_message);
    }

    Ok(())
}

