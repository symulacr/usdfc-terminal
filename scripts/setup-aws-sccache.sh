#!/bin/bash
# =============================================================================
# AWS S3 Setup Script for sccache - Week 2
# =============================================================================
#
# This script automates the setup of AWS S3 bucket and IAM user for sccache
# Used in GitHub Actions CI/CD pipeline for Rust compilation caching
#
# Prerequisites:
# - AWS CLI installed (aws --version)
# - AWS credentials configured (aws configure)
# - Permissions to create S3 buckets and IAM users
#
# Usage:
#   ./scripts/setup-aws-sccache.sh
#
# What it does:
# 1. Creates S3 bucket with unique name
# 2. Configures lifecycle policy (auto-delete after 30 days)
# 3. Creates IAM user for GitHub Actions
# 4. Creates and attaches IAM policy for S3 access
# 5. Generates access keys
# 6. Outputs GitHub secrets to add
#
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BUCKET_NAME="usdfc-terminal-sccache-$(date +%s)"
REGION="${AWS_REGION:-us-east-1}"
IAM_USER="github-actions-sccache"
POLICY_NAME="SccacheBucketAccess"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI not found. Please install: https://aws.amazon.com/cli/"
        exit 1
    fi

    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS credentials not configured. Run 'aws configure' first."
        exit 1
    fi

    log_success "Prerequisites OK"
}

# Create S3 bucket
create_s3_bucket() {
    log_info "Creating S3 bucket: ${BUCKET_NAME}..."

    if [ "${REGION}" = "us-east-1" ]; then
        aws s3 mb "s3://${BUCKET_NAME}" --region "${REGION}"
    else
        aws s3api create-bucket \
            --bucket "${BUCKET_NAME}" \
            --region "${REGION}" \
            --create-bucket-configuration LocationConstraint="${REGION}"
    fi

    log_success "S3 bucket created: ${BUCKET_NAME}"
}

# Enable versioning (optional)
enable_versioning() {
    log_info "Enabling versioning on bucket..."

    aws s3api put-bucket-versioning \
        --bucket "${BUCKET_NAME}" \
        --versioning-configuration Status=Enabled

    log_success "Versioning enabled"
}

