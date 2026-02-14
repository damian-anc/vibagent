use anyhow::Result;
use async_trait::async_trait;
use duckdb::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use super::Tool;

pub struct StationLookupTool {
    db_path: String,
}

impl StationLookupTool {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }
}

#[derive(Deserialize)]
struct LookupArgs {
    lat: f64,
    lon: f64,
}

#[derive(serde::Serialize)]
struct StationResult {
    id: String,
    name: String,
    lat: f64,
    lon: f64,
    distance_meters: f64,
}

#[async_trait]
impl Tool for StationLookupTool {
    fn name(&self) -> &str {
        "station_lookup"
    }

    fn description(&self) -> &str {
        "Look up the 3 weather stations closest to a given latitude and longitude."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "lat": {
                    "type": "number",
                    "description": "Latitude of the location."
                },
                "lon": {
                    "type": "number",
                    "description": "Longitude of the location."
                }
            },
            "required": ["lat", "lon"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: LookupArgs = serde_json::from_str(arguments)?;

        let conn = Connection::open(&self.db_path)?;
        
        // Load spatial extension
        conn.execute("LOAD spatial;", [])?;

        let mut stmt = conn.prepare(
            "SELECT id, name, lat, lon, ST_Distance_Sphere(geom, ST_Point(?, ?)) as distance \
             FROM ghcnd_stations \
             ORDER BY distance ASC \
             LIMIT 3"
        )?;

        let station_iter = stmt.query_map([args.lon, args.lat], |row| {
            Ok(StationResult {
                id: row.get(0)?,
                name: row.get(1)?,
                lat: row.get(2)?,
                lon: row.get(3)?,
                distance_meters: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for station in station_iter {
            results.push(station?);
        }

        Ok(serde_json::to_string_pretty(&results)?)
    }
}
