sequenceDiagram
participant WebAPI as Web API
participant Broker as Controller Broker
participant Controller as Kube Controller
participant K8s as Kubernetes API
participant Flux as Flux CD
participant Git as Source of Truth
participant SrcReg as Source Registry
participant DestReg as Destination Registry

    rect rgba(200,200,200,0.1)
        note right of Broker: Initialization Phase
        Broker->>Broker: Create MPSC channel
        Broker->>Broker: Spawn broker task
        Controller->>Broker: Clone sender
        WebAPI->>Broker: Clone sender
    end

    rect rgba(0,100,200,0.1)
        note right of Controller: Reconciliation Flow
        User ->> K8s: Add ReplicationCResource
        User ->> K8s: Add SrcCResource
        User ->> K8s: Add DestCResource
        Controller--) Controller: Create new Replication watcher
        Controller->>K8s: Watch Deployment events
        K8s-->>Controller: Deployment change notification
        Controller->>Broker: Send ReconcileEvent(Deployment_name)
        Broker->>Broker: Update metrics.reconcile_count

        note right of Controller: Image Replication Flow
        Controller->>K8s: Apply changes (Update finalizer)
        Controller-->>SrcReg: Pull image (from Deployment config)
        Controller-->>DestReg: Push image
        Controller-->>K8s: Update ReplicationResource with status and tag
        Controller->>Broker: Send ReplicationEvent (Deployment_name, ImageID)
        Broker->>Broker: Update metrics.replication_count (Deployment_name)
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
