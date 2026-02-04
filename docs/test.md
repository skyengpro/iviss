
```mermaid
graph TB
    subgraph "Client Layer"
        A[Android App<br/>Agent Field Device]
        B[Web Back-Office<br/>React SPA]
    end
  
    subgraph "Network Boundary"
        R[Router / Firewall<br/>Public IP: X.X.X.X]
    end
  
    subgraph "Server Infrastructure"
        G[API Gateway<br/>JWT + Rate Limit + CORS]
        W[IVISS WebService<br/>Rust + Axum + Tokio]
    end
  
    subgraph "Data Layer"
        DB1[(PostgreSQL Internal<br/>Organizations, Users,<br/>Agents, Audit Logs)]
        DB2[(PostgreSQL External<br/>National Vehicle DB<br/>Read-Only)]
    end
  
    subgraph "External Partners"
        API1[Insurance API]
        API2[Customs API]
        API3[Inspection API]
        API4[Wanted Vehicles API]
    end
  
    A -->|HTTPS + JWT| R
    B -->|HTTPS + JWT| R
    R -->|Port Forward :443 -> :8000| G
    G -->|Authenticated Request| W
    W -->|sqlx Queries<br/>Read-Write| DB1
    W -->|sqlx Queries<br/>Read-Only| DB2
    W -.->|HTTPS + API Key| API1
    W -.->|HTTPS + API Key| API2
    W -.->|HTTPS + API Key| API3
    W -.->|HTTPS + API Key| API4
  
    style A fill:#2dd4a8,stroke:#1a8f6f,color:#000
    style B fill:#60a5fa,stroke:#1e40af,color:#000
    style R fill:#fb923c,stroke:#c2410c,color:#000
    style G fill:#fb923c,stroke:#c2410c,color:#000
    style W fill:#a78bfa,stroke:#6d28d9,color:#000
    style DB1 fill:#f472b6,stroke:#be185d,color:#000
    style DB2 fill:#f87171,stroke:#b91c1c,color:#000
    style API1 fill:#f87171,stroke:#b91c1c,color:#000
    style API2 fill:#f87171,stroke:#b91c1c,color:#000
    style API3 fill:#f87171,stroke:#b91c1c,color:#000
    style API4 fill:#f87171,stroke:#b91c1c,color:#000
```



<style>#mermaid-1770193921173{font-family:sans-serif;font-size:16px;fill:#333;}#mermaid-1770193921173 .error-icon{fill:#552222;}#mermaid-1770193921173 .error-text{fill:#552222;stroke:#552222;}#mermaid-1770193921173 .edge-thickness-normal{stroke-width:2px;}#mermaid-1770193921173 .edge-thickness-thick{stroke-width:3.5px;}#mermaid-1770193921173 .edge-pattern-solid{stroke-dasharray:0;}#mermaid-1770193921173 .edge-pattern-dashed{stroke-dasharray:3;}#mermaid-1770193921173 .edge-pattern-dotted{stroke-dasharray:2;}#mermaid-1770193921173 .marker{fill:#333333;}#mermaid-1770193921173 .marker.cross{stroke:#333333;}#mermaid-1770193921173 svg{font-family:sans-serif;font-size:16px;}#mermaid-1770193921173 .label{font-family:sans-serif;color:#333;}#mermaid-1770193921173 .label text{fill:#333;}#mermaid-1770193921173 .node rect,#mermaid-1770193921173 .node circle,#mermaid-1770193921173 .node ellipse,#mermaid-1770193921173 .node polygon,#mermaid-1770193921173 .node path{fill:#ECECFF;stroke:#9370DB;stroke-width:1px;}#mermaid-1770193921173 .node .label{text-align:center;}#mermaid-1770193921173 .node.clickable{cursor:pointer;}#mermaid-1770193921173 .arrowheadPath{fill:#333333;}#mermaid-1770193921173 .edgePath .path{stroke:#333333;stroke-width:1.5px;}#mermaid-1770193921173 .flowchart-link{stroke:#333333;fill:none;}#mermaid-1770193921173 .edgeLabel{background-color:#e8e8e8;text-align:center;}#mermaid-1770193921173 .edgeLabel rect{opacity:0.5;background-color:#e8e8e8;fill:#e8e8e8;}#mermaid-1770193921173 .cluster rect{fill:#ffffde;stroke:#aaaa33;stroke-width:1px;}#mermaid-1770193921173 .cluster text{fill:#333;}#mermaid-1770193921173 div.mermaidTooltip{position:absolute;text-align:center;max-width:200px;padding:2px;font-family:sans-serif;font-size:12px;background:hsl(80,100%,96.2745098039%);border:1px solid #aaaa33;border-radius:2px;pointer-events:none;z-index:100;}#mermaid-1770193921173:root{--mermaid-font-family:sans-serif;}#mermaid-1770193921173:root{--mermaid-alt-font-family:sans-serif;}#mermaid-1770193921173 flowchart{fill:apa;}</style>
