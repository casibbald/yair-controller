## yapp-rs
[![ci](https://github.com/casibbald/yapp/actions/workflows/ci.yml/badge.svg)](https://github.com/casibbald/yapp/actions/workflows/ci.yml)

A kubernetes controller to handle promotions of images for air gapped environments.


## Installation

### CRD
Apply the CRD from [cached file](yaml/doc_crds/crd.yaml), or pipe it from `crdgen` to pickup schema changes:

```sh
cargo run --bin crdgen | kubectl apply -f -
```

### Controller

Install the controller via `helm` by setting your preferred settings. For defaults:

```sh
helm template charts/doc-controller | kubectl apply -f -
kubectl wait --for=condition=available deploy/doc-controller --timeout=30s
kubectl port-forward service/doc-controller 8080:80
```

### Opentelemetry

Build and run with `telemetry` feature, or configure it via `helm`:

```sh
helm template charts/doc-controller --set tracing.enabled=true | kubectl apply -f -
```

This requires an opentelemetry collector in your cluster. [Tempo](https://github.com/grafana/helm-charts/tree/main/charts/tempo) / [opentelemetry-operator](https://github.com/open-telemetry/opentelemetry-helm-charts/tree/main/charts/opentelemetry-operator) / [grafana agent](https://github.com/grafana/helm-charts/tree/main/charts/agent-operator) should all work out of the box. If your collector does not support grpc otlp you need to change the exporter in [`telemetry.rs`](./src/telemetry.rs).

Note that the [images are pushed either with or without the telemetry feature](https://hub.docker.com/r/clux/controller/tags/) depending on whether the tag includes `otel`.

### Metrics

Metrics is available on `/metrics` and a `ServiceMonitor` is configurable from the chart:

```sh
helm template charts/doc-controller --set serviceMonitor.enabled=true | kubectl apply -f -
```

## Running

### Locally

```sh
cargo run
```

or, with optional telemetry:

```sh
OPENTELEMETRY_ENDPOINT_URL=https://0.0.0.0:4317 RUST_LOG=info,kube=trace,controller=debug cargo run --features=telemetry
```

### In-cluster
To develop by building and deploying the image quickly, we recommend using [tilt](https://tilt.dev/), via `tilt up` instead.


------


```sh
cargo yapp-controller start
```

```sh
$ cargo loco start
Finished dev [unoptimized + debuginfo] target(s) in 21.63s
    Running `target/debug/myapp start`

    :
    :
    :

2025-01-30T11:55:45.988707Z  INFO loco_rs::boot: starting loco_rs
2025-01-30T11:55:45.988722Z  INFO loco_rs::config: loading environment from selected_path="config/development.yaml"
2025-01-30T11:55:45.989003Z  INFO loco_rs::boot: initializers loaded initializers=""
2025-01-30T11:55:46.060690Z  INFO loco_rs::controller::app_routes: [GET] /api/metrics
2025-01-30T11:55:46.060729Z  INFO loco_rs::controller::app_routes: [GET] /api/health
2025-01-30T11:55:46.060738Z  INFO loco_rs::controller::app_routes: [GET] /api
2025-01-30T11:55:46.060906Z  INFO loco_rs::controller::app_routes: +middleware name="limit_payload"
2025-01-30T11:55:46.061083Z  INFO loco_rs::controller::app_routes: +middleware name="cors"
2025-01-30T11:55:46.061095Z  INFO loco_rs::controller::app_routes: +middleware name="catch_panic"
2025-01-30T11:55:46.061108Z  INFO loco_rs::controller::app_routes: +middleware name="etag"
2025-01-30T11:55:46.061122Z  INFO loco_rs::controller::app_routes: +middleware name="logger"
2025-01-30T11:55:46.061125Z  INFO loco_rs::controller::app_routes: +middleware name="request_id"
2025-01-30T11:55:46.061131Z  INFO loco_rs::controller::app_routes: +middleware name="fallback"
2025-01-30T11:55:46.061143Z  INFO loco_rs::controller::app_routes: +middleware name="powered_by"

                      ▄     ▀                     
                                 ▀  ▄             
                  ▄       ▀     ▄  ▄ ▄▀           
                                    ▄ ▀▄▄         
                        ▄     ▀    ▀  ▀▄▀█▄       
                                          ▀█▄     
▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄▄ ▀▀█    
 ██████  █████   ███ █████   ███ █████   ███ ▀█   
 ██████  █████   ███ █████   ▀▀▀ █████   ███ ▄█▄  
 ██████  █████   ███ █████       █████   ███ ████▄
 ██████  █████   ███ █████   ▄▄▄ █████   ███ █████
 ██████  █████   ███  ████   ███ █████   ███ ████▀
   ▀▀▀██▄ ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀ ██▀  
       ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀    
                https://loco.rs

environment: development
     logger: debug
compilation: release
      modes: server

listening on http://localhost:8080
```


The following additional logs when in cluster with CRD and resource deployed.
```sh
2025-01-30T12:16:50.329718Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T12:16:50.335134Z  INFO kube_runtime::controller: press ctrl+c to shut down gracefully
2025-01-30T12:16:50.335161Z DEBUG kube_runtime::controller: applier runner held until store is ready
2025-01-30T12:16:50.335307Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=500 otel.name="list" otel.kind="client"
2025-01-30T12:16:50.337248Z DEBUG kube_runtime::controller: store is ready, starting runner
2025-01-30T12:16:50.337298Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&watch=true&timeoutSeconds=290&allowWatchBookmarks=true&resourceVersion=101631 otel.name="watch" otel.kind="client"
2025-01-30T12:16:50.341592Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=samuel namespace=default object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel"
2025-01-30T12:16:50.341666Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T12:16:50.342259Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=lorem namespace=default object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem"
2025-01-30T12:16:50.342315Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem? otel.name="patch" otel.kind="client"
2025-01-30T12:16:50.342680Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=illegal namespace=default object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated document="illegal"
2025-01-30T12:16:50.342716Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated document="illegal" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/illegal? otel.name="patch" otel.kind="client"
2025-01-30T12:16:50.352596Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=lorem namespace=default object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem"
2025-01-30T12:16:50.352645Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T12:16:50.353071Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=illegal namespace=default object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated document="illegal"
2025-01-30T12:16:50.353144Z  WARN reconciling object: yapp::core::kubecontroller: reconcile failed: Any(ApplyFailed(Any(Custom { kind: Other, error: "IllegalDocument" }))) object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated
2025-01-30T12:16:50.359760Z  INFO reconciling object:reconcile: yapp::core::kubecontroller: Reconciling Document document_name=lorem namespace=default object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem"
2025-01-30T12:16:50.359940Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"```
```
