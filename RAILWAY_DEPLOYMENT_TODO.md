# Railway Deployment Optimization - Complete TODO Plan
## Production Implementation Checklist

**Goal:** Reduce Railway deployment from 13+ min (timeout) to 4-6 min (git push → live)
**Strategy:** Build in GitHub Actions, deploy pre-built image to Railway

---

## Pre-Implementation Checklist

### Setup & Prerequisites

- [ ] **Verify GitHub repository exists**
  ```bash
  git remote -v
  # Should show GitHub URL
  ```

- [ ] **Create GitHub account tokens/access**
  - [ ] GitHub account with admin access to repository
  - [ ] Ability to add secrets to repository
  - [ ] GitHub Container Registry (GHCR) access enabled

- [ ] **Setup AWS account for S3**
  - [ ] Create AWS account (free tier)
  - [ ] Note: AWS Account ID: `________________`
  - [ ] Install AWS CLI locally
  - [ ] Configure AWS credentials: `aws configure`

- [ ] **Verify Railway access**
  - [ ] Railway account created
  - [ ] Project exists: `________________`
  - [ ] Service name: `________________`
  - [ ] Can access Railway CLI: `railway --version`

- [ ] **Document baseline metrics**
  - [ ] Current Railway build time: `13+ minutes (timeout)`
  - [ ] Current deployment status: `FAILING`
  - [ ] Target build time: `3-5 minutes (CI)`
  - [ ] Target deployment time: `30-60 seconds (Railway)`
  - [ ] Target total time: `4-6 minutes (git push → live)`

---

## Week 1: Foundation & Baseline (8 hours)

### Day 1: Monday (4 hours) - Establish Baseline

#### Task 1.1: Generate Build Timing Report

- [ ] **Clean build environment**
  ```bash
  cd /home/eya/claude/usdfc-terminal
  cargo clean
  ```

- [ ] **Run timed build with timing report**
  ```bash
  time cargo leptos build --release -p usdfc-analytics-terminal --timings
  ```
  - [ ] Record total time: `_______ minutes _______ seconds`
  - [ ] Save start time: `_______`
  - [ ] Save end time: `_______`

- [ ] **Open and analyze timing HTML report**
  ```bash
  ls -lh target/cargo-timings/
  # Open in browser: target/cargo-timings/cargo-timing-*.html
  ```

- [ ] **Identify bottlenecks in report**
  - [ ] Top 10 slowest crates documented
  - [ ] Crate #1: `____________ - _____ seconds`
  - [ ] Crate #2: `____________ - _____ seconds`
  - [ ] Crate #3: `____________ - _____ seconds`
  - [ ] Crate #4: `____________ - _____ seconds`
  - [ ] Crate #5: `____________ - _____ seconds`
  - [ ] Crate #6: `____________ - _____ seconds`
  - [ ] Crate #7: `____________ - _____ seconds`
  - [ ] Crate #8: `____________ - _____ seconds`
  - [ ] Crate #9: `____________ - _____ seconds`
  - [ ] Crate #10: `____________ - _____ seconds`

- [ ] **Identify proc-macro heavy crates**
  - [ ] serde_derive: `Yes / No`
  - [ ] syn: `Yes / No`
  - [ ] tokio-macros: `Yes / No`
  - [ ] leptos_macro: `Yes / No`
  - [ ] Other: `____________`

- [ ] **Estimate link time vs codegen**
  - [ ] Estimated codegen time: `_______ seconds`
  - [ ] Estimated link time: `_______ seconds`
  - [ ] Link time percentage: `_______ %`

- [ ] **Create BASELINE_MEASUREMENTS.md**
  ```bash
  cat > BASELINE_MEASUREMENTS.md << 'EOF'
  # Baseline Build Performance

  Date: $(date)
  Machine: $(uname -a)
  Rustc: $(rustc --version)
  Cargo: $(cargo --version)

  ## Cold Build (cargo clean)
  Total time: XXX minutes XXX seconds

  ## Top 10 Slowest Crates
  1. crate_name - XXs
  2. ...

  ## Bottleneck Analysis
  - Link time estimate: XXs (XX%)
  - Codegen time estimate: XXs (XX%)
  - Heaviest proc-macro crates: [list]
  - Serial bottlenecks: [list]

  ## Current State
  - Local cold build: XXXs
  - Railway build: 13+ min (timeout ❌)

  ## Targets
  - Local warm build: 2-4 min
  - CI cold build: 8-12 min
  - CI warm build: 3-5 min
  - Railway deploy: 30-60s
  EOF
  ```
  - [ ] File created
  - [ ] All data filled in
  - [ ] Committed to git

#### Task 1.2: Analyze Current Dependencies

- [ ] **Check current feature usage**
  ```bash
  cargo tree -e features --workspace | grep -E "tokio|axum|tower|leptos|serde" > /tmp/current-features.txt
  ```

- [ ] **Review tokio features**
  ```bash
  grep -A 5 "tokio" Cargo.toml crates/*/Cargo.toml
  ```
  - [ ] Documented current tokio features: `____________`
  - [ ] Identified unnecessary features: `____________`

- [ ] **Review axum features**
  ```bash
  grep -A 5 "axum" Cargo.toml crates/*/Cargo.toml
  ```
  - [ ] Documented current axum features: `____________`
  - [ ] Identified unnecessary features: `____________`

- [ ] **Review tower features**
  ```bash
  grep -A 5 "tower" Cargo.toml crates/*/Cargo.toml
  ```
  - [ ] Documented current tower features: `____________`
  - [ ] Identified unnecessary features: `____________`

- [ ] **Check for default-features = false**
  ```bash
  grep -r "default-features" Cargo.toml crates/*/Cargo.toml
  ```
  - [ ] Count dependencies with default-features disabled: `_____`
  - [ ] Count dependencies using default features: `_____`
  - [ ] List candidates for feature pruning: `____________`

---

### Day 2: Tuesday (4 hours) - Install Core Tools

#### Task 1.3: Install and Configure mold Linker

