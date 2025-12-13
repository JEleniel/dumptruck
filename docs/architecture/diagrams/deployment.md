
# Deployment Diagram

Deployment sketch (mermaid): ingress, API server replicas, workers, storage, and secrets. Labels simplified for consistent rendering.

```mermaid
flowchart LR
  User[User_Automation_Agents] --> LB[Load_Balancer_Ingress]
  LB --> API[API_Server_replicas]
  API --> Worker[Background_Workers]
  Worker --> Storage[(Object_Store_DB)]
  API --> Secrets[(Secret_Store)]
  CI[CI_CD] --> Registry[Container_Registry]
  Registry --> Deployment[Cluster]

  

```

Operational notes

- TLS at the ingress, OIDC/OAuth2 configured for API access, storage credentials via secret store.