# Set lifecycle policy
set_lifecycle_policy() {
    log_info "Setting lifecycle policy (delete after 30 days)..."

    cat > /tmp/lifecycle-${BUCKET_NAME}.json << 'EOF'
{
  "Rules": [
    {
      "Id": "delete-old-cache",
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
        --bucket "${BUCKET_NAME}" \
        --lifecycle-configuration "file:///tmp/lifecycle-${BUCKET_NAME}.json"

    rm -f "/tmp/lifecycle-${BUCKET_NAME}.json"
    log_success "Lifecycle policy set"
}

# Create IAM user
create_iam_user() {
    log_info "Creating IAM user: ${IAM_USER}..."

    if aws iam get-user --user-name "${IAM_USER}" &> /dev/null; then
        log_warning "IAM user ${IAM_USER} already exists. Skipping creation."
    else
        aws iam create-user --user-name "${IAM_USER}"
        log_success "IAM user created: ${IAM_USER}"
    fi
}

# Create IAM policy
create_iam_policy() {
    log_info "Creating IAM policy: ${POLICY_NAME}..."

    ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

    cat > /tmp/policy-${BUCKET_NAME}.json << EOF
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

    POLICY_ARN=$(aws iam create-policy \
        --policy-name "${POLICY_NAME}" \
        --policy-document "file:///tmp/policy-${BUCKET_NAME}.json" \
        --query 'Policy.Arn' \
        --output text 2>/dev/null || \
        aws iam list-policies --query "Policies[?PolicyName=='${POLICY_NAME}'].Arn" --output text)

    rm -f "/tmp/policy-${BUCKET_NAME}.json"
    log_success "IAM policy: ${POLICY_ARN}"

    echo "${POLICY_ARN}"
}

# Attach policy to user
attach_policy() {
    local POLICY_ARN=$1
    log_info "Attaching policy to user..."

    aws iam attach-user-policy \
        --user-name "${IAM_USER}" \
        --policy-arn "${POLICY_ARN}"

    log_success "Policy attached to user"
}

# Create access keys
create_access_keys() {
    log_info "Creating access keys..."

    # Delete old keys if they exist
    OLD_KEYS=$(aws iam list-access-keys --user-name "${IAM_USER}" --query 'AccessKeyMetadata[].AccessKeyId' --output text 2>/dev/null || echo "")

    if [ -n "${OLD_KEYS}" ]; then
        log_warning "Deleting old access keys..."
        for KEY_ID in ${OLD_KEYS}; do
            aws iam delete-access-key --user-name "${IAM_USER}" --access-key-id "${KEY_ID}" || true
        done
    fi

    # Create new keys
    aws iam create-access-key --user-name "${IAM_USER}" > /tmp/aws-keys-${BUCKET_NAME}.json

    ACCESS_KEY_ID=$(jq -r '.AccessKey.AccessKeyId' /tmp/aws-keys-${BUCKET_NAME}.json)
    SECRET_ACCESS_KEY=$(jq -r '.AccessKey.SecretAccessKey' /tmp/aws-keys-${BUCKET_NAME}.json)

    rm -f "/tmp/aws-keys-${BUCKET_NAME}.json"
    log_success "Access keys created"

    echo "${ACCESS_KEY_ID}|${SECRET_ACCESS_KEY}"
}

# Test S3 access
test_s3_access() {
    log_info "Testing S3 access..."

    echo "test" > /tmp/test-${BUCKET_NAME}.txt
    aws s3 cp /tmp/test-${BUCKET_NAME}.txt "s3://${BUCKET_NAME}/test.txt"
    aws s3 rm "s3://${BUCKET_NAME}/test.txt"
    rm -f "/tmp/test-${BUCKET_NAME}.txt"

    log_success "S3 access verified"
}

# Output results
output_results() {
    local ACCESS_KEY_ID=$1
    local SECRET_ACCESS_KEY=$2

    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo -e "${GREEN}AWS S3 Setup Complete!${NC}"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "S3 Bucket Information:"
    echo "  Name:   ${BUCKET_NAME}"
    echo "  Region: ${REGION}"
    echo "  URL:    s3://${BUCKET_NAME}"
    echo ""
    echo "IAM User Information:"
    echo "  Username: ${IAM_USER}"
    echo "  Policy:   ${POLICY_NAME}"
    echo ""
    echo "Access Keys (SAVE THESE SECURELY):"
    echo "  AWS_ACCESS_KEY_ID:     ${ACCESS_KEY_ID}"
    echo "  AWS_SECRET_ACCESS_KEY: ${SECRET_ACCESS_KEY}"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo -e "${YELLOW}Next Steps:${NC}"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "1. Add GitHub Secrets:"
    echo "   Go to: https://github.com/YOUR_USERNAME/usdfc-terminal/settings/secrets/actions"
    echo ""
    echo "   Add these secrets:"
    echo "   - Name: AWS_ACCESS_KEY_ID"
    echo "     Value: ${ACCESS_KEY_ID}"
    echo ""
    echo "   - Name: AWS_SECRET_ACCESS_KEY"
    echo "     Value: ${SECRET_ACCESS_KEY}"
    echo ""
    echo "2. Update GitHub Actions workflow:"
    echo "   Edit .github/workflows/docker-build.yml"
    echo "   Update SCCACHE_BUCKET to: ${BUCKET_NAME}"
    echo "   Update SCCACHE_REGION to: ${REGION}"
    echo ""
    echo "3. Commit and push to trigger CI build:"
    echo "   git add .github/workflows/docker-build.yml"
    echo "   git commit -m \"chore: update sccache bucket name\""
    echo "   git push origin main"
    echo ""
    echo "4. Monitor build at:"
    echo "   https://github.com/YOUR_USERNAME/usdfc-terminal/actions"
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo -e "${GREEN}Setup script complete!${NC}"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
}

# Main execution
main() {
    echo "═══════════════════════════════════════════════════════════════════"
    echo "AWS S3 Setup for sccache - Week 2"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""

    check_prerequisites
    create_s3_bucket
    enable_versioning
    set_lifecycle_policy
    create_iam_user
    POLICY_ARN=$(create_iam_policy)
    attach_policy "${POLICY_ARN}"
    KEYS=$(create_access_keys)
    ACCESS_KEY_ID=$(echo "${KEYS}" | cut -d'|' -f1)
    SECRET_ACCESS_KEY=$(echo "${KEYS}" | cut -d'|' -f2)
    test_s3_access
    output_results "${ACCESS_KEY_ID}" "${SECRET_ACCESS_KEY}"
}

# Run main function
main "$@"