- [ ] **Check current OS**
  ```bash
  cat /etc/os-release
  uname -a
  ```
  - [ ] OS identified: `____________`

- [ ] **Install mold (local development)**
  ```bash
  # For Ubuntu/Debian:
  sudo apt-get update
  sudo apt-get install -y mold clang

  # Verify
  mold --version
  which mold
  ```
  - [ ] mold installed successfully
  - [ ] Version: `____________`
  - [ ] Path: `____________`

- [ ] **Create .cargo/config.toml**
  ```bash
  mkdir -p .cargo
  cat > .cargo/config.toml << 'EOF'
  # Production-grade Cargo configuration

  [build]
  rustc-wrapper = "sccache"

  [target.x86_64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]

  [target.x86_64-unknown-linux-musl]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  EOF
  ```
  - [ ] File created at `.cargo/config.toml`
  - [ ] Verified contents are correct

- [ ] **Update Cargo.toml with CI profile**
  ```toml
  # Add to Cargo.toml after existing profiles

  [profile.ci]
  inherits = "release"
  lto = false              # Disable LTO for faster linking
  opt-level = 2            # Good optimization
  codegen-units = 16       # Maximum parallelism
  strip = true
  panic = "abort"
  debug = false
  incremental = false      # Always false in CI

  [profile.production]
  inherits = "release"
  lto = "thin"             # Thin LTO for production
  opt-level = 3
  codegen-units = 1
  strip = true
  panic = "abort"
  debug = false
  incremental = false
  ```
  - [ ] CI profile added to Cargo.toml
  - [ ] Production profile added to Cargo.toml
  - [ ] Verified syntax is correct

- [ ] **Test mold linker is working**
  ```bash
  cargo clean
  time cargo leptos build --release -p usdfc-analytics-terminal 2>&1 | tee /tmp/build-with-mold.log
  ```
  - [ ] Build completed successfully
  - [ ] Build time recorded: `_______ seconds`
  - [ ] Compared to baseline: `_______ seconds` (Δ `_______ %`)
  - [ ] Log saved for reference

- [ ] **Verify mold was used**
  ```bash
  grep -i "mold\|linker" /tmp/build-with-mold.log
  ```
  - [ ] mold linker confirmed in use

#### Task 1.4: Install and Configure sccache

- [ ] **Install sccache**
  ```bash
  cargo install sccache --locked
  ```
  - [ ] Installation successful
  - [ ] Time taken: `_______ minutes`

- [ ] **Verify sccache installation**
  ```bash
  sccache --version
  which sccache
  ```
  - [ ] Version: `____________`
  - [ ] Path: `____________`

- [ ] **Configure local sccache**
  ```bash
  export SCCACHE_DIR="$HOME/.cache/sccache"
  export SCCACHE_CACHE_SIZE="10G"

  # Add to shell profile for persistence
  echo 'export SCCACHE_DIR="$HOME/.cache/sccache"' >> ~/.bashrc
  echo 'export SCCACHE_CACHE_SIZE="10G"' >> ~/.bashrc
  ```
  - [ ] Environment variables set
  - [ ] Added to shell profile

- [ ] **Test sccache - Cold build**
  ```bash
  # Stop any running sccache server
  sccache --stop-server

  # Clear cache
  rm -rf ~/.cache/sccache

  # Cold build
  cargo clean
  time cargo leptos build --release -p usdfc-analytics-terminal 2>&1 | tee /tmp/sccache-cold.log

  # Check stats
  sccache --show-stats > /tmp/sccache-cold-stats.txt
  ```
  - [ ] Cold build completed
  - [ ] Cold build time: `_______ seconds`
  - [ ] Stats saved
  - [ ] Cache hits: `_____`
  - [ ] Cache misses: `_____`
  - [ ] Compilation requests: `_____`

- [ ] **Test sccache - Warm build**
  ```bash
  # Warm build (clean but sccache populated)
  cargo clean
  time cargo leptos build --release -p usdfc-analytics-terminal 2>&1 | tee /tmp/sccache-warm.log

  # Check stats
  sccache --show-stats > /tmp/sccache-warm-stats.txt
  ```
  - [ ] Warm build completed
  - [ ] Warm build time: `_______ seconds`
  - [ ] Stats saved
  - [ ] Cache hits: `_____`
  - [ ] Cache misses: `_____`
  - [ ] Hit rate: `_______ %`
  - [ ] Speedup: `_______ %` faster than cold

- [ ] **Document sccache performance**
  ```bash
  cat >> BASELINE_MEASUREMENTS.md << EOF

  ## Week 1: mold + sccache Results

  ### Local Builds
  - Baseline (no optimization): XXXs
  - With mold only: XXXs (XX% improvement)
  - With mold + sccache (cold): XXXs
  - With mold + sccache (warm): XXXs (XX% improvement)

  ### sccache Statistics
  - Cache hits (warm): XX%
  - Cache misses (cold): XX%
  - Cache size: XX MB

  ### Link Time Improvement
  - Before mold: XXXs
  - After mold: XXXs (XX% faster)
  EOF
  ```
  - [ ] Performance documented

---

### Day 3: Wednesday (Optional) - Commit and Document

- [ ] **Commit Week 1 progress**
  ```bash
  git add .cargo/config.toml
  git add Cargo.toml
  git add BASELINE_MEASUREMENTS.md
  git commit -m "week 1: add mold linker and sccache config with ci profile"
  git push origin main
  ```
  - [ ] Committed successfully
  - [ ] Pushed to remote

- [ ] **Create Week 1 summary**
  - [ ] Baseline build time: `_______`
  - [ ] With mold: `_______` (`_____% improvement`)
  - [ ] With sccache warm: `_______` (`_____% improvement`)
  - [ ] Total local improvement: `_____% faster`

---

## Week 2: CI Pipeline & Docker (12 hours)

### Day 1: Monday (4 hours) - Production Dockerfile

#### Task 2.1: Create Dockerfile.production

