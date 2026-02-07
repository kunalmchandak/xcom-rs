#!/bin/bash
# Demo script for auth and billing features

set -e

echo "=== Auth & Billing Demo ==="
echo ""

echo "1. Check authentication status (unauthenticated):"
cargo run -q -- auth status --output json | jq .
echo ""

echo "2. Estimate cost for a tweet:"
cargo run -q -- billing estimate tweets.create --text "hello world" --output json | jq .
echo ""

echo "3. Dry-run mode (zero cost):"
cargo run -q -- billing estimate tweets.create --text "hello world" --dry-run --output json | jq .
echo ""

echo "4. Cost guard - exceed max-cost-credits (should fail):"
cargo run -q -- billing estimate tweets.create --text "hello world" --max-cost-credits 1 --output json | jq . || true
echo ""

echo "5. Non-interactive mode (should fail with next steps):"
cargo run -q -- demo-interactive --non-interactive --output json | jq . || true
echo ""

echo "=== Demo Complete ==="
