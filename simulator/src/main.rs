
/// This program is an MQTT simulator that publishes temperature, humidity, and pressure
/// measurements to a specified MQTT broker at regular intervals. The MQTT broker details
/// (host, port, topic) and device ID are provided through environment variables.
///
/// # Environment Variables
/// - `MQTT_HOST`: The hostname of the MQTT broker.
/// - `MQTT_PORT`: The port of the MQTT broker.
/// - `MQTT_TOPIC`: The topic to which the measurements will be published.
/// - `DEVICE_ID`: The ID of the device publishing the measurements.
///
/// # Functionality
/// - Connects to the MQTT broker using the provided host and port.
/// - Publishes temperature, humidity, and pressure measurements to the specified topic.
/// - Increments the measurements by 1 every 10 seconds.
/// - Resets the measurements to 0 when they reach 100.
///
/// # Dependencies
/// - `paho_mqtt`: The Paho MQTT library for Rust.
/// - `std`: Standard library for environment variables, threading, and time handling.
///
/// # Example
/// ```sh
/// export MQTT_HOST="localhost"
/// export MQTT_PORT="1883"
/// export MQTT_TOPIC="sensors"
/// export DEVICE_ID="device123"
/// cargo run
/// ```
///
/// The program will then start publishing measurements to the MQTT broker at the specified
/// intervals.
use paho_mqtt as mqtt;
use std::{env, thread, time::Duration};

fn main() {
    let mqtt_host = env::var("MQTT_HOST").expect("MQTT_HOST must be set");
    let mqtt_port = env::var("MQTT_PORT").expect("MQTT_PORT must be set");
    let mqtt_topic = env::var("MQTT_TOPIC").expect("MQTT_TOPIC must be set");
    let device_id = env::var("DEVICE_ID").expect("DEVICE_ID must be set");

    let client_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(format!("{}:{}", mqtt_host, mqtt_port))
        .client_id("mqtt_simulator")
        .finalize();

    let mqtt_client = mqtt::AsyncClient::new(client_options).expect("Failed to create MQTT client");

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
        .finalize();

    mqtt_client
        .connect(connection_options)
        .wait()
        .expect("Failed to connect");

    println!("Publishing temperature on the mqtt_topic '{}'", mqtt_topic);

    let mut measurements = std::collections::HashMap::new();
    measurements.insert("temperature", 0);
    measurements.insert("humidity", 10);
    measurements.insert("pressure", 20);

    loop {
        measurements
            .iter_mut()
            .for_each(|measurement| *measurement.1 += 1);

        for (key, value) in &measurements {
            let msg = mqtt::Message::new(
                format!("{}/{}/{}", mqtt_topic, device_id, key),
                value.to_string(),
                1,
            );
            match mqtt_client.try_publish(msg) {
                Err(err) => eprintln!("Error creating/queuing the message for {}: {}", key, err),
                Ok(token) => {
                    if let Err(err) = token.wait() {
                        eprintln!("Error sending message for {}: {}", key, err);
                    }
                }
            }
        }

        measurements.iter_mut().for_each(|measurement| {
            if *measurement.1 >= 100 {
                *measurement.1 = 0;
            }
        });

        thread::sleep(Duration::from_secs(10));
    }
}
