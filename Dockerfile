FROM debian:bullseye-slim
COPY --chown=nonroot:nonroot ./yapp-controller /app/yapp-controller
COPY --chown=nonroot:nonroot ./config /app/config
# Ensure you have a config file under /app/config/[development|production].yaml
# This needs to be mounted with a volume or copied into the container
WORKDIR /app
ENV ENVIRONMENT=development
EXPOSE 8080
ENTRYPOINT ["./yapp-controller", "start"]
