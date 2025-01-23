FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./config/development.yaml /app/config/development.yaml
COPY --chown=nonroot:nonroot ./yapp-controller /app/yapp-controller
EXPOSE 8080
ENTRYPOINT ["/app/yapp-controller", "start"]
