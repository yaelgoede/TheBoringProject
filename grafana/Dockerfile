FROM grafana/grafana:latest
COPY grafana/provisioning /etc/grafana/provisioning

EXPOSE 3000
CMD ["grafana-server", "--homepath=/usr/share/grafana"]