# Acme Cloud API — Security Guide

## Data Encryption

All data is encrypted at rest using **AES-256** encryption. Encryption keys are managed through AWS KMS with automatic key rotation every 90 days.

Data in transit is protected using **TLS 1.3**. Connections using TLS 1.2 or earlier are rejected. Certificate pinning is available for Enterprise customers.

## API Key Security

API keys are generated using cryptographically secure random number generation (256-bit entropy). Keys are hashed using bcrypt before storage — Acme never stores plaintext API keys.

**API keys never expire** and remain valid indefinitely. We recommend rotating keys annually as a security best practice, but this is not enforced. Compromised keys can be revoked instantly from the Dashboard.

Each API key can be scoped to specific endpoints using the Key Permissions system. Available scopes: `read`, `write`, `admin`, `billing`.

## IP Allowlisting

IP allowlisting is available on the **Pro plan and above**. You can configure up to 20 allowed IP addresses or CIDR ranges per API key.

Requests from non-allowed IPs receive a `403 Forbidden` response.

## Audit Logging

All API requests are logged with the following metadata:
- Timestamp
- Source IP
- API key identifier (last 4 characters)
- Endpoint accessed
- Response status code
- Request duration

Audit logs are retained for 90 days and can be exported in CSV or JSON format.

## Compliance

The Acme Cloud API is compliant with:
- **SOC 2 Type II**
- **GDPR** (EU-West region)
- **HIPAA** (Enterprise plan with BAA)

## Incident Response

Security incidents are communicated via:
1. Status page (status.acme-cloud.com)
2. Email notifications to account owners
3. In-dashboard alerts

Our target response time for critical security incidents is under 1 hour.
