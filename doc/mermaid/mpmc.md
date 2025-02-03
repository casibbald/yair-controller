sequenceDiagram
participant WebAPI as Web API (Loco)
participant Broker as Controller Broker
participant Controller as Kube Controller
participant K8s as Kubernetes API

    rect rgba(200,200,200,0.1)
        note right of Broker: Initialization Phase
        Broker->>Broker: Create MPSC channel
        Broker->>Broker: Spawn broker task
        Controller->>Broker: Clone sender
        WebAPI->>Broker: Clone sender
    end

    rect rgba(0,100,200,0.1)
        note right of Controller: Reconciliation Flow
        Controller->>K8s: Watch Pod events
        K8s-->>Controller: Pod change notification
        Controller->>Broker: Send ReconcileEvent(pod_name)
        Broker->>Broker: Update metrics.reconcile_count
        Controller->>K8s: Apply changes (if needed)
    end

    rect rgba(200,0,0,0.1)
        note left of Controller: Error Handling Flow
        Controller->>K8s: API call
        K8s--xController: Error response
        Controller->>Broker: Send ErrorEvent(error)
        Broker->>Broker: Update metrics.error_count
        Broker->>Broker: Store last_error
    end

    rect rgba(0,200,0,0.1)
        note left of WebAPI: Metrics Query Flow
        WebAPI->>Broker: Send GetMetrics + response_channel
        Broker->>WebAPI: Send metrics via response_channel
        WebAPI-->>User: Return JSON metrics
    end

    par Concurrent Operations
        Broker--)Broker: Process ReconcileEvents
        Broker--)Broker: Process ErrorEvents
        Broker--)WebAPI: Handle metrics requests
    end