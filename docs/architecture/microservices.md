# Microservices Architecture Plan

This document defines the service boundaries, communication strategy, shared library, service discovery, and migration path for decomposing the current monolith. It is the design artifact for issue #963.

## 1. Service Boundaries

Boundaries are drawn along existing domain modules so extraction is incremental and does not require rewriting the application.

| Service | Responsibility | Current code | Scaling profile |
| --- | --- | --- | --- |
| `api-gateway` | Request routing, auth middleware, rate limiting, metrics exposure | `src/api/mod.rs`, `src/api/middleware.rs`, `src/api/metrics.rs` | Horizontal, high request volume |
| `config-service` | Configuration loading and distribution | `src/config.rs` | Low volume, high availability |
| `core-service` | Domain/business logic (first extraction target) | domain modules | CPU-bound, independent deploy cycle |

Rules:
- Each service owns its data; no shared database tables across boundaries.
- Cross-boundary reads go through the service API, not direct DB access.
- Shared, stateless helpers live in the shared library (section 3), not duplicated.

## 2. Inter-Service Communication

- **Synchronous calls:** HTTP/JSON over the internal network for request/response paths (gateway to core).
- **Contract:** versioned REST endpoints (`/v1/...`); breaking changes require a new version prefix.
- **Errors:** standard JSON error envelope `{ "error": { "code": string, "message": string } }` with appropriate HTTP status.
- **Timeouts/retries:** bounded timeouts on all outbound calls; retries only for idempotent requests.
- **Asynchronous messaging:** explicitly out of scope for this issue (no message queues).

## 3. Shared Library

Common functionality is factored into a shared crate consumed by all services:

- Config loading (`src/config.rs`)
- Auth/request middleware (`src/api/middleware.rs`)
- Metrics helpers (`src/api/metrics.rs`)
- Error envelope types and tracing setup

The shared library must remain stateless and free of service-specific business logic.

## 4. Service Discovery

- Services register themselves on startup and resolve peers by logical name (e.g. `core-service`).
- Initial implementation: static/DNS-based discovery via environment variables, sufficient for independent deployment without orchestration.
- Health endpoints (`/healthz`) on every service drive registration and readiness checks.
- Kubernetes/orchestration-based discovery is out of scope for this issue.

## 5. Migration Path

1. Extract the shared library from existing modules (no behavior change).
2. Stand up `core-service` behind the existing gateway; route a small percentage of traffic to it.
3. Verify health checks and discovery resolve `core-service` correctly.
4. Increase traffic to `core-service`; keep the monolith path as fallback until stable.
5. Remove the extracted code path from the monolith once `core-service` is fully serving.

Each step keeps the system deployable; no big-bang rewrite.

## Acceptance Criteria Mapping

- Service boundaries defined: section 1.
- Communication protocol documented: section 2.
- Independent deployability: per-service build/deploy with shared library, section 3.
- Service discovery: section 4.
- First service extracted: `core-service` per section 5.
