import duckdb
import pandas as pd

# Path to your stations file
stations_file = 'data/ghcnd-stations.txt'

# Column names and fixed widths
col_specs = [(0,11), (12,20), (21,30), (31,37), (38,40), (41,71), (72,75), (76,79), (80,85)]
col_names = ['id', 'lat', 'lon', 'elev', 'state', 'name', 'gsn_flag', 'hcn_crn_flag', 'wmo_id']

# Read the fixed-width file into pandas
stations_df = pd.read_fwf(stations_file, colspecs=col_specs, names=col_names)

# Clean up whitespace
stations_df = stations_df.map(lambda x: x.strip() if isinstance(x, str) else x)

# Optional: convert numeric columns
stations_df['lat'] = pd.to_numeric(stations_df['lat'], errors='coerce')
stations_df['lon'] = pd.to_numeric(stations_df['lon'], errors='coerce')
stations_df['elev'] = pd.to_numeric(stations_df['elev'], errors='coerce')

# Connect to DuckDB and create .db
con = duckdb.connect('data/ghcnd_stations.db')

# Create table and insert data
con.execute("DROP TABLE IF EXISTS ghcnd_stations")
con.execute("CREATE TABLE ghcnd_stations AS SELECT * FROM stations_df")

# Optional: create an index on lat/lon for faster nearest-neighbor queries
# DuckDB supports functional indexing for simple use:
con.execute("CREATE INDEX idx_lat_lon ON ghcnd_stations(lat, lon)")

# Close the connection
con.close()

print("GHCN stations imported successfully into ghcnd_stations.db")
