# Week 2 Setup Guide - Docker + CI/CD Pipeline

**Date:** 2026-01-06
**Status:** IN PROGRESS - Day 2
**Goal:** Production CI/CD pipeline with Docker layer caching + sccache S3

---

## Overview

Week 2 implements a production-grade CI/CD pipeline that builds Docker images in GitHub Actions and deploys pre-built images to Railway. This dramatically reduces Railway build times from 8-12 minutes to 30-60 seconds.

**Architecture:**
```
┌─────────────────┐      ┌──────────────────┐      ┌────────────────┐
│ Git Push        │──────▶│ GitHub Actions   │──────▶│ GHCR Registry  │
│ (Code Changes)  │      │ (Build Docker)   │      │ (Store Images) │
└─────────────────┘      └──────────────────┘      └────────────────┘
                                   │                         │
                                   │                         ▼
                                   │                ┌────────────────┐
                                   └───────────────▶│ Railway        │
                                     Deploy Webhook │ (Deploy Image) │
                                                    └────────────────┘
```

---

## Files Created

### 1. `Dockerfile.production`
Multi-stage optimized Dockerfile with:
- **cargo-chef** for dependency layer caching (5-10x faster rebuilds)
- **mold linker** (5-12x faster linking)
- **sccache** with S3 backend (50-70% faster compilation in CI)
- **BuildKit cache mounts** for aggressive caching
- Minimal runtime image (~200MB final size)

### 2. `.github/workflows/docker-build.yml`
CI/CD workflow with:
- Docker layer caching via GitHub Actions cache
- sccache S3 backend for Rust compilation caching
- Automated push to GitHub Container Registry (GHCR)
- Railway deployment webhook trigger
- Build time tracking and summaries

---

## Prerequisites

Before running the CI/CD pipeline, you need to configure:

### 1. AWS S3 Bucket for sccache

sccache uses S3 to share compilation cache across CI builds.

**Create S3 bucket:**
```bash
# Set your AWS profile if needed
export AWS_PROFILE=your-profile

# Create S3 bucket (use your preferred region)
aws s3api create-bucket \
  --bucket usdfc-terminal-sccache \
  --region us-east-1 \
  --create-bucket-configuration LocationConstraint=us-east-1

# Enable versioning (optional, for rollback)
aws s3api put-bucket-versioning \
  --bucket usdfc-terminal-sccache \
  --versioning-configuration Status=Enabled

# Set lifecycle policy to auto-delete old cache (30 days)
aws s3api put-bucket-lifecycle-configuration \
  --bucket usdfc-terminal-sccache \
  --lifecycle-configuration '{
    "Rules": [{
      "Id": "delete-old-cache",
      "Status": "Enabled",
      "Expiration": { "Days": 30 }
    }]
  }'
```

**Create IAM user for CI:**
```bash
# Create IAM user for GitHub Actions
aws iam create-user --user-name github-actions-sccache

# Create policy for S3 access
cat > sccache-policy.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::usdfc-terminal-sccache",
        "arn:aws:s3:::usdfc-terminal-sccache/*"
      ]
    }
  ]
}
EOF

# Attach policy to user
aws iam put-user-policy \
  --user-name github-actions-sccache \
  --policy-name SccacheS3Access \
  --policy-document file://sccache-policy.json

# Create access keys
aws iam create-access-key --user-name github-actions-sccache
# Save the AccessKeyId and SecretAccessKey for GitHub secrets
```

### 2. GitHub Repository Secrets

Add the following secrets to your GitHub repository:

**Settings → Secrets and variables → Actions → New repository secret**

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `AWS_ACCESS_KEY_ID` | `AKIA...` | IAM user access key for sccache S3 |
| `AWS_SECRET_ACCESS_KEY` | `wJa...` | IAM user secret key for sccache S3 |
| `RAILWAY_WEBHOOK_URL` | `https://...` | (Optional) Railway webhook for deploy |

**Verify secrets:**
```bash
gh secret list
# Should show:
# AWS_ACCESS_KEY_ID      Updated 2026-01-06
# AWS_SECRET_ACCESS_KEY  Updated 2026-01-06
# RAILWAY_WEBHOOK_URL    Updated 2026-01-06
```

### 3. Enable GitHub Container Registry (GHCR)

**Enable GHCR for your repository:**
1. Go to `https://github.com/settings/packages`
2. Enable "Improved container support"
3. Go to repository → Settings → Actions → General
4. Under "Workflow permissions", select:
   - ✅ Read and write permissions
   - ✅ Allow GitHub Actions to create and approve pull requests

**Verify GHCR access:**
```bash
# Login to GHCR with personal access token
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Should see: Login Succeeded
```

---

## Testing the Pipeline

### Local Docker Build (No S3)
Test the Dockerfile locally without S3 cache:

```bash
# Build with BuildKit (uses local cache only)
DOCKER_BUILDKIT=1 docker build \
  -f Dockerfile.production \
  -t usdfc-terminal:local \
  --progress=plain \
  . 2>&1 | tee /tmp/docker-build-local.log

# Check build time
grep "DONE" /tmp/docker-build-local.log | tail -5

# Test the image
docker run --rm -d \
  --name usdfc-test \
  -p 3000:3000 \
  -e RPC_URL="https://your-rpc-url" \
  -e SUBGRAPH_URL="https://your-subgraph-url" \
  usdfc-terminal:local

# Check logs
docker logs usdfc-test

# Test health endpoint
curl http://localhost:3000/api/health

# Stop container
docker stop usdfc-test
```

