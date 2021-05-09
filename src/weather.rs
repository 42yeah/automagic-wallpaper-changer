use std::{error::Error, fmt::Display, io::Read};
use serde::{Serialize, Deserialize};

const OPENWEATHER_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Serialize, Deserialize)]
pub struct ActualWeather {
    main: String
}

#[derive(Serialize, Deserialize)]
pub struct WeatherData {
    weather: Vec<ActualWeather>
}

#[derive(Debug)]
pub struct WeatherError();

impl Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to get weather")
    }
}

impl Error for WeatherError {}

pub fn get_weather(key: &str, city: &str) -> Result<String, Box<dyn Error>> {
    let mut response = reqwest::blocking::get(format!(
        "{}?q={}&appid={}",
        OPENWEATHER_URL,
        &city,
        &key
    ))?;
    let mut data = String::new();
    response.read_to_string(&mut data)?;
    let weather_data: WeatherData = serde_json::from_str(&data)?;
    if let Some(weather) = weather_data.weather.get(0) {
        return Ok(weather.main.clone());
    }
    Err(Box::new(WeatherError()))
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use super::*;

    #[test]
    #[ignore = "I think weather might change, you know"]
    fn test_weather() {
        let config = Config::from_path("config.json").unwrap();
        assert_eq!(get_weather(
            &config.openweather_access_key.unwrap(),
            &config.city_weather).unwrap(), "Clear");
    }
}
