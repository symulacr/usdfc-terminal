# ProPGF Batch 2 - Karma Portal Application

**Copy-paste ready fields for https://app.filpgf.io/programs/992/apply**

---

## Section 1: Project Information

**Project Name:**
```
USDFC Analytics Terminal
```

**Theme/Category:**
```
Tooling & Developer Ecosystem
```

**Description (max 500 chars):**
```
Unified REST API aggregating Filecoin RPC, Blockscout, Secured Finance Subgraph, and GeckoTerminal into 10 endpoints for USDFC stablecoin analytics. Provides real-time protocol metrics (TCR, supply, collateral), price/volume data, trove explorer, lending markets, and transaction history. Built with 15,000 lines of production Rust. Open source under MIT+Apache2.
```

**Website URL:**
```
[YOUR_DEPLOYED_URL]
```

---

## Section 2: Team Contact

**Team Lead Name:**
```
[YOUR_NAME]
```

**Email:**
```
[YOUR_EMAIL]
```

**Telegram:**
```
[YOUR_TELEGRAM]
```

**GitHub:**
```
[YOUR_GITHUB_URL]
```

**Slack:**
```
[YOUR_SLACK_HANDLE]
```

---

## Section 3: Problem Statement

```
Developers building on USDFC must integrate 4+ APIs (Filecoin RPC, Blockscout, Secured Finance, GeckoTerminal), each with different authentication, rate limits, and response formats. No unified interface exists for protocol metrics, lending markets, or price data. This fragmentation slows development and creates maintenance burden.
```

**Target Users:**
```
- DeFi developers building on Filecoin
- Analytics platforms (DeFiLlama, DefiPulse)
- Institutions monitoring USDFC protocol health
- Traders needing real-time price/volume data
```

**Expected Outputs:**
```
1. Production REST API with 10 endpoints
2. OpenAPI 3.0 specification with Swagger UI
3. TypeScript SDK published on npm
4. Goldsky subgraph for historical queries
5. Telegram alert bot for wallet/price monitoring
```

---

## Section 4: Milestones

### Milestone 1: Production API Deployment

**Work Description:**
```
Deploy 10 REST endpoints on public URL with security headers, CORS, caching layer, and health checks. Endpoints: /health, /price, /metrics, /history, /troves, /troves/:addr, /transactions, /address/:addr, /lending, /holders.
```

**Deliverables:**
```
- Live API at public URL
- All 10 endpoints returning valid JSON
- Security headers (CSP, HSTS, X-Frame-Options)
- Cache layer with appropriate TTLs
```

**Acceptance Criteria:**
```
GET /api/v1/health returns HTTP 200 with {"status":"healthy"}. All 10 endpoints return valid JSON responses. Lighthouse security score >90.
```

**Funding:** `$5,000`
**Timeline:** `2 weeks`

---

### Milestone 2: OpenAPI Specification

**Work Description:**
```
Create OpenAPI 3.0.3 specification documenting all endpoints with request/response schemas. Deploy Swagger UI at /api-docs for interactive exploration.
```

**Deliverables:**
```
- docs/openapi.yaml (OpenAPI 3.0.3 spec)
- Swagger UI hosted at /api-docs
- All request parameters documented
- All response schemas defined
```

**Acceptance Criteria:**
```
openapi.yaml passes `swagger-cli validate`. Swagger UI accessible at /api-docs. All 10 endpoints fully documented with examples.
```

**Funding:** `$2,000`
**Timeline:** `1 week`

---

### Milestone 3: TypeScript SDK

**Work Description:**
```
Build and publish @usdfc/terminal-sdk npm package with full API coverage. Includes TypeScript types, async/await interface, error handling, and comprehensive tests.
```

**Deliverables:**
```
- npm package: @usdfc/terminal-sdk
- 100% endpoint coverage
- TypeScript definitions
- Jest test suite (>80% coverage)
- README with usage examples
```

**Acceptance Criteria:**
```
Package published on npmjs.com. All endpoints callable via SDK. Tests passing in CI. TypeScript types exported correctly.
```

**Funding:** `$8,000`
**Timeline:** `3 weeks`

---

### Milestone 4: Goldsky Subgraph

**Work Description:**
```
Deploy USDFC event indexing subgraph on Goldsky. Index all USDFC transfers, DEX swaps, lending events, and liquidations from genesis block.
```

