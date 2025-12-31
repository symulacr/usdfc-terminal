# USDFC Terminal API Reference

Complete REST API documentation for the USDFC Analytics Terminal.

---

## Table of Contents

1. [Base URL](#base-url)
2. [Authentication](#authentication)
3. [Rate Limits](#rate-limits)
4. [Response Format](#response-format)
5. [Endpoints](#endpoints)
   - [Health](#health)
   - [Price](#price)
   - [Metrics](#metrics)
   - [History](#history)
   - [Troves](#troves)
   - [Transactions](#transactions)
   - [Address](#address)
   - [Lending](#lending)
   - [Holders](#holders)
6. [Data Types](#data-types)
7. [Error Codes](#error-codes)
8. [Changelog](#changelog)

---

## Base URL

### Production

```
https://usdfc-terminal.symulacr.dev/api/v1
```

### Local Development

```
http://localhost:3000/api/v1
```

### Health Endpoints (Non-versioned)

```
GET /health    # Full health check with service status
GET /ready     # Simple readiness probe
```

---

## Authentication

**Current Status:** No authentication required.

All API endpoints are currently public and do not require authentication. This is suitable for read-only analytics data.

### Future Plans

API key authentication may be implemented in future versions for:

- Rate limit increases
- Access to premium endpoints
- Usage tracking and analytics

When implemented, authentication will use the `Authorization` header:

```
Authorization: Bearer <api_key>
```

---

## Rate Limits

### Default Limits

| Tier       | Requests/Minute | Burst |
|------------|-----------------|-------|
| Public     | 100             | 20    |
| Authenticated (future) | 300 | 50    |

### Rate Limit Headers

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 85
X-RateLimit-Reset: 1703980800
```

### Server-Side Caching

The API implements server-side caching to reduce load on upstream data sources:

| Data Type        | Cache TTL   |
|------------------|-------------|
| Protocol Metrics | 15 seconds  |
| Price Data       | 30 seconds  |
| Lending Markets  | 60 seconds  |
| Troves           | 120 seconds |
| Token Holders    | 300 seconds |
| Holder Count     | 300 seconds |

---

## Response Format

All API responses follow a consistent JSON structure:

### Success Response

```json
{
  "success": true,
  "data": { ... },
  "timestamp": 1703980800
}
```

### Error Response

```json
{
  "success": false,
  "error": "Error message describing the issue",
  "timestamp": 1703980800
}
```

| Field       | Type    | Description                          |
|-------------|---------|--------------------------------------|
| `success`   | boolean | `true` if request succeeded          |
| `data`      | object  | Response payload (only on success)   |
| `error`     | string  | Error message (only on failure)      |
| `timestamp` | integer | Unix timestamp of response           |

---

## Endpoints

### Health

#### GET /api/v1/health

Returns API health status for all data sources (RPC, Blockscout, Subgraph, GeckoTerminal, History DB).

**Parameters:** None

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/health"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "services": [
      {
        "name": "rpc",
        "status": "healthy",
        "latency_ms": null
      },
      {
        "name": "blockscout",
        "status": "healthy",
        "latency_ms": null
      },
      {
        "name": "subgraph",
        "status": "healthy",
        "latency_ms": null
      },
      {
        "name": "gecko",
        "status": "healthy",
        "latency_ms": null
      },
      {
        "name": "database",
        "status": "healthy",
        "latency_ms": null
      }
    ]
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                    | Type   | Description                                                     |
|--------------------------|--------|-----------------------------------------------------------------|
| `status`                 | string | Overall status: `healthy` or `degraded`                         |
| `services`               | array  | Individual service health statuses                              |
| `services[].name`        | string | Service name: `rpc`, `blockscout`, `subgraph`, `gecko`, `database` |
| `services[].status`      | string | Service status: `healthy` or `unhealthy`                        |
| `services[].latency_ms`  | number | Response latency in milliseconds (optional)                     |

**Error Responses:**

| Status | Error Message          | Description                    |
|--------|------------------------|--------------------------------|
| 500    | Internal server error  | Health check failed            |

---

### Price

#### GET /api/v1/price

Returns current USDFC price data from GeckoTerminal DEX aggregator.

**Parameters:** None

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/price"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "usdfc_usd": 0.9987,
    "fil_usd": 4.52,
    "change_24h": -0.13,
    "volume_24h": 125000.50,
    "liquidity_usd": 850000.00
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field          | Type   | Description                                      |
|----------------|--------|--------------------------------------------------|
| `usdfc_usd`    | number | USDFC price in USD (null if unavailable)         |
| `fil_usd`      | number | FIL price in USD (null if unavailable)           |
| `change_24h`   | number | 24-hour price change percentage (null if N/A)    |
| `volume_24h`   | number | 24-hour trading volume in USD (null if N/A)      |
| `liquidity_usd`| number | Pool liquidity in USD (null if unavailable)      |

**Important Note:** All price fields may be `null` if the data source is unavailable. The API never returns fake fallback values (e.g., 1.0 for stablecoin price) to prevent masking depegging events.

**Error Responses:**

| Status | Error Message                      | Description                   |
|--------|------------------------------------|-------------------------------|
| 500    | GeckoTerminal API error: {details} | Price data fetch failed       |

---

### Metrics

#### GET /api/v1/metrics

Returns protocol-wide metrics including total supply, collateral, TCR, and more.

**Parameters:** None

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/metrics"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "tcr": "245.67%",
    "total_supply": "15000000.000000000000000000",
    "circulating_supply": "12500000.000000000000000000",
    "total_collateral": "8500000.000000000000000000",
    "active_troves": 342,
    "holders": 1250,
    "volume_24h": 125000.50,
    "liquidity_usd": 850000.00,
    "stability_pool_balance": "2500000.000000000000000000"
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                    | Type   | Description                                    |
|--------------------------|--------|------------------------------------------------|
| `tcr`                    | string | Total Collateralization Ratio as percentage    |
| `total_supply`           | string | Total USDFC supply (18 decimals)               |
| `circulating_supply`     | string | Circulating supply (excludes stability pool)   |
| `total_collateral`       | string | Total FIL collateral locked (18 decimals)      |
| `active_troves`          | number | Number of active troves (CDPs)                 |
| `holders`                | number | Number of USDFC token holders (optional)       |
| `volume_24h`             | number | 24-hour trading volume in USD (optional)       |
| `liquidity_usd`          | number | DEX pool liquidity in USD (optional)           |
| `stability_pool_balance` | string | USDFC deposited in stability pool              |

**Error Responses:**

| Status | Error Message                | Description                   |
|--------|------------------------------|-------------------------------|
| 500    | RPC error: {details}         | Filecoin RPC unavailable      |

---

### History

#### GET /api/v1/history

Returns historical volume data for charting.

**Query Parameters:**

| Parameter    | Type   | Required | Default  | Description                                    |
|--------------|--------|----------|----------|------------------------------------------------|
| `metric`     | string | No       | `volume` | Metric to retrieve (currently: `volume`)       |
| `from`       | number | No       | 30d ago  | Start timestamp (Unix seconds)                 |
| `to`         | number | No       | now      | End timestamp (Unix seconds)                   |
| `resolution` | string | No       | `1d`     | Resolution: `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `1d`, `1w` |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/history?metric=volume&from=1703376000&to=1703980800&resolution=1d"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "metric": "volume",
    "resolution": "1d",
    "from": 1703376000,
    "to": 1703980800,
    "data": [
      {
        "timestamp": 1703376000,
        "value": 45230.50
      },
      {
        "timestamp": 1703462400,
        "value": 52180.25
      },
      {
        "timestamp": 1703548800,
        "value": 38920.75
      }
    ]
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field              | Type   | Description                        |
|--------------------|--------|------------------------------------|
| `metric`           | string | Requested metric name              |
| `resolution`       | string | Data resolution                    |
| `from`             | number | Start timestamp                    |
| `to`               | number | End timestamp                      |
| `data`             | array  | Array of data points               |
| `data[].timestamp` | number | Unix timestamp                     |
| `data[].value`     | number | Metric value at that timestamp     |

**Error Responses:**

| Status | Error Message                   | Description                   |
|--------|---------------------------------|-------------------------------|
| 500    | Subgraph error: {details}       | Historical data fetch failed  |

---

### Troves

#### GET /api/v1/troves

Returns a paginated list of all active troves (collateralized debt positions).

**Query Parameters:**

| Parameter | Type   | Required | Default | Description                    |
|-----------|--------|----------|---------|--------------------------------|
| `limit`   | number | No       | 20      | Items per page (max: 100)      |
| `offset`  | number | No       | 0       | Pagination offset              |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/troves?limit=10&offset=0"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "troves": [
      {
        "address": "0x1234567890abcdef1234567890abcdef12345678",
        "collateral": "50000.000000000000000000",
        "debt": "25000.000000000000000000",
        "icr": "245.67%",
        "status": "active"
      },
      {
        "address": "0xabcdef1234567890abcdef1234567890abcdef12",
        "collateral": "12500.000000000000000000",
        "debt": "8500.000000000000000000",
        "icr": "132.50%",
        "status": "at_risk"
      }
    ],
    "total": 342,
    "offset": 0,
    "limit": 10
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                | Type   | Description                                          |
|----------------------|--------|------------------------------------------------------|
| `troves`             | array  | Array of trove objects                               |
| `troves[].address`   | string | Owner wallet address                                 |
| `troves[].collateral`| string | Collateral amount in FIL (18 decimals)               |
| `troves[].debt`      | string | Debt amount in USDFC (18 decimals)                   |
| `troves[].icr`       | string | Individual Collateralization Ratio as percentage     |
| `troves[].status`    | string | Status: `active`, `at_risk`, `critical`, `closed`    |
| `total`              | number | Total count of troves                                |
| `offset`             | number | Current page offset                                  |
| `limit`              | number | Page size limit                                      |

**Error Responses:**

| Status | Error Message                        | Description                    |
|--------|--------------------------------------|--------------------------------|
| 500    | RPC error fetching troves: {details} | Filecoin RPC unavailable       |
| 500    | FIL price is zero - price feed unavailable | Oracle failure            |

---

#### GET /api/v1/troves/:addr

Returns trove information for a specific address.

**Path Parameters:**

| Parameter | Type   | Required | Description                        |
|-----------|--------|----------|------------------------------------|
| `addr`    | string | Yes      | Wallet address (0x or f4 format)   |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/troves/0x1234567890abcdef1234567890abcdef12345678"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "collateral": "50000.000000000000000000",
    "debt": "25000.000000000000000000",
    "icr": "245.67%",
    "status": "active"
  },
  "timestamp": 1703980800
}
```

**Error Responses:**

| Status | Error Message                | Description                    |
|--------|------------------------------|--------------------------------|
| 400    | Invalid address format       | Address validation failed      |
| 404    | Trove not found for address  | No trove exists for address    |
| 500    | Internal server error        | RPC or data fetch failure      |

---

### Transactions

#### GET /api/v1/transactions

Returns recent USDFC token transactions.

**Query Parameters:**

| Parameter | Type   | Required | Default | Description                    |
|-----------|--------|----------|---------|--------------------------------|
| `limit`   | number | No       | 20      | Items per page (max: 100)      |
| `offset`  | number | No       | 0       | Pagination offset              |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/transactions?limit=5&offset=0"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "hash": "0xabc123def456789abc123def456789abc123def456789abc123def456789abc1",
        "tx_type": "transfer",
        "amount": "5000.000000000000000000",
        "from": "0x1234567890abcdef1234567890abcdef12345678",
        "to": "0xabcdef1234567890abcdef1234567890abcdef12",
        "timestamp": 1703980750,
        "block": 4523456,
        "status": "success"
      },
      {
        "hash": "0xdef789abc123456def789abc123456def789abc123456def789abc123456def7",
        "tx_type": "mint",
        "amount": "10000.000000000000000000",
        "from": "0x0000000000000000000000000000000000000000",
        "to": "0x9876543210fedcba9876543210fedcba98765432",
        "timestamp": 1703980650,
        "block": 4523445,
        "status": "success"
      }
    ],
    "total": 1250,
    "offset": 0,
    "limit": 5
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                     | Type   | Description                                           |
|---------------------------|--------|-------------------------------------------------------|
| `transactions`            | array  | Array of transaction objects                          |
| `transactions[].hash`     | string | Transaction hash                                      |
| `transactions[].tx_type`  | string | Type: `mint`, `burn`, `transfer`, `deposit`, `withdraw`, `liquidation`, `redemption` |
| `transactions[].amount`   | string | Amount in USDFC (18 decimals)                         |
| `transactions[].from`     | string | Sender address                                        |
| `transactions[].to`       | string | Recipient address                                     |
| `transactions[].timestamp`| number | Unix timestamp                                        |
| `transactions[].block`    | number | Block number                                          |
| `transactions[].status`   | string | Status: `pending`, `success`, `failed`                |
| `total`                   | number | Total count of transactions                           |
| `offset`                  | number | Current page offset                                   |
| `limit`                   | number | Page size limit                                       |

**Error Responses:**

| Status | Error Message                   | Description                   |
|--------|---------------------------------|-------------------------------|
| 500    | Blockscout API error: {details} | Transaction fetch failed      |

---

### Address

#### GET /api/v1/address/:addr

Returns detailed information about an address including USDFC balance and activity.

**Path Parameters:**

| Parameter | Type   | Required | Description                        |
|-----------|--------|----------|------------------------------------|
| `addr`    | string | Yes      | Wallet address (0x or f4 format)   |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/address/0x1234567890abcdef1234567890abcdef12345678"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "usdfc_balance": "25000.500000000000000000",
    "transfer_count": 47,
    "first_seen": "2024-01-15T10:30:00Z",
    "address_type": "eoa",
    "f4_address": "f410f2iqk4a3nzeqq7rqxrqxrqxrqxrqxrqxrqxrqxq"
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field            | Type   | Description                                           |
|------------------|--------|-------------------------------------------------------|
| `address`        | string | Normalized address (EVM format)                       |
| `usdfc_balance`  | string | Current USDFC balance                                 |
| `transfer_count` | number | Total number of USDFC transfers                       |
| `first_seen`     | string | First transaction timestamp (ISO 8601)                |
| `address_type`   | string | Type: `eoa`, `contract`, `protocol`                   |
| `f4_address`     | string | Filecoin f4 address (optional, if applicable)         |

**Error Responses:**

| Status | Error Message                                      | Description                           |
|--------|----------------------------------------------------|---------------------------------------|
| 400    | Invalid address format                             | Address validation failed             |
| 400    | f1/f3 addresses are not supported by Blockscout... | Unsupported address format            |
| 500    | Blockscout API error: {details}                    | API failure                           |

---

### Lending

#### GET /api/v1/lending

Returns lending market data from Secured Finance subgraph.

**Parameters:** None

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/lending"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "markets": [
      {
        "maturity": "1735689600",
        "currency": "USDFC",
        "lend_apr": 5.25,
        "borrow_apr": 7.50,
        "volume": "125000.000000000000000000",
        "is_active": true
      },
      {
        "maturity": "1738368000",
        "currency": "USDFC",
        "lend_apr": 4.80,
        "borrow_apr": 6.90,
        "volume": "89500.000000000000000000",
        "is_active": true
      }
    ]
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                  | Type    | Description                            |
|------------------------|---------|----------------------------------------|
| `markets`              | array   | Array of lending market objects        |
| `markets[].maturity`   | string  | Market maturity timestamp              |
| `markets[].currency`   | string  | Currency symbol                        |
| `markets[].lend_apr`   | number  | Current lending APR percentage         |
| `markets[].borrow_apr` | number  | Current borrowing APR percentage       |
| `markets[].volume`     | string  | Total volume (18 decimals)             |
| `markets[].is_active`  | boolean | Whether market is currently active     |

**Error Responses:**

| Status | Error Message                   | Description                   |
|--------|---------------------------------|-------------------------------|
| 500    | Subgraph error: {details}       | Subgraph query failed         |

---

### Holders

#### GET /api/v1/holders

Returns top USDFC token holders.

**Query Parameters:**

| Parameter | Type   | Required | Default | Description                    |
|-----------|--------|----------|---------|--------------------------------|
| `limit`   | number | No       | 20      | Items per page (max: 50)       |

**Example Request:**

```bash
curl -X GET "https://usdfc-terminal.symulacr.dev/api/v1/holders?limit=10"
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "holders": [
      {
        "address": "0x791Ad78bBc58324089D3E0A8689E7D045B9592b5",
        "balance": "2500000.000000000000000000",
        "share": null
      },
      {
        "address": "0x1234567890abcdef1234567890abcdef12345678",
        "balance": "500000.000000000000000000",
        "share": null
      }
    ],
    "total_holders": 1250
  },
  "timestamp": 1703980800
}
```

**Response Fields:**

| Field                | Type   | Description                              |
|----------------------|--------|------------------------------------------|
| `holders`            | array  | Array of holder objects                  |
| `holders[].address`  | string | Holder wallet address                    |
| `holders[].balance`  | string | USDFC balance (18 decimals)              |
| `holders[].share`    | number | Percentage of total supply (optional)    |
| `total_holders`      | number | Total count of all holders (optional)    |

**Error Responses:**

| Status | Error Message                     | Description                   |
|--------|-----------------------------------|-------------------------------|
| 500    | Blockscout API error: {details}   | Holder data fetch failed      |

---

## Data Types

### TroveStatus

Trove health status based on Individual Collateralization Ratio (ICR):

| Status     | ICR Range     | Description                           |
|------------|---------------|---------------------------------------|
| `active`   | >= 150%       | Healthy, well-collateralized          |
| `at_risk`  | 125% - 150%   | Warning zone, should add collateral   |
| `critical` | 110% - 125%   | Danger zone, at risk of liquidation   |
| `closed`   | < 110%        | Closed or liquidated                  |

### TransactionType

| Type          | Description                                |
|---------------|--------------------------------------------|
| `mint`        | New USDFC minted from trove                |
| `burn`        | USDFC burned (debt repayment)              |
| `transfer`    | Standard ERC20 transfer                    |
| `deposit`     | Stability pool deposit                     |
| `withdraw`    | Stability pool withdrawal                  |
| `liquidation` | Trove liquidation event                    |
| `redemption`  | USDFC redemption for collateral            |

### TransactionStatus

| Status    | Description                   |
|-----------|-------------------------------|
| `pending` | Transaction is pending        |
| `success` | Transaction succeeded         |
| `failed`  | Transaction failed            |

### AddressType

| Type       | Description                           |
|------------|---------------------------------------|
| `eoa`      | Externally Owned Account (wallet)     |
| `contract` | Smart contract                        |
| `protocol` | Known protocol contract               |

### ChartResolution

Available resolutions for historical data:

| Code  | Description  |
|-------|--------------|
| `1m`  | 1 minute     |
| `5m`  | 5 minutes    |
| `15m` | 15 minutes   |
| `30m` | 30 minutes   |
| `1h`  | 1 hour       |
| `4h`  | 4 hours      |
| `1d`  | 1 day        |
| `1w`  | 1 week       |

### ServiceStatus

| Status      | Description                               |
|-------------|-------------------------------------------|
| `healthy`   | Service is operating normally             |
| `unhealthy` | Service is unavailable or erroring        |
| `degraded`  | Service is partially operational          |

---

## Error Codes

### HTTP Status Codes

| Code | Description                                        |
|------|----------------------------------------------------|
| 200  | Success                                            |
| 400  | Bad Request - Invalid parameters or address format |
| 404  | Not Found - Resource does not exist                |
| 429  | Too Many Requests - Rate limit exceeded            |
| 500  | Internal Server Error - Upstream API failure       |
| 503  | Service Unavailable - Maintenance or overload      |

### Common Error Messages

| Error                                               | Cause                                    |
|-----------------------------------------------------|------------------------------------------|
| `Invalid address format`                            | Address doesn't match 0x or f4 format    |
| `Trove not found for address`                       | No trove exists for the given address    |
| `f1/f3 addresses are not supported by Blockscout...`| Unsupported Filecoin address type        |
| `RPC error fetching troves`                         | Filecoin RPC node unavailable            |
| `FIL price is zero - price feed unavailable`        | Oracle/price feed failure                |
| `Blockscout API error`                              | Blockscout explorer API failure          |
| `GeckoTerminal OHLCV error`                         | Price data API unavailable               |
| `Subgraph error`                                    | Secured Finance subgraph unavailable     |

---

## Changelog

### v1.0.0 (2024-12-31)

Initial API release with the following endpoints:

- `GET /api/v1/health` - API health status
- `GET /api/v1/price` - USDFC price data
- `GET /api/v1/metrics` - Protocol metrics
- `GET /api/v1/history` - Historical volume data
- `GET /api/v1/troves` - List all troves
- `GET /api/v1/troves/:addr` - Get trove by address
- `GET /api/v1/transactions` - Recent transactions
- `GET /api/v1/address/:addr` - Address information
- `GET /api/v1/lending` - Lending market data
- `GET /api/v1/holders` - Top token holders

**Data Sources:**

- Filecoin RPC (Glif) - On-chain contract data
- Blockscout - Token transfers and holder data
- Secured Finance Subgraph (Goldsky) - Lending market data
- GeckoTerminal - DEX price and volume data

---

## Additional Resources

### Infrastructure Endpoints

These endpoints are for infrastructure monitoring and are not part of the versioned API:

```bash
# Full health check with detailed service status (includes RPC, Blockscout, Subgraph, Gecko, Database)
GET /health

# Simple readiness probe (returns "ok")
GET /ready
```

**Full Health Check Response:**

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_secs": 86400,
  "checks": {
    "rpc": {
      "status": "ok",
      "latency_ms": 245,
      "error": null
    },
    "blockscout": {
      "status": "ok",
      "latency_ms": 180,
      "error": null
    },
    "subgraph": {
      "status": "ok",
      "latency_ms": 320,
      "error": null
    },
    "gecko": {
      "status": "ok",
      "latency_ms": 150,
      "error": null
    },
    "database": {
      "status": "ok",
      "latency_ms": 5,
      "error": null
    }
  }
}
```

### CORS Support

The API supports Cross-Origin Resource Sharing (CORS) for all `/api/v1/*` endpoints:

- **Allowed Origins:** `*` (all origins)
- **Allowed Methods:** `GET`
- **Allowed Headers:** All

### Server Functions (Internal)

The terminal also exposes Leptos server functions at `/api/*` for internal use by the web application. These are not part of the public REST API and may change without notice:

- `GetProtocolMetrics`
- `GetRecentTransactions`
- `GetTroves`
- `GetLendingMarkets`
- `GetDailyVolumes`
- `GetAddressInfo`
- `GetNormalizedAddress`
- `GetTopHolders`
- `GetStabilityPoolTransfers`
- `GetUSDFCPriceData`
- `CheckApiHealth`
- `GetHolderCount`
- `GetOrderBook`
- `GetRecentLendingTrades`
- `GetAdvancedChartData`

---

## Contract Addresses

For reference, key USDFC protocol contract addresses on Filecoin mainnet:

| Contract           | Address                                      |
|--------------------|----------------------------------------------|
| USDFC Token        | `0x80B98d3aa09ffff255c3ba4A241111Ff1262F045` |
| Trove Manager      | `0x5aB87c2398454125Dd424425e39c8909bBE16022` |
| Stability Pool     | `0x791Ad78bBc58324089D3E0A8689E7D045B9592b5` |
| Price Feed         | `0x80e651c9739C1ed15A267c11b85361780164A368` |
| Active Pool        | `0x8637Ac7FdBB4c763B72e26504aFb659df71c7803` |
| Borrower Operations| `0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0` |
| Sorted Troves      | `0x2C32e48e358d5b893C46906b69044D342d8DDd5F` |
| Multi Trove Getter | `0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F` |

### DEX Pool Addresses

| Pool              | Address                                      |
|-------------------|----------------------------------------------|
| USDFC/WFIL        | `0x4e07447bd38e60b94176764133788be1a0736b30` |
| USDFC/axlUSDC     | `0x21ca72fe39095db9642ca9cc694fa056f906037f` |
| USDFC/USDC        | `0xc8f38dbaf661b897b6a2ee5721aac5a8766ffa13` |
