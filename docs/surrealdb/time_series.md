# SurrealDB Time Series Example

This example demonstrates how to use SurrealDB for time-series data analysis with the SurrealKV storage engine. SurrealDB provides native capabilities for time-series data, making it an excellent choice for IoT, monitoring, or any application that deals with time-ordered measurements.

## Entity Definitions

```rust
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use crate::db::{
    DatabaseClient, DatabaseConfig, StorageEngine, Error, Result, Entity,
    connect_database
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

// Helper function for timestamps
fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

// Base entity with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaseEntity {
    /// Entity ID
    pub id: Option<String>,
    /// Creation timestamp
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            created_at: utc_now(),
        }
    }
}

// Define a sensor reading entity for time-series data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorReading {
    #[serde(flatten)]
    base: BaseEntity,
    sensor_id: String,
    temperature: f32,
    humidity: f32,
    pressure: f32,
    timestamp: DateTime<Utc>,
}

impl Entity for SensorReading {
    fn table_name() -> &'static str {
        "sensor_readings"
    }

    fn id(&self) -> Option<String> {
        self.base.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.base.id = Some(id);
    }

    fn generate_id() -> String {
        format!("{}:{}", Self::table_name(), Uuid::new_v4())
    }
}

impl SensorReading {
    fn new(
        sensor_id: impl Into<String>,
        temperature: f32,
        humidity: f32,
        pressure: f32,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            base: BaseEntity::new(),
            sensor_id: sensor_id.into(),
            temperature,
            humidity,
            pressure,
            timestamp: timestamp.unwrap_or_else(utc_now),
        }
    }
}

// Domain-specific stream types for time series operations
struct ReadingStream {
    rx: mpsc::Receiver<Result<SensorReading>>,
    _handle: JoinHandle<()>,
}

struct ReadingsStream {
    rx: mpsc::Receiver<Result<Vec<SensorReading>>>,
    _handle: JoinHandle<()>,
}

struct SetupStream {
    rx: mpsc::Receiver<Result<()>>,
    _handle: JoinHandle<()>,
}

struct HourlyAverageStream {
    rx: mpsc::Receiver<Result<Vec<(String, f32)>>>,
    _handle: JoinHandle<()>,
}

struct RollingAverageStream {
    rx: mpsc::Receiver<Result<Vec<(DateTime<Utc>, f32)>>>,
    _handle: JoinHandle<()>,
}

// Implementation for stream types
impl ReadingStream {
    async fn get(mut self) -> Result<SensorReading> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl ReadingsStream {
    async fn get(mut self) -> Result<Vec<SensorReading>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl SetupStream {
    async fn get(mut self) -> Result<()> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl HourlyAverageStream {
    async fn get(mut self) -> Result<Vec<(String, f32)>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl RollingAverageStream {
    async fn get(mut self) -> Result<Vec<(DateTime<Utc>, f32)>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

// Time series operations manager
struct TimeSeriesOps {
    client: DatabaseClient,
}

impl TimeSeriesOps {
    fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    // Set up time-series table (synchronous interface)
    fn setup_time_series_table(&self) -> SetupStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                println!("Setting up time-series table...");

                // Define table with time-series optimizations
                let setup_query = r#"
                DEFINE TABLE sensor_readings SCHEMAFULL;
                DEFINE FIELD sensor_id ON sensor_readings TYPE string;
                DEFINE FIELD temperature ON sensor_readings TYPE float;
                DEFINE FIELD humidity ON sensor_readings TYPE float;
                DEFINE FIELD pressure ON sensor_readings TYPE float;
                DEFINE FIELD timestamp ON sensor_readings TYPE datetime;
                DEFINE FIELD created_at ON sensor_readings TYPE datetime;

                -- Define indexes for time-series queries
                DEFINE INDEX idx_sensor_time ON sensor_readings COLUMNS sensor_id, timestamp;
                DEFINE INDEX idx_timestamp ON sensor_readings COLUMNS timestamp;
                DEFINE INDEX idx_temperature ON sensor_readings COLUMNS temperature;
                "#;

                client.query::<serde_json::Value>(setup_query).await?;
                println!("Time-series table setup complete");
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Insert sample data (synchronous interface)
    fn insert_sample_data(&self) -> SetupStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                println!("Inserting sample time-series data...");

                // Generate sample data for two sensors over 24 hours
                // with readings every hour
                let num_sensors = 2;
                let hours = 24;
                let readings_per_hour = 4; // 15-minute intervals

                for sensor_idx in 0..num_sensors {
                    let sensor_id = format!("sensor-{:03}", sensor_idx + 1);

                    for hour in 0..hours {
                        // Create a base time for this hour
                        let now = Utc::now();
                        let base_time = now - chrono::Duration::hours(hours - hour);

                        for minute in 0..readings_per_hour {
                            // Add some randomness to timestamps within the hour
                            let timestamp = base_time + chrono::Duration::minutes(15 * minute);

                            // Generate somewhat realistic sensor data with daily patterns
                            // Temperature increases during the day, decreases at night
                            let time_factor = (hour as f32 / 24.0) * 2.0 * std::f32::consts::PI;
                            let temp_base = 20.0 + 8.0 * f32::sin(time_factor - std::f32::consts::PI / 2.0);
                            let humidity_base = 50.0 - 10.0 * f32::sin(time_factor);
                            let pressure_base = 1013.0 + 5.0 * f32::sin(time_factor / 2.0);

                            // Add some noise
                            let temp_noise = (sensor_idx as f32 * 0.3) + (rand::random::<f32>() * 2.0 - 1.0);
                            let humidity_noise = rand::random::<f32>() * 5.0 - 2.5;
                            let pressure_noise = rand::random::<f32>() * 3.0 - 1.5;

                            // Create the reading
                            let reading = SensorReading::new(
                                sensor_id.clone(),
                                temp_base + temp_noise,
                                humidity_base + humidity_noise,
                                pressure_base + pressure_noise,
                                Some(timestamp),
                            );

                            // Insert the reading
                            self.create_sensor_reading(&client, reading).await?;
                        }
                    }
                }

                println!("Sample time-series data inserted");
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Create sensor reading (internal async method)
    async fn create_sensor_reading(&self, client: &DatabaseClient, mut reading: SensorReading) -> Result<SensorReading> {
        let id = SensorReading::generate_id();
        reading.set_id(id);

        let created: SensorReading = client.create(SensorReading::table_name(), &reading).await?;
        Ok(created)
    }

    // Get latest readings (synchronous interface)
    fn get_latest_readings(&self) -> ReadingsStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let query = r#"
                SELECT * FROM sensor_readings GROUP BY sensor_id FETCH timestamp ORDER BY timestamp DESC LIMIT 1;
                "#;

                let results: Vec<SensorReading> = client.query(query).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        ReadingsStream { rx, _handle: handle }
    }

    // Calculate hourly averages (synchronous interface)
    fn calculate_hourly_averages(&self, sensor_id: &str) -> HourlyAverageStream {
        let client = self.client.clone();
        let sensor_id = sensor_id.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let query = format!(
                    "SELECT
                        time::hour(timestamp) + ':00' AS hour,
                        math::mean(temperature) AS avg_temperature
                    FROM sensor_readings
                    WHERE sensor_id = '{sensor_id}'
                    GROUP BY time::hour(timestamp)
                    ORDER BY hour",
                    sensor_id = sensor_id
                );

                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut hourly_avgs = Vec::new();
                for result in results {
                    if let (Some(hour), Some(avg_temp)) = (
                        result.get("hour").and_then(|h| h.as_str()),
                        result.get("avg_temperature").and_then(|t| t.as_f64())
                    ) {
                        hourly_avgs.push((hour.to_string(), avg_temp as f32));
                    }
                }

                Ok(hourly_avgs)
            }.await;

            let _ = tx.send(result).await;
        });

        HourlyAverageStream { rx, _handle: handle }
    }

    // Detect temperature spikes (synchronous interface)
    fn detect_temperature_spikes(&self, threshold: f32) -> ReadingsStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let query = format!(
                    "SELECT * FROM sensor_readings
                    WHERE temperature > {threshold}
                    ORDER BY timestamp DESC",
                    threshold = threshold
                );

                let results: Vec<SensorReading> = client.query(&query).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        ReadingsStream { rx, _handle: handle }
    }

    // Calculate rolling window average (synchronous interface)
    fn rolling_window_average(&self, sensor_id: &str, window_size: usize) -> RollingAverageStream {
        let client = self.client.clone();
        let sensor_id = sensor_id.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let query = format!(
                    "SELECT
                        timestamp,
                        math::mean(temperature)
                            OVER (PARTITION BY sensor_id ORDER BY timestamp RANGE {w} PRECEDING)
                            AS rolling_avg_temp
                    FROM sensor_readings
                    WHERE sensor_id = '{sensor_id}'
                    ORDER BY timestamp",
                    sensor_id = sensor_id,
                    w = window_size
                );

                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut rolling_avgs = Vec::new();
                for result in results {
                    if let (Some(ts_str), Some(avg_temp)) = (
                        result.get("timestamp").and_then(|t| t.as_str()),
                        result.get("rolling_avg_temp").and_then(|t| t.as_f64())
                    ) {
                        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                            rolling_avgs.push((timestamp.with_timezone(&Utc), avg_temp as f32));
                        }
                    }
                }

                Ok(rolling_avgs)
            }.await;

            let _ = tx.send(result).await;
        });

        RollingAverageStream { rx, _handle: handle }
    }
}
```

