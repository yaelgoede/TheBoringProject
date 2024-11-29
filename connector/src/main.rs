/// This is the main entry point for the application. It performs the following tasks:
///
/// 1. Reads environment variables for PostgreSQL and MQTT configurations.
/// 2. Connects to a PostgreSQL database using the provided credentials.
/// 3. Connects to an MQTT broker and subscribes to a specified topic.
/// 4. Listens for incoming MQTT messages on the subscribed topic.
/// 5. Inserts received MQTT messages into the PostgreSQL database.
///
/// # Environment Variables
///
/// - `POSTGRES_HOST`: The hostname of the PostgreSQL server.
/// - `POSTGRES_PORT`: The port number of the PostgreSQL server.
/// - `POSTGRES_USER`: The username for the PostgreSQL database.
/// - `POSTGRES_PASSWORD`: The password for the PostgreSQL database.
/// - `POSTGRES_DB`: The name of the PostgreSQL database.
/// - `MQTT_HOST`: The hostname of the MQTT broker.
/// - `MQTT_PORT`: The port number of the MQTT broker.
/// - `MQTT_TOPIC`: The MQTT topic to subscribe to.
///
/// # Panics
///
/// This function will panic if any of the required environment variables are not set or if there are any errors
/// connecting to the PostgreSQL database or the MQTT broker.
use chrono::format;
use paho_mqtt as mqtt;
use std::env;
use tokio_postgres::{NoTls, Statement};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let database_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
    let database_port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");
    let database_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let database_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let database_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");

    let database_url = format!(
        "host={} port={} user={} password={} dbname={} sslmode=disable",
        database_host, database_port, database_user, database_password, database_name
    );

    println!("Connecting to database at {}", database_url);

    let (db_client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await
        .expect("Failed to connect to the database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let mqtt_host = env::var("MQTT_HOST").expect("MQTT_HOST must be set");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");
    let mqtt_topic = env::var("MQTT_TOPIC").expect("MQTT_TOPIC must be set");

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(format!("{}:{}", mqtt_host, mqtt_port))
        .client_id("mqtt_connector")
        .finalize();

    let mut mqtt_client =
        mqtt::AsyncClient::new(create_opts).expect("Failed to create MQTT client");

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    mqtt_client
        .connect(connection_options)
        .await
        .expect("Failed to connect to MQTT broker");

    mqtt_client
        .subscribe(mqtt_topic.clone(), 1)
        .await
        .expect("Failed to subscribe to MQTT topic");

    let mut stream = mqtt_client.get_stream(25);

    println!("Listening for messages on topic '{}'", mqtt_topic);

    while let Some(msg_option) = stream.next().await {
        if let Some(msg) = msg_option {
            let payload = msg.payload_str();
            println!("Received from topic: {}, message: {}", msg.topic(), payload);

            let topic_parts: Vec<&str> = msg.topic().split('/').collect();
            let device_id = topic_parts
                .get(1)
                .expect("Failed to extract device_id from topic");
            let measurement_name = topic_parts
                .get(2)
                .expect("Failed to extract measurement from topic");

            let statement = format!(
                "INSERT INTO telemetry (device_id, name, value, type) VALUES ('{}', '{}', '{}', 'integer')",
                device_id, measurement_name, payload
            );

            db_client
                .execute(&statement, &[])
                .await
                .expect("Failed to insert message into database");
        }
    }
}