- [ ] **Create Dockerfile.production**
  ```bash
  # Copy template from PRODUCTION_IMPLEMENTATION_PLAN.md
  # Section: "Week 2: Day 1: Monday - Task 2.1"
  ```
  - [ ] File created: `Dockerfile.production`
  - [ ] Syntax verified: `docker build -f Dockerfile.production --help`

- [ ] **Verify Dockerfile stages**
  - [ ] Stage 1: chef (cargo-chef install)
  - [ ] Stage 2: planner (recipe.json generation)
  - [ ] Stage 3: builder (dependency + app build)
  - [ ] Stage 4: runtime (minimal final image)

- [ ] **Test Dockerfile locally (without S3)**
  ```bash
  # Build without secrets (will use local cache only)
  DOCKER_BUILDKIT=1 docker build \
    -f Dockerfile.production \
    -t usdfc-terminal:week2-test \
    . 2>&1 | tee /tmp/docker-build-test.log
  ```
  - [ ] Build started successfully
  - [ ] chef stage completed
  - [ ] planner stage completed
  - [ ] builder stage completed
  - [ ] runtime stage completed
  - [ ] Total build time: `_______ minutes`
  - [ ] Log saved

- [ ] **Test Docker image runs**
  ```bash
  # Run container locally
  docker run --rm -d \
    --name usdfc-test \
    -p 3000:3000 \
    -e PORT=3000 \
    -e HOST=0.0.0.0 \
    -e RPC_URL="${RPC_URL}" \
    -e SUBGRAPH_URL="${SUBGRAPH_URL}" \
    # ... add other required env vars
    usdfc-terminal:week2-test

  # Check if running
  docker ps | grep usdfc-test

  # Check logs
  docker logs usdfc-test

  # Test health endpoint
  sleep 10
  curl http://localhost:3000/api/health

  # Stop container
  docker stop usdfc-test
  ```
  - [ ] Container started successfully
  - [ ] Health check returns "healthy"
  - [ ] Application accessible at http://localhost:3000
  - [ ] Logs show no errors
  - [ ] Container stopped cleanly

- [ ] **Analyze Docker image size**
  ```bash
  docker images | grep usdfc-terminal
  docker history usdfc-terminal:week2-test
  ```
  - [ ] Final image size: `_______ MB`
  - [ ] Largest layers identified: `____________`

---

### Day 2: Tuesday (4 hours) - S3 Setup for sccache

#### Task 2.2: AWS S3 Bucket Configuration

- [ ] **Install AWS CLI**
  ```bash
  # Ubuntu/Debian:
  sudo apt-get install -y awscli

  # Or via pip:
  pip3 install --user awscli

  # Verify
  aws --version
  ```
  - [ ] AWS CLI installed
  - [ ] Version: `____________`

- [ ] **Configure AWS credentials**
  ```bash
  aws configure
  # Enter:
  # - AWS Access Key ID: [your-access-key]
  # - AWS Secret Access Key: [your-secret-key]
  # - Default region: us-east-1 (or your preference)
  # - Default output format: json
  ```
  - [ ] Credentials configured
  - [ ] Region set: `____________`

- [ ] **Create S3 bucket**
  ```bash
  BUCKET_NAME="usdfc-terminal-sccache-$(date +%s)"
  REGION="us-east-1"

  # Create bucket
  aws s3 mb s3://${BUCKET_NAME} --region ${REGION}

  # Verify
  aws s3 ls s3://${BUCKET_NAME}
  ```
  - [ ] Bucket created successfully
  - [ ] Bucket name: `________________________________`
  - [ ] Region: `____________`

- [ ] **Enable versioning (optional)**
  ```bash
  aws s3api put-bucket-versioning \
    --bucket ${BUCKET_NAME} \
    --versioning-configuration Status=Enabled
  ```
  - [ ] Versioning enabled

- [ ] **Set lifecycle policy**
  ```bash
  cat > /tmp/lifecycle.json << 'EOF'
  {
    "Rules": [
      {
        "Id": "Delete old cache after 30 days",
        "Status": "Enabled",
        "Prefix": "",
        "Expiration": {
          "Days": 30
        }
      }
    ]
  }
  EOF

  aws s3api put-bucket-lifecycle-configuration \
    --bucket ${BUCKET_NAME} \
    --lifecycle-configuration file:///tmp/lifecycle.json
  ```
  - [ ] Lifecycle policy created
  - [ ] Verified: Objects will be deleted after 30 days

- [ ] **Create IAM user for CI**
  ```bash
  # Create user
  aws iam create-user --user-name github-actions-sccache
  ```
  - [ ] IAM user created
  - [ ] Username: `github-actions-sccache`

- [ ] **Create IAM policy**
  ```bash
  ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

  cat > /tmp/sccache-policy.json << EOF
  {
    "Version": "2012-10-17",
    "Statement": [
      {
        "Effect": "Allow",
        "Action": [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ],
        "Resource": "arn:aws:s3:::${BUCKET_NAME}/*"
      },
      {
        "Effect": "Allow",
        "Action": [
          "s3:ListBucket"
        ],
        "Resource": "arn:aws:s3:::${BUCKET_NAME}"
      }
    ]
  }
  EOF

  aws iam create-policy \
    --policy-name SccacheBucketAccess \
    --policy-document file:///tmp/sccache-policy.json
  ```
  - [ ] Policy created
  - [ ] Policy ARN: `________________________________`

- [ ] **Attach policy to user**
  ```bash
  aws iam attach-user-policy \
    --user-name github-actions-sccache \
    --policy-arn arn:aws:iam::${ACCOUNT_ID}:policy/SccacheBucketAccess
  ```
  - [ ] Policy attached

- [ ] **Create access keys**
  ```bash
  aws iam create-access-key --user-name github-actions-sccache > /tmp/aws-keys.json

  # View keys (SAVE THESE!)
  cat /tmp/aws-keys.json
  ```
  - [ ] Access keys created
  - [ ] ⚠️ **SAVE THESE SECURELY:**
    - Access Key ID: `________________________________`
    - Secret Access Key: `________________________________`

