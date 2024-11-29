FROM eclipse-mosquitto:latest

COPY mqttserver/mosquitto.conf /mosquitto/config/mosquitto.conf

EXPOSE 1883
CMD ["mosquitto", "-c", "/mosquitto/config/mosquitto.conf"]