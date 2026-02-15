use anyhow::{Context, Result};
use duckdb::Connection;
use std::path::Path;

pub fn ensure_station_data_loaded(conn: &Connection, data_dir: &str, station_id: &str) -> Result<()> {
    // Check if table exists
    let table_exists: bool = conn.query_row(
        "SELECT count(*) > 0 FROM information_schema.tables WHERE table_name = ?",
        [station_id],
        |row| row.get(0),
    )?;

    if !table_exists {
        let csv_path = format!("{}/{}.csv", data_dir, station_id);
        if Path::new(&csv_path).exists() {
            // DuckDB can read GHCND daily format if it's standard CSV.
            // Assuming standard CSV as per request.
            conn.execute(
                &format!(
                    "CREATE TABLE \"{}\" AS SELECT * FROM read_csv_auto('{}')",
                    station_id, csv_path
                ),
                [],
            ).context(format!("Failed to load CSV for station {}", station_id))?;
        }
    }
    Ok(())
}
