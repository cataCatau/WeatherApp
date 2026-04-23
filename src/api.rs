use serde::Deserialize;

pub type FullWeatherData = (CurrentData, HourlyData, DailyForecast, CurrentAqi);

#[derive(Deserialize, Debug)]
pub struct GeoSearch {
    pub results: Option<Vec<GeoLocation>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub current: CurrentData,
    pub hourly: HourlyData,
    pub daily: DailyForecast,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct AirQualityResponse {
    pub current: CurrentAqi,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CurrentAqi {
    pub european_aqi: f32,
    pub pm2_5: f32,
    pub carbon_monoxide: f32,
    pub nitrogen_dioxide: f32,
    pub sulphur_dioxide: f32,
    pub ozone: f32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct CurrentData {
    pub temperature_2m: f32,
    pub weather_code: u32,
    pub surface_pressure: f32,
    pub wind_speed_10m: f32,
    pub precipitation: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DailyForecast {
    pub time: Vec<String>,
    pub temperature_2m_min: Vec<f32>,
    pub temperature_2m_max: Vec<f32>,
    pub uv_index_max: Vec<f32>,
    pub sunrise: Vec<String>,
}

pub async fn fetch_weather(city: String) -> Result<FullWeatherData, String> {
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ro&format=json",
        city
    );
    let geo_resp = match reqwest::get(&geo_url).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("GEO network error: {}", e)),
    };
    let geo_data = match geo_resp.json::<GeoSearch>().await {
        Ok(data) => data,
        Err(e) => return Err(format!("JSON parsing error: {}", e)),
    };
    let results = match geo_data.results {
        Some(r) => r,
        None => return Err("No results received".to_string()),
    };
    let loc = match results.first() {
        Some(l) => l.clone(),
        None => return Err("City not found".to_string()),
    };

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,surface_pressure,wind_speed_10m,precipitation&daily=temperature_2m_max,temperature_2m_min,uv_index_max,sunrise&timezone=auto&forecast_days=7&hourly=temperature_2m",
        loc.latitude, loc.longitude
    );
    let poluation_url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=european_aqi,pm2_5,carbon_monoxide,nitrogen_dioxide,sulphur_dioxide,ozone",
        loc.latitude, loc.longitude
    );

    let w_task = async {
        let w_resp = match reqwest::get(&weather_url).await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Weather network error: {}", e)),
        };
        match w_resp.json::<WeatherResponse>().await {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Weather JSON parsing error: {}", e)),
        }
    };

    let aqi_task = async {
        let aqi_resp = match reqwest::get(&poluation_url).await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("AQI Network error: {}", e)),
        };
        match aqi_resp.json::<AirQualityResponse>().await {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("AQI JSON Parsing error:  {}", e)),
        }
    };

    let (w_result, aqi_result) = futures::join!(w_task, aqi_task);

    let w_data = match w_result {
        Ok(data) => data,
        Err(e) => return Err(e),
    };
    let aqi_data = match aqi_result {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    Ok((
        w_data.current,
        w_data.hourly,
        w_data.daily,
        aqi_data.current,
    ))
}
