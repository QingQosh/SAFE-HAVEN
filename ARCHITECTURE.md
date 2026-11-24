# Architecture

This document defines the service boundaries, inter-service communication protocol,
shared library, service discovery mechanism, and migration path for the
microservices refactor (issue #963).

## Service Boundaries

Services are split along domain boundaries. Each service owns its data and is
deployed independently.

| Service | Domain | Responsibilities | Scaling profile |
| --- | --- | --- | --- |
| `api-gateway` | Edge | Request routing, auth, rate limiting, metrics | Horizontal, high throughput |
| `core-api` | Core | Primary business logic and resource CRUD | Horizontal, moderate |
| `metrics` | Observability | Metrics collection and aggregation | Low, periodic |
| `config` | Platform | Configuration distribution | Low, static |

Boundaries follow existing module layout (`src/api`, `src/config`, `src/api/metrics`)
so extraction is incremental rather than a rewrite.

## Inter-Service Communication

- **Synchronous:** HTTP/JSON over the internal network for request/response calls.
  - Requests carry `X-Request-Id` for tracing and `Authorization` for identity.
  - Responses use standard status codes; errors return `{ "error": { "code", "message" } }`.
- **Asynchronous:** message queues are explicitly out of scope for this issue.
- **Contract:** each service exposes a versioned base path (`/v1/...`). Breaking
  changes require a new version prefix.

## Shared Library

Common functionality (config loading, error types, tracing/metrics helpers,
HTTP client) lives in a shared crate consumed by all services. This avoids
duplicated logic while keeping services independently deployable.

## Service Discovery

- Services register their instance address and health status on startup.
- Consumers resolve instances by service name through the discovery registry.
- Health checks (`/healthz`) drive registration/deregistration.
- Orchestration (Kubernetes, etc.) is out of scope; discovery is registry-based.

## Migration Path

1. Extract the first service (`metrics`) behind the gateway while the monolith
   continues to serve all other routes.
2. Route traffic to the extracted service; keep a fallback to the monolith.
3. Repeat extraction per boundary in the table above.
4. Decommission monolith routes once each service is verified.

## Acceptance Criteria Mapping

- Service boundaries: see **Service Boundaries**.
- Communication protocol: see **Inter-Service Communication**.
- Independent deployment: each service has its own build/deploy unit.
- Service discovery: see **Service Discovery**.
- First service extracted: `metrics` (step 1 of the migration path).
