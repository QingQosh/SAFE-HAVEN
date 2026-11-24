# API Versioning Strategy

This document defines how the API is versioned, how clients select a version, how
version information is surfaced in responses, and how versions are deprecated.

## Versioning scheme

The API uses **URL-based versioning**. The major version is part of the path:

```
/api/v1/<resource>
/api/v2/<resource>
```

- Only the **major** version appears in the URL. Minor and patch changes are
  backward compatible and do not change the path.
- A breaking change (removing/renaming a field, changing a type or the meaning of
  an existing field, changing status codes, etc.) requires a new major version.
- Additive, backward-compatible changes (new optional fields, new endpoints) ship
  within the current major version.

## Running multiple versions

At most **two** major versions run simultaneously: the current version and the
previous one. When a new major version is introduced, the oldest supported version
enters its deprecation window (see below) and is removed once the window closes.

## Selecting a version

Clients specify the desired version in one of two ways, in order of precedence:

1. **URL path** — `GET /api/v2/health` (preferred).
2. **`Accept-Version` header** — `Accept-Version: v2`. Used when the client cannot
   change the path (e.g. a shared gateway).

If neither is provided, the request is served by the **default (current)** version.
Requests for an unsupported or unknown version are rejected with `400 Bad Request`
and a body describing the supported versions.

## Version information in responses

Every API response includes the version that served it:

- Response header: `X-API-Version: v2`
- Response body envelope (where applicable): a top-level `api_version` field.

When a request is served by a deprecated version, the response also includes:

- `Deprecation: true`
- `Sunset: <HTTP-date>` — the date the version will be removed.
- `Link: <docs/API_VERSIONING.md>; rel="deprecation"`

## Deprecation policy and timeline

When a major version is superseded:

| Phase            | Duration        | Behavior                                                        |
|------------------|-----------------|-----------------------------------------------------------------|
| Announcement     | day 0           | New version released; old version marked deprecated in docs.    |
| Deprecation      | 6 months        | Old version still served; responses carry deprecation headers.  |
| Sunset           | end of window   | Old version removed; requests return `410 Gone`.                |

- The minimum deprecation window is **6 months** from announcement to sunset.
- The sunset date is published in the `Sunset` header and in this document.
- Breaking changes are never introduced into a version that is still supported.

## Migrating existing clients

1. Read the changelog for the new major version and identify breaking changes.
2. Point clients at the new path (`/api/v2/...`) or send `Accept-Version: v2`.
3. Watch for `Deprecation`/`Sunset` headers on the old version to track the
   remaining migration window.
4. Remove references to the old version before its sunset date.

## Version history

| Version | Status      | Notes                          |
|---------|-------------|--------------------------------|
| v1      | Deprecated  | Initial versioned API.         |
| v2      | Current     | Current default version.       |
