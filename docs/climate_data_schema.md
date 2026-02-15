# Climate Data Schema (GHCN Daily)

The `climate_data` tool provides access to historical weather data from the Global Historical Climatology Network (GHCN) Daily dataset. Data is stored in DuckDB, with each weather station having its own table named after its Station ID (e.g., `USW00023272`).

## Core Columns

| Column | Type | Description |
|--------|------|-------------|
| **STATION** | VARCHAR | The station identifier (e.g., "USW00023272"). |
| **DATE** | DATE | The date of the observation. |
| **LATITUDE** | DOUBLE | Latitude of the station. |
| **LONGITUDE** | DOUBLE | Longitude of the station. |
| **ELEVATION** | DOUBLE | Elevation of the station (meters). |
| **NAME** | VARCHAR | Name of the station. |
| **PRCP** | BIGINT | **Precipitation** in tenths of millimeters (e.g., 100 = 10.0 mm). |
| **SNOW** | BIGINT | **Snowfall** in millimeters. |
| **SNWD** | BIGINT | **Snow depth** in millimeters. |
| **TMAX** | BIGINT | **Maximum temperature** in tenths of degrees Celsius (e.g., 255 = 25.5°C). |
| **TMIN** | BIGINT | **Minimum temperature** in tenths of degrees Celsius (e.g., 120 = 12.0°C). |

## Additional Columns

Stations may also contain the following specific data types:

- **TOBS**: Temperature at the time of observation (tenths of °C).
- **TAVG**: Average daily temperature (tenths of °C).
- **AWND**: Average daily wind speed (tenths of meters per second).
- **WSFG**: Peak gust wind speed (tenths of meters per second).
- **WTxx**: Weather Type indicators (WT01 = Fog, WT02 = Heavy Fog, etc.).

## Attributes

Most data columns have a corresponding attribute column (e.g., `PRCP_ATTRIBUTES`) which contains flags for data quality, source, and consistency. These are generally not needed for basic analysis but are available for reference.

## Querying Tips

- **Table Names**: Always use quotes if the station ID starts with numbers or contains special characters (though most are alphanumeric).
- **Units**: Remember to divide `TMAX`, `TMIN`, and `PRCP` by 10 to get standard units (°C and mm).
- **Missing Data**: Missing values are represented as `NULL`.
