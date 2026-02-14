use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::warn;
use super::Tool;

pub struct GeocodingTool;

#[async_trait]
impl Tool for GeocodingTool {
    fn name(&self) -> &str {
        "geocoding"
    }

    fn description(&self) -> &str {
        "Find the latitude and longitude of a location using the Google Maps Geocoding API."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "The address or location to look up."
                }
            },
            "required": ["address"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let address = args["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'address' argument"))?;

        let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
            .map_err(|_| anyhow::anyhow!("GOOGLE_MAPS_API_KEY environment variable not set"))?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://maps.googleapis.com/maps/api/geocode/json?address={}&key={}",
            urlencoding::encode(address),
            api_key
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("Google Maps API error: {}", response.status());
            return Err(anyhow::anyhow!("Google Maps API error: {}", response.status()));
        }

        let json: Value = response.json().await?;

        if let Some(status) = json["status"].as_str() {
            if status != "OK" {
                if let Some(error_message) = json["error_message"].as_str() {
                     return Err(anyhow::anyhow!("Google Maps API error: {} - {}", status, error_message));
                }
                return Err(anyhow::anyhow!("Google Maps API error: {}", status));
            }
        }

        if let Some(results) = json["results"].as_array() {
            if let Some(first_result) = results.first() {
                if let Some(geometry) = first_result["geometry"].as_object() {
                    if let Some(location) = geometry["location"].as_object() {
                        let lat = location["lat"].as_f64().unwrap_or(0.0);
                        let lng = location["lng"].as_f64().unwrap_or(0.0);
                        let formatted_address = first_result["formatted_address"].as_str().unwrap_or(address);
                        
                        return Ok(format!(
                            "Location: {}\nLatitude: {}\nLongitude: {}",
                            formatted_address, lat, lng
                        ));
                    }
                }
            }
        }

        Ok("No results found.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geocoding_tool_metadata() {
        let tool = GeocodingTool;
        assert_eq!(tool.name(), "geocoding");
        assert!(tool.description().contains("Google Maps Geocoding API"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["address"].is_object());
    }
}
