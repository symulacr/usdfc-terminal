#!/bin/bash
# Test deployed API cursor pagination

set -e

API_URL="https://usdfc-terminal-backend-production.up.railway.app"

echo "=== Testing Deployed API Cursor Pagination ==="
echo ""

# Test 1: Check version endpoint
echo "Test 1: Version endpoint"
curl -s "${API_URL}/api/v1/version" | jq '.'
echo ""

# Test 2: Get first page of holders
echo "Test 2: First page of holders (no cursor)"
RESPONSE=$(curl -s "${API_URL}/api/v1/holders")
echo "$RESPONSE" | jq '{
  holder_count: (.data.holders | length),
  total_holders: .data.total_holders,
  has_next_cursor: (.data.next_cursor != null),
  first_holder: .data.holders[0].address
}'
NEXT_CURSOR=$(echo "$RESPONSE" | jq -r '.data.next_cursor // empty')
echo ""

# Test 3: Get second page using cursor
if [ -n "$NEXT_CURSOR" ]; then
  echo "Test 3: Second page with cursor"
  ENCODED_CURSOR=$(echo "$NEXT_CURSOR" | jq -sRr @uri)
  RESPONSE2=$(curl -s "${API_URL}/api/v1/holders?cursor=${ENCODED_CURSOR}")
  echo "$RESPONSE2" | jq '{
    holder_count: (.data.holders | length),
    has_next_cursor: (.data.next_cursor != null),
    first_holder: .data.holders[0].address
  }'
  echo ""

  # Verify pages are different
  FIRST_PAGE_ADDR=$(echo "$RESPONSE" | jq -r '.data.holders[0].address')
  SECOND_PAGE_ADDR=$(echo "$RESPONSE2" | jq -r '.data.holders[0].address')

  if [ "$FIRST_PAGE_ADDR" != "$SECOND_PAGE_ADDR" ]; then
    echo "✅ SUCCESS: Pagination working - pages have different holders"
  else
    echo "❌ FAILED: Pagination broken - same holders on both pages"
  fi
else
  echo "⚠ WARNING: No next_cursor returned, can't test second page"
fi
