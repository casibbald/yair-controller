## yair-rs
[![ci](https://github.com/casibbald/yair-controller/actions/workflows/ci.yml/badge.svg)](https://github.com/casibbald/yair-controller/actions/workflows/ci.yml)

A kubernetes controller to handle replication of images between registries for use by kubernetes deployments and orchestrated by FluxCD image automation.


## Installation

### CRD
Apply the CRD from [cached file](yaml/doc_crds/crd.yaml), or pipe it from `crdgen` to pickup schema changes:

```sh
cargo run --bin crdgen | kubectl apply -f -
```

### Controller

Install the controller via `helm` by setting your preferred settings. For defaults:

```sh
helm template charts/yair-controller | kubectl apply -f -
kubectl wait --for=condition=available deploy/yair-controller --timeout=30s
kubectl port-forward service/yair-controller 8080:80
```

### Opentelemetry

Build and run with `telemetry` feature, or configure it via `helm`:

```sh
helm template charts/yair-controller --set tracing.enabled=true | kubectl apply -f -
```

This requires an opentelemetry collector in your cluster. [Tempo](https://github.com/grafana/helm-charts/tree/main/charts/tempo) / [opentelemetry-operator](https://github.com/open-telemetry/opentelemetry-helm-charts/tree/main/charts/opentelemetry-operator) / [grafana agent](https://github.com/grafana/helm-charts/tree/main/charts/agent-operator) should all work out of the box. 

We have taken to using otel-collector as a default, we may choose at a later stage to make this optional or select a secondary option.

### Metrics

Metrics is available on `/metrics` and a `ServiceMonitor` is configurable from the chart:

```sh
helm template charts/yair-controller --set serviceMonitor.enabled=true | kubectl apply -f -
```

## Running

### Locally

```sh
cargo run
```

or, with telemetry when in cluster:

```sh
OPENTELEMETRY_ENDPOINT_URL=https://0.0.0.0:4317 RUST_LOG=info,kube=trace,controller=debug cargo run --features=telemetry
```

### Development workflow and In-cluster testing
We use [tilt](https://tilt.dev/), via `tilt up`, this route provides the fastest workflow to becoming productive as a contributor.


------



```sh
$ tilt up

❯ tilt up
Tilt started on http://localhost:10350/
v0.33.22, built 2025-01-03

(space) to open the browser
(s) to stream logs (--stream=true)
(t) to open legacy terminal mode (--legacy=true)
(ctrl-c) to exit

```

Navigate to the provided URL, and you should see the following logs:
```chrome
Finished dev [unoptimized + debuginfo] target(s) in 21.63s
    Running `target/debug/myair start`

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
2025-01-30T21:41:21.469029Z  INFO yair::core::telemetry: Global default subscriber already set!
2025-01-30T21:41:21.470283Z DEBUG tower::buffer::worker: service.ready=true processing request
2025-01-30T21:41:21.470534Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.470728Z DEBUG HTTP: hyper_util::client::legacy::connect::http: connecting to 10.96.0.1:443 http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.470964Z DEBUG HTTP: hyper_util::client::legacy::connect::http: connected to 10.96.0.1:443 http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.471017Z DEBUG HTTP: rustls::client::hs: No cached session for IpAddress(V4(Ipv4Addr([10, 96, 0, 1])))     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.471202Z DEBUG HTTP: rustls::client::hs: Not resuming any session     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.472946Z DEBUG HTTP: rustls::client::hs: Using ciphersuite TLS13_AES_128_GCM_SHA256     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.472994Z DEBUG HTTP: rustls::client::tls13: Not resuming     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.473164Z DEBUG HTTP: rustls::client::tls13: TLS1.3 encrypted extensions: []     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.473185Z DEBUG HTTP: rustls::client::hs: ALPN protocol is None     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.473212Z DEBUG HTTP: rustls::client::tls13: Got CertificateRequest CertificateRequestPayloadTls13 { context: , extensions: [Unknown(UnknownExtension { typ: StatusRequest, payload:  }), Unknown(UnknownExtension { typ: SCT, payload:  }), SignatureAlgorithms([RSA_PSS_SHA256, ECDSA_NISTP256_SHA256, ED25519, RSA_PSS_SHA384, RSA_PSS_SHA512, RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, ECDSA_NISTP384_SHA384, ECDSA_NISTP521_SHA512, RSA_PKCS1_SHA1, ECDSA_SHA1_Legacy]), AuthorityNames([DistinguishedName(3015311330110603550403130a6b756265726e65746573), DistinguishedName(3019311730150603550403130e66726f6e742d70726f78792d6361)])] }     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.473251Z DEBUG HTTP: rustls::client::common: Client auth requested but no cert/sigscheme available     http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.476735Z DEBUG HTTP: hyper_util::client::legacy::pool: pooling idle connection for ("https", 10.96.0.1) http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=1 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.477136Z  INFO kube_runtime::controller: press ctrl+c to shut down gracefully
2025-01-30T21:41:21.477151Z DEBUG kube_runtime::controller: applier runner held until store is ready
2025-01-30T21:41:21.477258Z DEBUG tower::buffer::worker: service.ready=true processing request
2025-01-30T21:41:21.477334Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=500 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.477357Z DEBUG HTTP: hyper_util::client::legacy::pool: reuse idle connection for ("https", 10.96.0.1) http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&limit=500 otel.name="list" otel.kind="client"
2025-01-30T21:41:21.478831Z DEBUG hyper_util::client::legacy::pool: pooling idle connection for ("https", 10.96.0.1)
2025-01-30T21:41:21.479075Z DEBUG kube_runtime::controller: store is ready, starting runner
2025-01-30T21:41:21.479122Z DEBUG tower::buffer::worker: service.ready=true processing request
2025-01-30T21:41:21.479212Z DEBUG HTTP: kube_client::client::builder: requesting http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&watch=true&timeoutSeconds=290&allowWatchBookmarks=true&resourceVersion=140817 otel.name="watch" otel.kind="client"
2025-01-30T21:41:21.479230Z DEBUG HTTP: hyper_util::client::legacy::pool: reuse idle connection for ("https", 10.96.0.1) http.method=GET http.url=https://10.96.0.1/apis/kube.rs/v1/documents?&watch=true&timeoutSeconds=290&allowWatchBookmarks=true&resourceVersion=140817 otel.name="watch" otel.kind="client"
2025-01-30T21:41:21.481390Z  INFO reconciling object:reconcile: yair::core::kubecontroller: Reconciling Document document_name=test namespace=default object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test"
2025-01-30T21:41:21.481482Z DEBUG reconciling object:reconcile: tower::buffer::worker: service.ready=true processing request object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test"
2025-01-30T21:41:21.481506Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.481558Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connecting to 10.96.0.1:443 object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.482076Z  INFO reconciling object:reconcile: yair::core::kubecontroller: Reconciling Document document_name=samuel namespace=default object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel"
2025-01-30T21:41:21.482101Z DEBUG reconciling object:reconcile: tower::buffer::worker: service.ready=true processing request object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel"
2025-01-30T21:41:21.482140Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.482161Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connecting to 10.96.0.1:443 object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.482605Z  INFO reconciling object:reconcile: yair::core::kubecontroller: Reconciling Document document_name=lorem namespace=default object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem"
2025-01-30T21:41:21.482630Z DEBUG reconciling object:reconcile: tower::buffer::worker: service.ready=true processing request object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem"
2025-01-30T21:41:21.482639Z DEBUG reconciling object:reconcile:HTTP: kube_client::client::builder: requesting object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.482655Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connecting to 10.96.0.1:443 object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483050Z  INFO reconciling object:reconcile: yair::core::kubecontroller: Reconciling Document document_name=illegal namespace=default object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated document="illegal"
2025-01-30T21:41:21.483104Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connected to 10.96.0.1:443 object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483171Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Resuming session     object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483378Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connected to 10.96.0.1:443 object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483430Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: No cached session for IpAddress(V4(Ipv4Addr([10, 96, 0, 1])))     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483547Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Not resuming any session     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483705Z  WARN reconciling object: yair::core::kubecontroller: reconcile failed: Any(ApplyFailed(Any(Custom { kind: Other, error: "IllegalDocument" }))) object.ref=Document.v1.kube.rs/illegal.default object.reason=object updated
2025-01-30T21:41:21.483712Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::connect::http: connected to 10.96.0.1:443 object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.483727Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: No cached session for IpAddress(V4(Ipv4Addr([10, 96, 0, 1])))     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.484060Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Not resuming any session     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.484123Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Using ciphersuite TLS13_AES_128_GCM_SHA256     object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.484141Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: Resuming using PSK     object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.484185Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: TLS1.3 encrypted extensions: []     object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.484188Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: ALPN protocol is None     object.ref=Document.v1.kube.rs/lorem.default object.reason=object updated document="lorem" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/lorem/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485386Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Using ciphersuite TLS13_AES_128_GCM_SHA256     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485404Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: Not resuming     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485471Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: TLS1.3 encrypted extensions: []     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485476Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: ALPN protocol is None     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485482Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: Got CertificateRequest CertificateRequestPayloadTls13 { context: , extensions: [Unknown(UnknownExtension { typ: StatusRequest, payload:  }), Unknown(UnknownExtension { typ: SCT, payload:  }), SignatureAlgorithms([RSA_PSS_SHA256, ECDSA_NISTP256_SHA256, ED25519, RSA_PSS_SHA384, RSA_PSS_SHA512, RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, ECDSA_NISTP384_SHA384, ECDSA_NISTP521_SHA512, RSA_PKCS1_SHA1, ECDSA_SHA1_Legacy]), AuthorityNames([DistinguishedName(3015311330110603550403130a6b756265726e65746573), DistinguishedName(3019311730150603550403130e66726f6e742d70726f78792d6361)])] }     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485490Z DEBUG reconciling object:reconcile:HTTP: rustls::client::common: Client auth requested but no cert/sigscheme available     object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485942Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: Using ciphersuite TLS13_AES_128_GCM_SHA256     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.485954Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: Not resuming     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.486020Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: TLS1.3 encrypted extensions: []     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.486029Z DEBUG reconciling object:reconcile:HTTP: rustls::client::hs: ALPN protocol is None     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.486034Z DEBUG reconciling object:reconcile:HTTP: rustls::client::tls13: Got CertificateRequest CertificateRequestPayloadTls13 { context: , extensions: [Unknown(UnknownExtension { typ: StatusRequest, payload:  }), Unknown(UnknownExtension { typ: SCT, payload:  }), SignatureAlgorithms([RSA_PSS_SHA256, ECDSA_NISTP256_SHA256, ED25519, RSA_PSS_SHA384, RSA_PSS_SHA512, RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, ECDSA_NISTP384_SHA384, ECDSA_NISTP521_SHA512, RSA_PKCS1_SHA1, ECDSA_SHA1_Legacy]), AuthorityNames([DistinguishedName(3015311330110603550403130a6b756265726e65746573), DistinguishedName(3019311730150603550403130e66726f6e742d70726f78792d6361)])] }     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.486040Z DEBUG reconciling object:reconcile:HTTP: rustls::client::common: Client auth requested but no cert/sigscheme available     object.ref=Document.v1.kube.rs/samuel.default object.reason=object updated document="samuel" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/samuel/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.487226Z DEBUG hyper_util::client::legacy::pool: pooling idle connection for ("https", 10.96.0.1)
2025-01-30T21:41:21.488217Z DEBUG reconciling object:reconcile:HTTP: hyper_util::client::legacy::pool: pooling idle connection for ("https", 10.96.0.1) object.ref=Document.v1.kube.rs/test.default object.reason=object updated document="test" http.method=PATCH http.url=https://10.96.0.1/apis/kube.rs/v1/namespaces/default/documents/test/status?&force=true&fieldManager=cntrlr otel.name="patch_status" otel.kind="client"
2025-01-30T21:41:21.488245Z DEBUG hyper_util::client::legacy::pool: pooling idle connection for ("https", 10.96.0.1)```
