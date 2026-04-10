# Acme Cloud API — Version 1.0 Documentation

## Overview

The Acme Cloud API provides a RESTful interface for managing cloud resources, executing serverless functions, and accessing distributed storage. This documentation covers the original release of the API.

## Authentication

All API requests must include a valid API key in the `X-API-Key` header. API keys can be generated from the Acme Dashboard under Settings > API Keys.

Only API key authentication is supported. Each account can generate up to 5 API keys.

## Rate Limiting

The API enforces a global rate limit of **100 requests per minute** per API key. Exceeding this limit returns a `429 Too Many Requests` response with a `Retry-After` header.

There is no distinction between plans — all users share the same rate limit.

## Connection Settings

The default connection timeout is **60 seconds**. This value is not configurable. If a request does not complete within 60 seconds, the server terminates the connection and returns a `504 Gateway Timeout`.

Keep-alive connections are supported with a maximum idle time of 30 seconds.

## Request Limits

The maximum request payload size is **5 MB**. Requests exceeding this limit receive a `413 Payload Too Large` response. This applies to all endpoints including file uploads.

Response bodies are not subject to size limits but are typically under 1 MB.

## Available Regions

The API is available in the following regions:

- **US-East** (us-east-1) — Primary region, lowest latency for North American users
- **EU-West** (eu-west-1) — GDPR-compliant region for European users

Cross-region replication is not available. Data stored in one region cannot be accessed from another.

## Error Handling

All errors follow a standard format:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests",
    "retry_after": 30
  }
}
```

## Support

For technical support, contact api-support@acme-cloud.com. Response times are typically within 48 hours for all support tiers.