**Deliverables:**
```
- Subgraph schema (schema.graphql)
- Mapping handlers for all events
- Deployed on Goldsky infrastructure
- GraphQL endpoint accessible
```

**Acceptance Criteria:**
```
Subgraph live on Goldsky. Queries return data from genesis block. Historical transfers queryable by address and time range.
```

**Funding:** `$10,000`
**Timeline:** `4 weeks`

---

### Milestone 5: Telegram Alert Bot

**Work Description:**
```
Build @usdfc_alerts Telegram bot for wallet monitoring and price alerts. Users can set thresholds for notifications on wallet activity, price movements, and protocol health changes.
```

**Deliverables:**
```
- Telegram bot: @usdfc_alerts
- Wallet activity alerts
- Price threshold alerts
- TCR health alerts
- User configuration persistence
```

**Acceptance Criteria:**
```
Bot responds to /start command. Users can configure alerts via /setalert. Alerts delivered within 60 seconds of trigger event.
```

**Funding:** `$5,000`
**Timeline:** `3 weeks`

---

## Section 5: Budgeting

**Total Funding Request:**
```
$30,000
```

**Prior Funding:**
```
$0 (self-funded development to date)
```

**Minimum Viable Amount:**
```
$15,000 (Milestones 1-3 only: API + OpenAPI + SDK)
```

**Budget Breakdown:**
```
M1 ($5,000): Server hosting $500, DevOps $1,500, Testing $1,500, Documentation $1,500
M2 ($2,000): Schema design $1,000, Swagger setup $500, Validation $500
M3 ($8,000): SDK development $5,000, npm publishing $500, Tests $1,500, Docs $1,000
M4 ($10,000): Goldsky fees $3,000 (3mo), Schema design $2,000, Indexer dev $4,000, Testing $1,000
M5 ($5,000): Bot framework $1,500, Notification service $2,000, User testing $1,500
```

---

## Section 6: Measurement & Indicators

**Key Metric Selection:**
```
- Raw PiBs Onboarded (indirect via developer adoption)
- Total Value Transacted (via trading bot integrations)
```

**Impact Pathway:**
```
SDK adoption → More developers building on USDFC → Increased protocol usage → More FIL locked as collateral → Higher network activity
```

**SMART Metrics (2-3):**
```
1. API Requests/Day: 10,000 requests/day within 3 months of launch
2. SDK Downloads: 500 npm downloads within 3 months
3. Subgraph Queries: 5,000 queries/day via Goldsky within 3 months
```

---

## Section 7: Sustainability & Team

**Sustainability Plan:**
```
Open source under MIT+Apache2 dual license. Community maintenance model with CONTRIBUTING.md guidelines. Post-grant revenue via premium API tiers: Growth ($99/mo for hosted API with 30-day retention), Enterprise (custom SLA and dedicated support).
```

**Team Information:**
```
[YOUR_NAME] - Lead Developer
GitHub: [YOUR_GITHUB]
Experience: [BRIEF_EXPERIENCE]
```

---

## Section 8: Risks & Dependencies

**Key Risks:**
```
1. External API downtime (Medium likelihood, High impact)
   Mitigation: Cache layer with fallback; graceful degradation

2. Goldsky indexing delays (Low likelihood, Medium impact)
   Mitigation: Start schema design early; buffer in timeline

3. SDK adoption slower than expected (Medium likelihood, Low impact)
   Mitigation: Marketing via Filecoin channels; documentation quality
```

**Ecosystem Dependencies:**
```
- Filecoin RPC (Glif API) - public endpoint, no auth required
- Blockscout API - public endpoint, no auth required
- Secured Finance Subgraph - hosted on Goldsky
- GeckoTerminal API - public endpoint, rate limited
```

---

## Section 9: Payout

**Karma Profile URL:**
```
[CREATE AT app.filpgf.io AND PASTE URL]
```

**Wallet Address:**
```
[YOUR_FILECOIN_ADDRESS]
```

---

## Checklist Before Submission

- [ ] Joined Telegram: https://t.me/+nUc-d7FXmt1kOWVl
- [ ] Joined Slack #propgf: filecoin.io/slack
- [ ] Created Karma profile
- [ ] Deployed live demo
- [ ] Filled all fields above
- [ ] Wallet address ready for payout
