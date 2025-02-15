sequenceDiagram
participant GitHub
participant FluxCD as Flux CD
participant K8s as Kubernetes API
participant Controller as Replicator Controller
participant SrcReg as Source Registry
participant DestReg as Destination Registry
participant WebAPI as Web API
participant Metrics as Metrics Service

    rect rgba(200,200,200,0.1)
        note right of GitHub: Initial Setup
        GitHub->>FluxCD: Push CRs to repository
        FluxCD->>K8s: Apply CRs for Source & Destination Repositories
        FluxCD->>K8s: Apply CR for ContainerReplicator
    end

    rect rgba(0,100,200,0.1)
        note right of Controller: Reconciliation Flow
        Controller->>K8s: Watch for Custom Resource events
        K8s-->>Controller: Notify on CR changes (Source/Destination/Replicator)
        
        loop For each CR change
            Note right of Controller: Reconcile operation begins
            Controller->>+SrcReg: Authenticate & Pull image from Source
            Controller-->>SrcReg: Fetch image metadata
            Controller->>+DestReg: Authenticate to Destination
            Controller-->>DestReg: Push image
            Controller->>K8s: Update CR status with replication results
            Controller->>Metrics: Report reconciliation details
            Controller->>WebAPI: Notify reconciliation success
            WebAPI-->>Controller: Acknowledge successful notification
        end
    end

    rect rgba(150,0,150,0.1)
        note right of Metrics: Comprehensive Metrics Updates
        Controller->>Metrics: Report replication success/failure
        Controller->>Metrics: Report replication latency and throughput
        Controller->>Metrics: Detailed error statistics
    end
    
    rect rgba(200,0,0,0.1)
        note left of Controller: Error Handling
        Controller->>SrcReg: API call failure
        SrcReg--xController: Error response
        Controller->>K8s: Update CR status with error details
        Controller->>Metrics: Report error details
        Controller->>WebAPI: Notify about error
    end

    rect rgba(0,200,0,0.1)
        note left of K8s: Continuous Monitoring
        Controller--) K8s: Monitor CR and deployment changes
        K8s-->>Controller: Trigger reconciliations as needed
        Note right of Controller: Reconciliation triggered by changes
    end
