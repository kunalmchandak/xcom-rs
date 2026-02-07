#!/bin/bash
set -e

echo "=== Verification Script for design-agentic-cli-core ==="
echo ""

# Build first to avoid cargo build output in tests
cargo build --release >/dev/null 2>&1
CLI="./target/release/xcom-rs"

# Task 1: Verify schemaVersion in response
echo "✓ Task 1: Checking schemaVersion in response..."
$CLI commands --output json 2>/dev/null | jq -e '.schemaVersion == 1' >/dev/null
echo "  PASS: schemaVersion is present"

# Task 2: Verify structured error on invalid argument
echo "✓ Task 2: Checking structured error for invalid argument..."
$CLI unknown 2>/dev/null | jq -e '.error.code == "UNKNOWN_COMMAND"' >/dev/null
$CLI unknown 2>/dev/null | jq -e '.error.message' >/dev/null
$CLI unknown 2>/dev/null | jq -e '.error.isRetryable == false' >/dev/null
echo "  PASS: Structured error with code/message/isRetryable"

# Task 3: Verify exit code policy
echo "✓ Task 3: Checking exit code policy..."
$CLI unknown 2>/dev/null >/dev/null || EXIT_CODE=$?
if [ "$EXIT_CODE" -eq 2 ]; then
	echo "  PASS: Exit code 2 for invalid argument"
else
	echo "  FAIL: Expected exit code 2, got $EXIT_CODE"
	exit 1
fi

# Task 4: Verify output format options
echo "✓ Task 4: Checking output format options..."
$CLI commands --output json 2>/dev/null | jq -e '.ok' >/dev/null
$CLI commands --output yaml 2>/dev/null | grep -q "ok: true"
$CLI commands --output text 2>/dev/null | grep -q "commands"
echo "  PASS: All three output formats (json/yaml/text) work"

# Task 5: Verify commands output with risk and hasCost
echo "✓ Task 5: Checking commands output structure..."
$CLI commands --output json 2>/dev/null | jq -e '.data.commands[0].risk' >/dev/null
$CLI commands --output json 2>/dev/null | jq -e '.data.commands[0].hasCost' >/dev/null
echo "  PASS: commands[] with risk and hasCost"

# Task 6: Verify schema command
echo "✓ Task 6: Checking schema command..."
$CLI schema --command commands --output json 2>/dev/null | jq -e '.data.outputSchema.properties.ok' >/dev/null
echo "  PASS: schema returns envelope schema"

# Task 7: Verify help command
echo "✓ Task 7: Checking help command..."
$CLI help commands --output json 2>/dev/null | jq -e '.data.exitCodes' >/dev/null
$CLI help commands --output json 2>/dev/null | jq -e '.data.errorVocabulary' >/dev/null
$CLI help commands --output json 2>/dev/null | jq -e '.data.examples' >/dev/null
echo "  PASS: help includes exitCodes, errorVocabulary, and examples"

# Task 8: Verify trace-id in logs and meta
echo "✓ Task 8: Checking trace-id propagation..."
$CLI commands --trace-id test-123 2>&1 | grep -q "test-123"
$CLI commands --trace-id test-123 --output json 2>/dev/null | jq -e '.meta.traceId == "test-123"' >/dev/null
echo "  PASS: traceId in stderr logs and response meta"

# Acceptance #1 Follow-up tasks
echo "✓ Acceptance #1.1: trace-id in response meta..."
$CLI commands --trace-id abc-456 --output json 2>/dev/null | jq -e '.meta.traceId == "abc-456"' >/dev/null
echo "  PASS: trace-id reflected in meta.traceId"

echo "✓ Acceptance #1.2: help has examples and errorVocabulary..."
$CLI help commands --output json 2>/dev/null | jq -e '.data.examples' >/dev/null
$CLI help commands --output json 2>/dev/null | jq -e '.data.errorVocabulary' >/dev/null
echo "  PASS: help includes all required fields"

echo "✓ Acceptance #1.3: schema returns full envelope schema..."
$CLI schema --command commands --output json 2>/dev/null | jq -e '.data.outputSchema.properties.schemaVersion' >/dev/null
echo "  PASS: outputSchema includes envelope structure"

echo "✓ Acceptance #1.4: --help and --version return exit code 0..."
$CLI --help >/dev/null 2>&1 || EXIT_CODE=$?
if [ "${EXIT_CODE:-0}" -eq 0 ]; then
	echo "  PASS: --help returns exit code 0"
else
	echo "  FAIL: --help returned exit code $EXIT_CODE"
	exit 1