- [ ] **Test S3 access**
  ```bash
  # Upload test file
  echo "test" > /tmp/test.txt
  aws s3 cp /tmp/test.txt s3://${BUCKET_NAME}/test.txt

  # List
  aws s3 ls s3://${BUCKET_NAME}/

  # Download
  aws s3 cp s3://${BUCKET_NAME}/test.txt /tmp/test-download.txt

  # Verify
  cat /tmp/test-download.txt

  # Cleanup
  aws s3 rm s3://${BUCKET_NAME}/test.txt
  ```
  - [ ] Upload successful
  - [ ] List successful
  - [ ] Download successful
  - [ ] S3 access confirmed working

- [ ] **Test sccache with S3**
  ```bash
  # Configure sccache for S3
  export SCCACHE_BUCKET="${BUCKET_NAME}"
  export SCCACHE_REGION="${REGION}"
  export AWS_ACCESS_KEY_ID="<from-aws-keys.json>"
  export AWS_SECRET_ACCESS_KEY="<from-aws-keys.json>"

  # Stop local sccache
  sccache --stop-server

  # Clear local cache
  rm -rf ~/.cache/sccache

  # Test build with S3
  cargo clean
  time cargo leptos build --profile ci -p usdfc-analytics-terminal

  # Check stats
  sccache --show-stats
  ```
  - [ ] Build completed with S3 backend
  - [ ] Build time: `_______ seconds`
  - [ ] sccache stats show S3 usage
  - [ ] Cache storage location shows S3

- [ ] **Verify objects in S3**
  ```bash
  aws s3 ls s3://${BUCKET_NAME}/ --recursive | head -20
  ```
  - [ ] Objects created in S3
  - [ ] Count: `_____` objects
  - [ ] Total size: `_______ MB`

---

### Day 3: Wednesday (4 hours) - GitHub Actions Setup

#### Task 2.3: GitHub Repository Configuration

- [ ] **Verify GitHub repository**
  ```bash
  git remote -v
  # Should show GitHub URL
  ```
  - [ ] Repository URL: `________________________________`
  - [ ] Have admin access: `Yes / No`

- [ ] **Enable GitHub Container Registry (GHCR)**
  - [ ] Go to GitHub profile → Settings → Developer settings → Personal access tokens
  - [ ] Create token with `write:packages` scope (if needed)
  - [ ] Token saved securely: `________________________________`

#### Task 2.4: Add GitHub Secrets

- [ ] **Navigate to repository secrets**
  - Go to: `https://github.com/<owner>/<repo>/settings/secrets/actions`

- [ ] **Add S3 secrets**
  - [ ] `SCCACHE_BUCKET` = `________________________________`
  - [ ] `SCCACHE_REGION` = `us-east-1` (or your region)
  - [ ] `AWS_ACCESS_KEY_ID` = `________________________________`
  - [ ] `AWS_SECRET_ACCESS_KEY` = `________________________________`

- [ ] **Add Railway token**
  - [ ] Go to Railway dashboard → Account → Tokens
  - [ ] Create new token: `________________________________`
  - [ ] Add to GitHub secrets: `RAILWAY_TOKEN` = `________________________________`

- [ ] **Verify all secrets added**
  - [ ] SCCACHE_BUCKET
  - [ ] SCCACHE_REGION
  - [ ] AWS_ACCESS_KEY_ID
  - [ ] AWS_SECRET_ACCESS_KEY
  - [ ] RAILWAY_TOKEN

#### Task 2.5: Create GitHub Actions Workflow

- [ ] **Create workflow directory**
  ```bash
  mkdir -p .github/workflows
  ```

- [ ] **Create ci-cd.yml**
  ```bash
  # Copy template from PRODUCTION_IMPLEMENTATION_PLAN.md
  # Section: "Week 2: Day 3: Task 2.3"
  ```
  - [ ] File created: `.github/workflows/ci-cd.yml`
  - [ ] Syntax verified

- [ ] **Verify workflow structure**
  - [ ] Job: build-and-push
  - [ ] Job: deploy-to-railway
  - [ ] Checkout step
  - [ ] Docker Buildx setup
  - [ ] GHCR login
  - [ ] Docker metadata
  - [ ] Build and push with secrets
  - [ ] Build time calculation
  - [ ] Build time budget check
  - [ ] Railway deployment

- [ ] **Commit and push workflow**
  ```bash
  git add .github/workflows/ci-cd.yml
  git add Dockerfile.production
  git commit -m "week 2: add production dockerfile and github actions ci/cd"
  git push origin main
  ```
  - [ ] Committed successfully
  - [ ] Pushed to remote

#### Task 2.6: Monitor First CI Build

- [ ] **Watch GitHub Actions**
  - Go to: `https://github.com/<owner>/<repo>/actions`
  - [ ] Workflow triggered automatically
  - [ ] Job `build-and-push` started

- [ ] **Monitor build progress**
  - [ ] Checkout completed
  - [ ] Docker Buildx setup completed
  - [ ] Login to GHCR completed
  - [ ] Extract metadata completed
  - [ ] Build started
  - [ ] ... Monitor in real-time ...

- [ ] **Record first build metrics**
  - [ ] Build start time: `_______`
  - [ ] Build end time: `_______`
  - [ ] Total build time: `_______ minutes`
  - [ ] Within 12-minute budget? `Yes / No`
  - [ ] sccache stats from logs:
    - Cache hits: `_____`
    - Cache misses: `_____`
    - Hit rate: `_______%`

- [ ] **Verify image pushed to GHCR**
  - Go to: `https://github.com/<owner>/<repo>/pkgs/container/<image>`
  - [ ] Image visible
  - [ ] Tag: `sha-<commit>`
  - [ ] Tag: `latest`
  - [ ] Image size: `_______ MB`

- [ ] **Check Railway deployment**
  - [ ] Job `deploy-to-railway` started
  - [ ] Railway CLI installed
  - [ ] Deployment triggered
  - [ ] Deployment status: `Success / Failure`

