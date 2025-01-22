FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./yapp-controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/yapp-controller"]