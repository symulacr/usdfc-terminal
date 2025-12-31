# USDFC Analytics Terminal - API Reference

REST API aggregating Filecoin RPC, Blockscout, Secured Finance, and GeckoTerminal into 10 endpoints.

## Base URL
```
Production: https://[YOUR-DOMAIN]/api/v1
Local:      http://localhost:3000/api/v1
```

## Endpoints

### GET /health
Service health status.
```bash
curl /api/v1/health
```
```json
{"status":"healthy","services":[{"name":"rpc","status":"ok","latency_ms":245}]}
```

### GET /price
Current USDFC and FIL prices.
```bash
curl /api/v1/price
```
```json
{"usdfc_usd":0.9996,"fil_usd":5.23,"change_24h":0.88,"volume_24h":125000}
```

### GET /metrics
Protocol health metrics (TCR, supply, collateral).
```bash
curl /api/v1/metrics
```
```json
{"total_supply":1250000,"total_collateral":850000,"tcr":189.5,"active_troves":234}
```

### GET /history
Historical metrics with time aggregation.
```bash
curl /api/v1/history?metrics=tcr,supply&resolution=1h&from=1735000000&to=1735086400
```
| Param | Values |
|-------|--------|
| metrics | tcr, supply, collateral, troves_count |
| resolution | 1m, 5m, 15m, 1h, 4h, 1d |

### GET /troves
Paginated list of all troves.
```bash
curl /api/v1/troves?limit=50&offset=0
```
```json
{"troves":[{"address":"0x...","debt":50000,"collateral":75000,"icr":250.5}],"total":234}
```

### GET /troves/:address
Single trove details.
```bash
curl /api/v1/troves/0x1234...
```

### GET /transactions
Recent USDFC transfers.
```bash
curl /api/v1/transactions?limit=20
```

### GET /address/:address
Full address info (balances, trove, transfers).
```bash
curl /api/v1/address/0x1234...
```

### GET /lending
Secured Finance lending markets.
```bash
curl /api/v1/lending
```
```json
{"markets":[{"currency_pair":"USDC/USDFC","borrow_rate":2.5,"lend_rate":2.3}]}
```

### GET /holders
Top USDFC token holders.
```bash
curl /api/v1/holders?limit=20
```

## Response Format
```json
{"success":true,"data":{...}}
```
Error:
```json
{"success":false,"error":"message"}
```

## Cache TTLs
| Endpoint | TTL |
|----------|-----|
| /metrics | 15s |
| /price | 30s |
| /lending | 60s |
| /troves | 120s |
| /holders | 300s |

## Rate Limits
Currently unlimited. Future: 100 req/min per IP.