- [ ] **Test second build (warm cache)**
  ```bash
  # Make trivial change to trigger rebuild
  echo "# Week 2 complete" >> README.md
  git add README.md
  git commit -m "test: trigger warm cache build"
  git push origin main
  ```
  - [ ] Second build triggered
  - [ ] Build time: `_______ minutes` (should be 3-5 min!)
  - [ ] sccache hit rate: `_______%` (should be >80%)

- [ ] **Document Week 2 results**
  ```bash
  cat >> BASELINE_MEASUREMENTS.md << EOF

  ## Week 2: CI/CD Pipeline Results

  ### GitHub Actions Builds
  - First build (cold cache): XX min XXs
  - Second build (warm cache): XX min XXs
  - Improvement: XX%

  ### sccache S3 Performance
  - Cold build hit rate: XX%
  - Warm build hit rate: XX%
  - S3 bucket size: XX MB

  ### GHCR Image
  - Image size: XX MB
  - Push time: XXs

  ### Railway Deployment
  - Pull image: XXs
  - Start container: XXs
  - Health check: XXs
  - Total deploy time: XXs
  EOF
  ```

---

## Week 3: Optimization & Refinement (6 hours)

### Day 1: Monday (3 hours) - Dependency Optimization

#### Task 3.1: Feature Pruning

- [ ] **Review current workspace dependencies**
  ```bash
  grep -A 10 "\[workspace.dependencies\]" Cargo.toml
  ```

- [ ] **Prune tokio features**
  - [ ] Current features: `____________`
  - [ ] Update to minimal:
    ```toml
    tokio = { version = "1", default-features = false, features = [
        "rt-multi-thread",
        "macros",
        "signal",
    ]}
    ```
  - [ ] Verify still compiles
  - [ ] If compilation errors, add back needed features

- [ ] **Prune axum features**
  - [ ] Current features: `____________`
  - [ ] Update to minimal:
    ```toml
    axum = { version = "0.7", default-features = false, features = [
        "macros",
        "json",
        "tokio",
        "http1",
        "http2",
    ]}
    ```
  - [ ] Verify still compiles

- [ ] **Prune tower features**
  - [ ] Current features: `____________`
  - [ ] Update to minimal:
    ```toml
    tower = { version = "0.4", default-features = false, features = [
        "util",
    ]}
    ```
  - [ ] Verify still compiles

- [ ] **Prune tower-http features**
  - [ ] Current features: `____________`
  - [ ] Update to minimal:
    ```toml
    tower-http = { version = "0.5", default-features = false, features = [
        "fs",
        "cors",
        "compression-gzip",
    ]}
    ```
  - [ ] Verify still compiles

- [ ] **Prune chrono features**
  - [ ] Current features: `____________`
  - [ ] Update to minimal:
    ```toml
    chrono = { version = "0.4", default-features = false, features = [
        "serde",
        "clock",
        "alloc",
    ]}
    ```
  - [ ] Verify still compiles

- [ ] **Test application still works**
  ```bash
  cargo leptos build --profile ci -p usdfc-analytics-terminal

  # Run locally
  ./target/ci/usdfc-analytics-terminal

  # Test endpoints
  curl http://localhost:3000/api/health
  curl http://localhost:3000/api/health/detailed
  ```
  - [ ] Build successful
  - [ ] Application starts
  - [ ] Health endpoints work
  - [ ] No runtime errors

- [ ] **Measure build time improvement**
  ```bash
  cargo clean
  time cargo leptos build --profile ci -p usdfc-analytics-terminal
  ```
  - [ ] Build time: `_______ seconds`
  - [ ] Compared to Week 2: `_______ seconds` (Δ `_____s`)
  - [ ] Improvement: `_______%`

- [ ] **Commit feature pruning**
  ```bash
  git add Cargo.toml
  git commit -m "week 3: feature pruning - tokio axum tower chrono"
  git push origin main
  ```
  - [ ] Committed and pushed
  - [ ] CI build triggered
  - [ ] CI build time: `_______ minutes`

---

### Day 2: Tuesday (3 hours) - WASM Optimization

#### Task 3.2: Optimize WASM Settings

- [ ] **Review current wasm-opt settings**
  ```bash
  grep -A 5 "wasm-opt-features" crates/terminal/Cargo.toml
  ```
  - [ ] Current setting: `____________`

- [ ] **Change to -O2 (faster, slightly larger)**
  ```toml
  # In crates/terminal/Cargo.toml

  [package.metadata.leptos]
  # ... other settings ...
  wasm-opt-features = [
      "-O2",  # Changed from -Oz (40-60% faster, ~5-10% larger)
      "--enable-bulk-memory",
      "--enable-nontrapping-float-to-int",
      "--enable-sign-ext"
  ]
  ```
  - [ ] Updated setting
  - [ ] File saved

- [ ] **Test WASM build**
  ```bash
  cargo clean
  time cargo leptos build --profile ci -p usdfc-analytics-terminal
  ```
  - [ ] Build successful
  - [ ] Build time: `_______ seconds`
  - [ ] Compared to previous: `_____s faster`

- [ ] **Check WASM size**
  ```bash
  ls -lh target/site/pkg/*.wasm
  ```
  - [ ] WASM size with -Oz: `_______ MB` (from before)
  - [ ] WASM size with -O2: `_______ MB` (current)
  - [ ] Size increase: `_______ KB` or `_______%`
  - [ ] Acceptable? `Yes / No` (should be ~5-10% larger)

- [ ] **Test application performance**
  ```bash
  # Start server
  ./target/ci/usdfc-analytics-terminal &

  # Wait for startup
  sleep 5

  # Test page load (measure in browser)
  # Open: http://localhost:3000
  ```
  - [ ] Page loads correctly
  - [ ] No JavaScript errors in console
  - [ ] UI interactive
  - [ ] Performance acceptable

- [ ] **Commit WASM optimization**
  ```bash
  git add crates/terminal/Cargo.toml
  git commit -m "week 3: optimize wasm-opt from -Oz to -O2 for faster builds"
  git push origin main
  ```
  - [ ] Committed and pushed
  - [ ] CI build triggered
  - [ ] CI build time: `_______ minutes` (should be faster!)

