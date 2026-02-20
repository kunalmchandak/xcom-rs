#!/usr/bin/env bash
set -euo pipefail

CRATE_NAME="xcom-rs"
FORMULA_PATH="Formula/xcom_rs.rb"

usage() {
	cat <<'EOF'
Usage:
  scripts/update_homebrew_formula.sh [version]

Updates the Homebrew formula in-place to point at the given crates.io version
and sets the correct sha256 for the crate tarball.

If version is omitted, it is read from Cargo.toml.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
	if [[ ! -f Cargo.toml ]]; then
		echo "Error: Cargo.toml not found; pass a version explicitly." >&2
		exit 1
	fi

	VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
fi

if [[ ! -f "$FORMULA_PATH" ]]; then
	echo "Error: formula not found: $FORMULA_PATH" >&2
	exit 1
fi

DOWNLOAD_URL="https://crates.io/api/v1/crates/${CRATE_NAME}/${VERSION}/download"

tmpfile=$(mktemp)
cleanup() {
	rm -f "$tmpfile"
}
trap cleanup EXIT

# crates.io download can lag slightly after publish; retry a bit.
sha256=""
for _ in {1..10}; do
	if curl -fsSL "$DOWNLOAD_URL" -o "$tmpfile"; then
		sha256=$(shasum -a 256 "$tmpfile" | awk '{print $1}')
		break
	fi
	sleep 3
done

if [[ -z "$sha256" ]]; then
	echo "Error: failed to download tarball and compute sha256: $DOWNLOAD_URL" >&2
	exit 1
fi

perl -pi -e "s|^  url \".*\"\$|  url \"$DOWNLOAD_URL\"|" "$FORMULA_PATH"
perl -pi -e "s|^  sha256 \"[0-9a-f]{64}\"\$|  sha256 \"$sha256\"|" "$FORMULA_PATH"

if ! grep -q "^  url \"$DOWNLOAD_URL\"$" "$FORMULA_PATH"; then
	echo "Error: failed to update url line in $FORMULA_PATH" >&2
	exit 1
fi

if ! grep -q "^  sha256 \"$sha256\"$" "$FORMULA_PATH"; then
	echo "Error: failed to update sha256 line in $FORMULA_PATH" >&2
	exit 1
fi

echo "Updated $FORMULA_PATH"
echo "- url: $DOWNLOAD_URL"
echo "- sha256: $sha256"
