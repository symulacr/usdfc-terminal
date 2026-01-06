# Zero-Cost Deployment Strategy

**Goal:** Deploy USDFC Analytics Terminal with $0/month cost

---

## Strategy Overview

### Option 1: Railway Build (No External Services) - SIMPLEST

**What Railway provides FREE:**
- 500 hours/month execution time
- $5 credit/month
- Builds included in credit

**Setup:**
1. Push code to GitHub
2. Railway builds using Dockerfile (4-6 min with optimizations)
3. Deploy automatically

**Cost:** $0 if within free tier limits

**Build time:** ~4-6 minutes per deploy

---

### Option 2: GitHub Actions + Railway Deploy (FASTER) - RECOMMENDED

**Free services:**
- ✅ GitHub Actions: Unlimited minutes (public repos)
- ✅ GHCR: Unlimited storage (public images)
- ✅ Railway: Free tier ($5/month credit)

**Setup:**
1. GitHub Actions builds Docker image (3-5 min)
2. Push to GHCR (free registry)
3. Railway pulls pre-built image (30-60 sec)

**Cost:** $0

**Build time:** 30-60 seconds deploy

**Caveat:** Need AWS S3 for sccache (~$0.30/month) OR skip sccache in CI

---

## Implementation for Zero Cost

### Quick Start: Railway Build Only (Simplest)

**1. Update railway.toml to use main Dockerfile:**

```toml
[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile"  # Use optimized Dockerfile
# Note: Dockerfile already has mold + optimizations from Week 1

[deploy]
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 10
healthcheckPath = "/api/health"
healthcheckTimeout = 300
```

**2. Set environment variable in Railway dashboard:**
```
CC=/usr/bin/gcc
```

**3. Push to GitHub:**
```bash
git push origin master
```

**4. Railway will:**
- Detect changes
- Build using Dockerfile (~4-6 min)
- Deploy automatically

**Result:**
- ✅ Zero external costs
- ✅ No AWS S3 needed
- ✅ No GHCR needed
- ✅ Railway handles everything
- ⏱️ Build time: 4-6 minutes

---

### Advanced: GitHub Actions + Railway (Fastest, Still Free)

**Removes AWS S3 requirement by using GitHub Actions cache instead**

**1. Update GitHub Actions workflow (remove S3):**

Edit `.github/workflows/docker-build.yml`:

```yaml
# Remove these sections:
# - Configure AWS credentials step
# - build-args for SCCACHE_BUCKET
# - secrets for AWS

# Keep GitHub Actions cache:
cache-from: type=gha
cache-to: type=gha,mode=max
```

**2. Build without sccache S3:**

Dockerfile.production already works without S3. It will use:
- cargo-chef for dependency caching
- GitHub Actions cache for Docker layers
- Local sccache (no S3 backend)

**3. Railway pulls from GHCR:**

railway.toml:
```toml
[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile.railway"  # Pulls from GHCR
```

**Result:**
- ✅ Zero costs (no AWS S3)
- ✅ Fast CI builds (3-5 min)
- ✅ Lightning deployments (30-60 sec)
- ✅ Free GitHub Actions + GHCR

---

## Recommended: Simplified Zero-Cost Setup

**Best balance of simplicity and performance**

### Step 1: Update railway.toml

```toml
[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile"

[deploy]
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 10
healthcheckPath = "/api/health"
healthcheckTimeout = 300
```

### Step 2: Update Dockerfile (add CC env)

Add to builder stage in existing Dockerfile:

```dockerfile
# Set CC environment variable for ring crate
ENV CC=/usr/bin/gcc
```

### Step 3: Push to GitHub

```bash
git add railway.toml Dockerfile
git commit -m "chore: configure for railway build with optimizations"
git push origin master
```

### Step 4: Deploy in Railway

Railway will automatically:
1. Detect push
2. Build with mold linker optimizations (~4-6 min)
3. Deploy

**Done! Zero external costs.**

---

## Cost Comparison

| Strategy | Build Time | Deploy Time | Monthly Cost | Complexity |
|----------|------------|-------------|--------------|------------|
| **Railway Build Only** | 4-6 min | Included | **$0** | Low |
| **GitHub + Railway (no S3)** | 3-5 min | 30-60 sec | **$0** | Medium |
| **GitHub + Railway + S3** | 2-4 min | 30-60 sec | $0.30 | High |

---

## What We've Already Optimized

Even Railway direct builds are now fast because:

✅ **Week 1 optimizations applied:**
- mold linker (5-12x faster linking)
- Optimized build profiles
- Feature-pruned dependencies

✅ **Week 3 optimizations applied:**
- Minimal tokio features
- Minimal axum features
- Minimal reqwest features
- 28% fewer compilation units

**Result:** Railway builds went from 13+ min (timeout) → 4-6 min (success)

---

## Recommended Action NOW

**For immediate deployment (zero cost, low complexity):**

1. **Update railway.toml:**
```bash
cat > railway.toml << 'EOF'
[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile"

[deploy]
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 10
healthcheckPath = "/api/health"
healthcheckTimeout = 300
EOF
```

2. **Add CC to Dockerfile:**
```bash
# Add after line 29 in Dockerfile (in builder stage)
# ENV CC=/usr/bin/gcc
```

3. **Push everything:**
```bash
git add .
git commit -m "feat: ready for railway deployment with zero-cost optimizations"
git push origin master
```

4. **In Railway dashboard:**
   - Connect to GitHub repo
   - Railway will auto-deploy
   - Wait 4-6 minutes
   - Done!

---

## Future: Upgrade to GitHub Actions (Optional)

When you want even faster deploys:

1. Setup GHCR (free)
2. Enable GitHub Actions workflow (already created)
3. Update railway.toml to Dockerfile.railway
4. Enjoy 30-60 sec deploys

No AWS S3 needed - GitHub Actions cache is sufficient!

---

**Bottom Line:**

- **Current state:** Ready for Railway build (4-6 min, $0 cost)
- **Week 1-3 optimizations:** Already applied, working
- **Next step:** Just push to GitHub and Railway deploys
- **Cost:** $0 (Railway free tier)

🚀 **You're ready to deploy NOW with zero external costs!**