## Main Program (Using Synchronous Interface with Hidden Async)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    println!("SurrealDB Time Series Example\n");

    // Set up the database configuration for SurrealKV
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/time_series_db".to_string(),
        namespace: "demo".to_string(),
        database: "time_series".to_string(),
        check_migrations: false,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;

    // Create our time series operations manager
    let time_series_ops = TimeSeriesOps::new(client);

    // Set up time-series tables
    time_series_ops.setup_time_series_table().get().await?;

    // Insert sample time-series data
    time_series_ops.insert_sample_data().get().await?;

    // Query the latest reading for each sensor
    println!("\nLatest reading for each sensor:");
    let latest_readings = time_series_ops.get_latest_readings().get().await?;
    for reading in latest_readings {
        println!(
            "Sensor: {}, Temp: {:.1}°C, Humidity: {:.1}%, Pressure: {:.1} hPa, Time: {}",
            reading.sensor_id,
            reading.temperature,
            reading.humidity,
            reading.pressure,
            reading.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    // Calculate hourly averages
    println!("\nHourly temperature averages for sensor-001:");
    let hourly_avgs = time_series_ops.calculate_hourly_averages("sensor-001").get().await?;
    for (hour, avg_temp) in hourly_avgs {
        println!("Hour: {}, Avg Temperature: {:.1}°C", hour, avg_temp);
    }

    // Detect temperature spikes
    println!("\nTemperature spikes (>30°C):");
    let spikes = time_series_ops.detect_temperature_spikes(30.0).get().await?;
    for reading in spikes {
        println!(
            "Spike detected - Sensor: {}, Temp: {:.1}°C, Time: {}",
            reading.sensor_id,
            reading.temperature,
            reading.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    // Rolling window average
    println!("\nRolling average (window: 5 readings) for sensor-001:");
    let rolling_avgs = time_series_ops.rolling_window_average("sensor-001", 5).get().await?;
    for (timestamp, avg_temp) in rolling_avgs {
        println!(
            "Time: {}, Rolling Avg Temp: {:.1}°C",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            avg_temp
        );
    }

    println!("\nExample completed");
    Ok(())
}
```
