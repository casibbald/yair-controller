FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./yapp-controller /app/yapp-controller
# Ensure you have a config file under /app/config/[development|production].yaml
EXPOSE 8080
ENTRYPOINT ["/app/yapp-controller", "start"]