- [ ] **Document Week 3 results**
  ```bash
  cat >> BASELINE_MEASUREMENTS.md << EOF

  ## Week 3: Optimization Results

  ### Feature Pruning Impact
  - Build time before: XX min XXs
  - Build time after: XX min XXs
  - Improvement: XX seconds (XX%)

  ### WASM Optimization Impact
  - Build time before (-Oz): XX min XXs
  - Build time after (-O2): XX min XXs
  - Improvement: XX seconds (XX%)
  - WASM size increase: XX KB (XX%)

  ### Combined Week 3 Impact
  - Total time saved: XX seconds
  - Current CI build time: XX min XXs
  - Target: <5 minutes ✓ / ✗
  EOF
  ```

---

## Week 4: Railway Integration & Production (4 hours)

### Day 1: Monday (2 hours) - Railway Configuration

#### Task 4.1: Configure Railway for Pre-Built Images

- [ ] **Update railway.toml**
  ```toml
  [build]
  builder = "DOCKERFILE"
  dockerfilePath = "Dockerfile.production"

  [deploy]
  restartPolicyType = "ON_FAILURE"
  restartPolicyMaxRetries = 10
  healthcheckPath = "/api/health"
  healthcheckTimeout = 300

  [experimental]
  incrementalDeployment = true
  ```
  - [ ] File updated
  - [ ] Settings verified

- [ ] **Verify Railway environment variables**
  ```bash
  railway variables
  ```
  - [ ] RPC_URL set
  - [ ] SUBGRAPH_URL set
  - [ ] BLOCKSCOUT_URL set
  - [ ] GECKOTERMINAL_URL set
  - [ ] All contract addresses set
  - [ ] All pool addresses set
  - [ ] HOST set
  - [ ] Database path set
  - [ ] Other required vars set

- [ ] **Test Railway deployment**
  ```bash
  # Make change to trigger deployment
  echo "# Week 4 deployment test" >> README.md
  git add README.md railway.toml
  git commit -m "week 4: configure railway for pre-built images"
  git push origin main
  ```
  - [ ] Git push successful
  - [ ] GitHub Actions triggered
  - [ ] Build job completed
  - [ ] Image pushed to GHCR
  - [ ] Railway job started
  - [ ] Railway deployment triggered

- [ ] **Monitor Railway deployment**
  ```bash
  railway logs --follow
  ```
  - [ ] Image pull started
  - [ ] Image pull completed: `_____ seconds`
  - [ ] Container start initiated
  - [ ] Container started: `_____ seconds`
  - [ ] Health check started
  - [ ] Health check passed: `_____ seconds`
  - [ ] Traffic switched
  - [ ] Total deployment time: `_____ seconds`

- [ ] **Verify application is live**
  ```bash
  # Get Railway URL
  RAILWAY_URL=$(railway variables --json | jq -r '.RAILWAY_STATIC_URL')

  # Test health endpoint
  curl https://${RAILWAY_URL}/api/health

  # Test detailed health
  curl https://${RAILWAY_URL}/api/health/detailed

  # Test main app
  curl https://${RAILWAY_URL}/
  ```
  - [ ] Health endpoint returns: `healthy`
  - [ ] Detailed health returns: JSON with status
  - [ ] Main app returns: HTML
  - [ ] Application URL: `________________________________`

- [ ] **Test zero-downtime deployment**
  ```bash
  # Make another change
  echo "# Testing zero-downtime" >> README.md
  git add README.md
  git commit -m "test: zero-downtime deployment"
  git push origin main

  # While deploying, continuously poll:
  while true; do
    curl -s https://${RAILWAY_URL}/api/health
    sleep 1
  done
  ```
  - [ ] New deployment triggered
  - [ ] Application remained accessible during deployment
  - [ ] No 503 errors
  - [ ] Zero-downtime confirmed

---

### Day 2: Tuesday (2 hours) - Monitoring & Documentation

#### Task 4.2: Setup Monitoring

- [ ] **Create monitoring script**
  ```bash
  mkdir -p scripts
  # Copy monitor-builds.sh from PRODUCTION_IMPLEMENTATION_PLAN.md
  ```
  - [ ] File created: `scripts/monitor-builds.sh`
  - [ ] Permissions set: `chmod +x scripts/monitor-builds.sh`

- [ ] **Install GitHub CLI (if needed)**
  ```bash
  # Ubuntu/Debian:
  sudo apt-get install -y gh

  # Or download from: https://cli.github.com/

  # Login
  gh auth login

  # Verify
  gh auth status
  ```
  - [ ] GitHub CLI installed
  - [ ] Authenticated successfully

- [ ] **Test monitoring script**
  ```bash
  ./scripts/monitor-builds.sh
  ```
  - [ ] Script runs successfully
  - [ ] Shows last 10 builds
  - [ ] Shows average build time
  - [ ] Output saved for reference

- [ ] **Setup build time alerting (optional)**
  ```bash
  # Add to .github/workflows/ci-cd.yml after build step:

  - name: Alert on slow build
    if: steps.build-time.outputs.build-time > 420
    uses: 8398a7/action-slack@v3
    with:
      status: custom
      custom_payload: |
        {
          text: "Build time exceeded budget: ${{ steps.build-time.outputs.build-time }}s > 420s"
        }
    env:
      SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
  ```
  - [ ] Slack webhook setup (optional)
  - [ ] Alert tested (optional)

#### Task 4.3: Create Documentation

- [ ] **Create PRODUCTION_DEPLOYMENT.md**
  ```bash
  # Copy template from PRODUCTION_IMPLEMENTATION_PLAN.md
  # Section: "Week 4: Day 2: Task 4.4"
  ```
  - [ ] File created
  - [ ] Architecture section filled
  - [ ] Build optimization techniques documented
  - [ ] Performance metrics table filled with actual numbers
  - [ ] Development workflow documented
  - [ ] Troubleshooting section reviewed
  - [ ] Maintenance schedule documented

