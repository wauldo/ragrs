# Acme Cloud API — Version 2.0 Documentation

## Overview

The Acme Cloud API v2 introduces significant improvements to performance, security, and developer experience. This documentation covers the latest release.

## Authentication

The API supports two authentication methods:

1. **API Key** — Include your key in the `X-API-Key` header. Suitable for server-to-server communication.
2. **OAuth2** — Use the `/oauth/token` endpoint to obtain a bearer token. Recommended for user-facing applications.

OAuth2 supports authorization code flow and client credentials grant. Tokens expire after 1 hour and can be refreshed.

## Rate Limiting

Rate limits vary by plan:

- **Basic**: 500 requests per minute
- **Pro**: 5,000 requests per minute
- **Enterprise**: Custom limits negotiated per contract

Rate limit headers (`X-RateLimit-Remaining`, `X-RateLimit-Reset`) are included in every response.

## Connection Settings

The default connection timeout is **30 seconds**. This can be configured per-request using the `timeout` query parameter (minimum 5s, maximum 120s).

Example: `GET /api/v2/resources?timeout=15`

HTTP/2 is supported by default. Keep-alive connections have a maximum idle time of 60 seconds.

## Request Limits

The maximum request payload size is **10 MB**. For file uploads specifically, the limit is **50 MB** when using multipart form encoding.

Streaming uploads are supported for payloads exceeding the standard limit.

## Available Regions

The API is available in four regions:

- **US-East** (us-east-1) — Primary region
- **US-West** (us-west-2) — Added in v2 for West Coast latency optimization
- **EU-West** (eu-west-1) — GDPR-compliant
- **AP-Southeast** (ap-southeast-1) — Asia-Pacific region, added in v2

Cross-region replication is available on Pro and Enterprise plans.

## Webhooks

v2 introduces webhook support for asynchronous event notifications. Configure webhook URLs in the Dashboard under Settings > Webhooks.

Events include: `resource.created`, `resource.updated`, `resource.deleted`, `job.completed`, `job.failed`.

## Support

- **Basic**: Email support, 48-hour response time
- **Pro**: Priority email + chat, 4-hour response time
- **Enterprise**: Dedicated support engineer, 1-hour response time