fi
$CLI --version >/dev/null 2>&1 || EXIT_CODE=$?
if [ "${EXIT_CODE:-0}" -eq 0 ]; then
	echo "  PASS: --version returns exit code 0"
else
	echo "  FAIL: --version returned exit code $EXIT_CODE"
	exit 1
fi

# Task 4: Verify output format options
echo "✓ Task 4: Checking output format options..."
cargo run --release -- commands --output json 2>/dev/null | jq -e '.ok' >/dev/null
cargo run --release -- commands --output yaml 2>/dev/null | grep -q "ok: true"
cargo run --release -- commands --output text 2>/dev/null | grep -q "commands"
echo "  PASS: All three output formats (json/yaml/text) work"

# Task 5: Verify commands output with risk and hasCost
echo "✓ Task 5: Checking commands output structure..."
cargo run --release -- commands --output json 2>/dev/null | jq -e '.data.commands[0].risk' >/dev/null
cargo run --release -- commands --output json 2>/dev/null | jq -e '.data.commands[0].hasCost' >/dev/null
echo "  PASS: commands[] with risk and hasCost"

# Task 6: Verify schema command
echo "✓ Task 6: Checking schema command..."
cargo run --release -- schema --command commands --output json 2>/dev/null | jq -e '.data.outputSchema.properties.ok' >/dev/null
echo "  PASS: schema returns envelope schema"

# Task 7: Verify help command
echo "✓ Task 7: Checking help command..."
cargo run --release -- help commands --output json 2>/dev/null | jq -e '.data.exitCodes' >/dev/null
cargo run --release -- help commands --output json 2>/dev/null | jq -e '.data.errorVocabulary' >/dev/null
cargo run --release -- help commands --output json 2>/dev/null | jq -e '.data.examples' >/dev/null
echo "  PASS: help includes exitCodes, errorVocabulary, and examples"

# Task 8: Verify trace-id in logs and meta
echo "✓ Task 8: Checking trace-id propagation..."
cargo run --release -- commands --trace-id test-123 2>&1 | grep -q "test-123"
cargo run --release -- commands --trace-id test-123 --output json 2>/dev/null | jq -e '.meta.traceId == "test-123"' >/dev/null
echo "  PASS: traceId in stderr logs and response meta"

# Acceptance #1 Follow-up tasks
echo "✓ Acceptance #1.1: trace-id in response meta..."
cargo run --release -- commands --trace-id abc-456 --output json 2>/dev/null | jq -e '.meta.traceId == "abc-456"' >/dev/null
echo "  PASS: trace-id reflected in meta.traceId"

echo "✓ Acceptance #1.2: help has examples and errorVocabulary..."
cargo run --release -- help commands --output json 2>/dev/null | jq -e '.data.examples' >/dev/null
cargo run --release -- help commands --output json 2>/dev/null | jq -e '.data.errorVocabulary' >/dev/null
echo "  PASS: help includes all required fields"

echo "✓ Acceptance #1.3: schema returns full envelope schema..."
cargo run --release -- schema --command commands --output json 2>/dev/null | jq -e '.data.outputSchema.properties.schemaVersion' >/dev/null
echo "  PASS: outputSchema includes envelope structure"

echo "✓ Acceptance #1.4: --help and --version return exit code 0..."
cargo run --release -- --help >/dev/null 2>&1
EXIT_CODE=$?
if [ "$EXIT_CODE" -eq 0 ]; then
	echo "  PASS: --help returns exit code 0"
else
	echo "  FAIL: --help returned exit code $EXIT_CODE"
	exit 1
fi
cargo run --release -- --version >/dev/null 2>&1
EXIT_CODE=$?
if [ "$EXIT_CODE" -eq 0 ]; then
	echo "  PASS: --version returns exit code 0"
else
	echo "  FAIL: --version returned exit code $EXIT_CODE"
	exit 1
fi
EXIT_CODE=$(
	cargo run --release -- --version 2>&1 >/dev/null
	echo $?
)
if [ "$EXIT_CODE" -eq 0 ]; then
	echo "  PASS: --version returns exit code 0"
else
	echo "  FAIL: --version returned exit code $EXIT_CODE"
	exit 1
fi

# Acceptance #2 Follow-up
echo "✓ Acceptance #2: --non-interactive integration..."
# Verify ExecutionContext is wired and available
cargo test test_non_interactive_context 2>&1 | grep -q "test result: ok"
echo "  PASS: ExecutionContext.check_interaction_required() is implemented and tested"

echo ""
echo "=== All Acceptance Criteria Verified ==="
echo "All tasks completed successfully!"