- [ ] **Fill in actual performance metrics**
  - [ ] CI Build (cold): `_______ minutes`
  - [ ] CI Build (warm): `_______ minutes`
  - [ ] Railway Deploy: `_______ seconds`
  - [ ] Total (git push → live): `_______ minutes`
  - [ ] All targets met? `Yes / No`

- [ ] **Create runbook checklist**
  ```bash
  cat > RUNBOOK.md << 'EOF'
  # Production Runbook

  ## Daily Checks
  - None required (fully automated)

  ## Weekly Checks
  - [ ] Run ./scripts/monitor-builds.sh
  - [ ] Review build time trends
  - [ ] Check S3 bucket size: aws s3 ls --summarize --recursive s3://bucket

  ## Monthly Checks
  - [ ] Review dependency updates: cargo outdated
  - [ ] Check AWS costs (should be $0 on free tier)
  - [ ] Verify sccache hit rate >80%

  ## When Build Fails
  1. Check GitHub Actions logs
  2. Check sccache stats in logs
  3. Verify S3 access
  4. Verify secrets are set correctly
  5. Check for new dependency issues

  ## When Deployment Fails
  1. Check Railway logs: railway logs
  2. Test health endpoint
  3. Verify environment variables set
  4. Check GHCR image exists
  5. Verify Railway can pull from GHCR

  ## Rollback Procedure
  1. Railway dashboard → Deployments
  2. Select previous successful deployment
  3. Click "Redeploy"
  4. Or: git revert HEAD && git push
  EOF
  ```
  - [ ] Runbook created

- [ ] **Commit all documentation**
  ```bash
  git add PRODUCTION_DEPLOYMENT.md
  git add RUNBOOK.md
  git add scripts/monitor-builds.sh
  git add BASELINE_MEASUREMENTS.md
  git commit -m "week 4: add production documentation and monitoring"
  git push origin main
  ```
  - [ ] Committed successfully
  - [ ] Pushed to remote

---

## Final Validation & Testing

### End-to-End Test

- [ ] **Full deployment test**
  ```bash
  # Make meaningful change
  # (e.g., add a comment to a file)
  echo "// Production deployment test" >> crates/terminal/src/main.rs
  git add crates/terminal/src/main.rs
  git commit -m "test: end-to-end production deployment"
  git push origin main
  ```

- [ ] **Monitor complete flow**
  - [ ] GitHub Actions triggered
  - [ ] Build started: `_______` (timestamp)
  - [ ] Build completed: `_______` (timestamp)
  - [ ] Build time: `_______ minutes` (target: <5 min)
  - [ ] Image pushed to GHCR
  - [ ] Railway deployment triggered
  - [ ] Railway pull completed
  - [ ] Container started
  - [ ] Health check passed
  - [ ] Traffic switched
  - [ ] Deployment completed: `_______` (timestamp)
  - [ ] Total time: `_______ minutes` (target: <7 min)

- [ ] **Verify application works**
  - [ ] Visit application URL: `________________________________`
  - [ ] Main page loads
  - [ ] Health endpoint: `https://<app>.railway.app/api/health`
  - [ ] Detailed health: `https://<app>.railway.app/api/health/detailed`
  - [ ] No errors in browser console
  - [ ] No errors in Railway logs

### Performance Validation

- [ ] **Build performance**
  - [ ] CI cold build: `_______ min` (target: <12 min) ✓ / ✗
  - [ ] CI warm build: `_______ min` (target: <5 min) ✓ / ✗
  - [ ] sccache hit rate: `_______%` (target: >80%) ✓ / ✗

- [ ] **Deployment performance**
  - [ ] Railway deploy: `_______ seconds` (target: <90s) ✓ / ✗
  - [ ] Total time: `_______ minutes` (target: <7 min) ✓ / ✗

- [ ] **Cost validation**
  - [ ] AWS S3 costs: `$_______` (target: $0 on free tier)
  - [ ] GitHub Actions minutes used: `_______` (target: <2000/month)
  - [ ] Railway costs: `$_______` (target: $0-5/month)

### Quality Assurance

- [ ] **Code quality**
  - [ ] All files have proper comments
  - [ ] No hardcoded secrets
  - [ ] .gitignore updated
  - [ ] No unnecessary files committed

- [ ] **Documentation quality**
  - [ ] README.md updated with new deployment process
  - [ ] PRODUCTION_DEPLOYMENT.md complete
  - [ ] RUNBOOK.md complete
  - [ ] BASELINE_MEASUREMENTS.md has all data
  - [ ] Comments explain why, not just what

- [ ] **Security checklist**
  - [ ] AWS keys stored in GitHub secrets (not in code)
  - [ ] Railway token stored in GitHub secrets
  - [ ] S3 bucket policy follows least privilege
  - [ ] Docker container runs as non-root user
  - [ ] No sensitive data in logs
  - [ ] .env files in .gitignore

### Troubleshooting Tests

- [ ] **Test build failure recovery**
  ```bash
  # Introduce intentional syntax error
  echo "invalid rust syntax }}" >> crates/terminal/src/main.rs
  git add -A
  git commit -m "test: build failure"
  git push origin main
  ```
  - [ ] Build fails as expected
  - [ ] Error clearly shown in GitHub Actions
  - [ ] Railway not deployed (good!)
  - [ ] Revert: `git revert HEAD && git push`
  - [ ] Build succeeds after revert

- [ ] **Test deployment failure recovery**
  ```bash
  # Remove required env var temporarily
  railway variables --unset RPC_URL

  # Trigger deployment
  echo "# test deploy fail" >> README.md
  git add README.md && git commit -m "test: deploy fail" && git push
  ```
  - [ ] Deployment fails (missing env var)
  - [ ] Error logged in Railway
  - [ ] Previous version still running (rollback worked)
  - [ ] Fix: `railway variables --set RPC_URL=<value>`
  - [ ] Retry deployment succeeds