### Trigger GitHub Actions Build

**Option 1: Push to main branch**
```bash
git add .
git commit -m "feat: add week 2 docker ci/cd pipeline"
git push origin main
```

**Option 2: Manual workflow dispatch**
```bash
# Trigger via GitHub CLI
gh workflow run docker-build.yml

# Watch the run
gh run watch

# View logs
gh run view --log
```

**Check build results:**
- Go to: `https://github.com/YOUR_USERNAME/usdfc-terminal/actions`
- Click on latest "Docker Build & Push to GHCR" run
- Expand steps to see:
  - Build time for each stage
  - sccache statistics (cache hits/misses)
  - Final image tags
  - Push confirmation

---

## Expected Performance

### Build Times (GitHub Actions)

**Cold Build (First Run - Empty Caches):**
- cargo-chef install: ~30s
- Recipe generation: ~10s
- Dependency build: ~5-7 minutes
- Application build: ~2-3 minutes
- **Total: 8-12 minutes**

**Warm Build (Cached Dependencies):**
- cargo-chef (cached): ~5s
- Recipe (cached): ~5s
- Dependencies (cached): ~30s
- Application build: ~2-3 minutes (with sccache S3)
- **Total: 3-5 minutes**

**Hot Build (Only Code Changes):**
- All stages cached except application: ~1-2 minutes
- **Total: 1-2 minutes**

### Cache Statistics

**sccache S3 performance:**
- First build: 0% cache hits (populating S3)
- Second build: 60-80% cache hits
- Third+ builds: 80-95% cache hits
- Cache size: ~300-500 MiB in S3

**Docker layer cache:**
- cargo-chef recipe layer: Invalidated only when dependencies change
- Dependency build layer: Cached unless Cargo.toml/Cargo.lock changes
- Application build layer: Invalidated on any source code change

---

## Railway Integration

### Option A: Pull Pre-built Image (Recommended)

Configure Railway to pull from GHCR instead of building:

**railway.toml:**
```toml
[build]
builder = "DOCKERFILE"
dockerfilePath = "Dockerfile.railway"

[deploy]
healthcheckPath = "/api/health"
healthcheckTimeout = 100
restartPolicyType = "ON_FAILURE"
```

**Create Dockerfile.railway (simple pull):**
```dockerfile
FROM ghcr.io/YOUR_USERNAME/usdfc-terminal:latest
```

**Benefits:**
- Railway deployment: 30-60 seconds (just pulls image)
- No build timeout issues
- Consistent images between CI and production
- Faster rollbacks (just change tag)

### Option B: GitHub Actions → Railway Webhook

Trigger Railway deployment after successful image push:

1. Get Railway webhook URL:
   ```bash
   # In Railway dashboard → Project → Settings → Webhooks
   # Create webhook, copy URL
   ```

2. Add to GitHub secrets:
   ```bash
   gh secret set RAILWAY_WEBHOOK_URL --body "https://..."
   ```

3. Workflow automatically calls webhook on push to main

---

## Monitoring & Debugging

### View sccache Statistics

In GitHub Actions logs, look for "Show sccache stats" step:
```
Compile requests                   1500
Cache hits                         1200
Cache misses                        300
Cache hit rate                   80.00 %
```

### Check Docker Layer Cache

GitHub Actions cache dashboard:
- Settings → Actions → Caches
- Look for keys like `buildkit-ghcr.io-...`
- Size should be 1-3 GB

### Troubleshoot Failed Builds

**Common issues:**

1. **S3 access denied:**
   - Check AWS credentials in secrets
   - Verify IAM policy allows s3:PutObject and s3:GetObject
   - Confirm bucket name matches SCCACHE_BUCKET

2. **GHCR push denied:**
   - Enable write permissions in Actions settings
   - Verify GITHUB_TOKEN has packages:write scope

3. **Docker build timeout:**
   - Default GitHub Actions timeout: 6 hours (should be plenty)
   - If hitting limits, optimize Dockerfile further

---

## Cost Estimation

### AWS S3 Costs

**Storage:**
- Cache size: ~300-500 MiB
- S3 Standard: $0.023/GB/month
- Monthly cost: ~$0.01-$0.02

**Requests:**
- ~100 builds/month × 500 requests/build = 50,000 requests
- PUT requests: $0.005/1000 = $0.25
- GET requests: $0.0004/1000 = $0.02
- Monthly cost: ~$0.27

**Total AWS cost: ~$0.30/month**

### GitHub Actions Minutes

- Free tier: 2,000 minutes/month (private repos) or unlimited (public repos)
- Build time: 3-5 minutes/build
- Builds/month: ~100-200
- Usage: 300-1000 minutes/month (well within free tier)

**Total cost: $0 for public repos, $0 for low-volume private repos**

---

## Next Steps

- [ ] Create AWS S3 bucket
- [ ] Create IAM user and access keys
- [ ] Add GitHub secrets
- [ ] Enable GHCR permissions
- [ ] Test local Docker build
- [ ] Push to trigger CI build
- [ ] Verify image in GHCR
- [ ] Configure Railway to pull image
- [ ] Test Railway deployment
- [ ] Document Week 2 results

---

**Last updated:** 2026-01-06 02:55 UTC
**Status:** Ready for testing
**Next:** AWS setup and first CI build