- [ ] **Test sccache failure handling**
  ```bash
  # Temporarily invalidate AWS keys in GitHub secrets
  # (Set to dummy value)
  ```
  - [ ] Build still completes (falls back to no cache)
  - [ ] Build time slower (expected)
  - [ ] No fatal errors
  - [ ] Restore correct keys
  - [ ] Next build uses cache again

---

## Success Criteria - Final Check

### Must Have (Blocking)

- [ ] ✅ CI builds complete in <12 minutes (cold)
- [ ] ✅ CI builds complete in <5 minutes (warm)
- [ ] ✅ Railway deployment <90 seconds
- [ ] ✅ Total git push → live <7 minutes
- [ ] ✅ Application accessible and healthy
- [ ] ✅ Zero-downtime deployments working
- [ ] ✅ Health checks passing
- [ ] ✅ Monitoring in place

### Should Have (Important)

- [ ] ✅ sccache hit rate >80% on warm builds
- [ ] ✅ Documentation complete
- [ ] ✅ Runbook created
- [ ] ✅ Cost <$5/month
- [ ] ✅ No security issues
- [ ] ✅ Rollback procedure tested

### Nice to Have (Optional)

- [ ] ⭐ Build time alerts configured
- [ ] ⭐ Slack/Discord notifications
- [ ] ⭐ Performance dashboard
- [ ] ⭐ Automated weekly reports

---

## Metrics Summary

### Final Performance Numbers

**Before Optimization:**
- Railway build: `TIMEOUT (13+ min)` ❌
- Deployment: `FAIL` ❌
- Total: `FAILED` ❌

**After Optimization:**
- CI build (cold): `_______ min` ✅
- CI build (warm): `_______ min` ✅
- Railway deploy: `_______ sec` ✅
- Total: `_______ min` ✅

**Improvement:**
- Build speedup: `_______%`
- Deployment success: `FAIL → SUCCESS`
- Total time: `TIMEOUT → _______ min`

### Cost Summary

- AWS S3: `$_______/month` (target: $0)
- GitHub Actions: `$_______/month` (target: $0)
- Railway: `$_______/month` (target: $0-5)
- **Total: `$_______/month`**

### Quality Metrics

- sccache hit rate: `_______%`
- Build success rate: `_______%`
- Deployment success rate: `_______%`
- Zero-downtime achieved: `Yes / No`

---

## Post-Implementation

### Week 5+: Maintenance

- [ ] **Setup weekly monitoring routine**
  - [ ] Add calendar reminder: Every Monday
  - [ ] Run: `./scripts/monitor-builds.sh`
  - [ ] Check for trends (increasing build times)

- [ ] **Setup monthly reviews**
  - [ ] Add calendar reminder: First of month
  - [ ] Check AWS costs
  - [ ] Review dependency updates
  - [ ] Check for new Rust tooling improvements

- [ ] **Document lessons learned**
  ```bash
  cat > LESSONS_LEARNED.md << 'EOF'
  # Lessons Learned

  ## What Went Well
  - ...

  ## What Could Be Improved
  - ...

  ## Surprises
  - ...

  ## Recommendations for Next Time
  - ...
  EOF
  ```

### Future Enhancements

- [ ] **Consider Cranelift (when stable)**
  - [ ] Monitor Rust 2025H2 roadmap
  - [ ] Test Cranelift for dev builds
  - [ ] Measure impact (20-40% faster expected)

- [ ] **Workspace optimization (if needed)**
  - [ ] Monitor build times over time
  - [ ] If increasing, consider splitting large crates
  - [ ] Use `cargo build --timings` regularly

- [ ] **Multi-arch support (optional)**
  - [ ] Add ARM64 builds for better Railway performance
  - [ ] Use Docker buildx multi-platform

---

## Completion Certificate

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║        Railway Deployment Optimization - COMPLETE ✅          ║
║                                                               ║
║  Project: usdfc-terminal                                      ║
║  Completion Date: _______________                             ║
║                                                               ║
║  Metrics Achieved:                                            ║
║  • CI Build Time: _______ min (target: <5 min)               ║
║  • Railway Deploy: _______ sec (target: <90s)                ║
║  • Total Time: _______ min (target: <7 min)                  ║
║  • Success Rate: 100%                                         ║
║                                                               ║
║  Status: PRODUCTION-READY 🚀                                  ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

**Implemented by:** _______________
**Date:** _______________
**Signature:** _______________

---

## Appendix: Quick Reference

### Common Commands

```bash
# Local development
cargo leptos watch --hot-reload

# Local build
cargo leptos build --profile ci -p usdfc-analytics-terminal

# Check sccache stats
sccache --show-stats

# Monitor CI builds
./scripts/monitor-builds.sh

# Railway logs
railway logs --follow

# Test health endpoint
curl https://<app>.railway.app/api/health

# Trigger deployment
git push origin main

# Rollback (via git)
git revert HEAD && git push

# Check build time in CI
gh run list --limit 5
gh run view <id> --log | grep "Build completed"
```

### Important URLs

- GitHub repo: `________________________________`
- GitHub Actions: `https://github.com/<owner>/<repo>/actions`
- GHCR packages: `https://github.com/<owner>/<repo>/pkgs/container/<image>`
- Railway dashboard: `https://railway.app/project/<project-id>`
- Application URL: `________________________________`
- Health check: `https://<app>.railway.app/api/health`

### Key Files

- `.cargo/config.toml` - mold linker + sccache config
- `Cargo.toml` - CI profile + feature pruning
- `Dockerfile.production` - Production multi-stage build
- `.github/workflows/ci-cd.yml` - CI/CD pipeline
- `railway.toml` - Railway deployment config
- `PRODUCTION_DEPLOYMENT.md` - Complete documentation
- `RUNBOOK.md` - Operations manual
- `scripts/monitor-builds.sh` - Monitoring script

---

**End of TODO Plan**

Total tasks: 200+
Estimated time: 30 hours
Recommended timeline: 4 weeks
Success rate: High (if followed completely)

🚀 **Let's optimize those builds!**
