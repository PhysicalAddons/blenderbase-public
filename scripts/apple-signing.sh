#!/usr/bin/env bash
# Prepares a macOS GitHub runner to sign and notarize the Tauri build, and
# proves both credentials in seconds instead of after the compile. Reads the
# repository secrets from the environment:
#
#   APPLE_CERTIFICATE           base64 of the Developer ID Application .p12
#   APPLE_CERTIFICATE_PASSWORD  its password (both from scripts/apple-certificate.sh)
#   APPLE_API_KEY               App Store Connect API key id, for notarization
#   APPLE_API_ISSUER            issuer id shown above the key list
#   APPLE_API_KEY_P8            contents of the downloaded AuthKey_<id>.p8
#
# and hands Tauri what it looks for through $GITHUB_ENV: APPLE_SIGNING_IDENTITY,
# plus APPLE_API_KEY / APPLE_API_ISSUER / APPLE_API_KEY_PATH. The certificate is
# imported here rather than passed to Tauri as APPLE_CERTIFICATE so that a bad
# certificate or password fails with `security`'s own message.
#
#   scripts/apple-signing.sh             release build: without the certificate
#                                        secret the build stays unsigned, without
#                                        the API key it is signed, not notarized
#   scripts/apple-signing.sh --notarize  Sign check: every secret is required and
#                                        a small signed binary goes through
#                                        notarization
#
# Written for the bash 3.2 that ships with macOS.
set -euo pipefail

check=0
if [ "${1:-}" = "--notarize" ]; then check=1; fi

tmp="${RUNNER_TEMP:-$(mktemp -d)}"

# Secrets pasted on Windows can carry CR LF or a trailing newline.
certificate="$(printf '%s' "${APPLE_CERTIFICATE:-}" | tr -d '[:space:]')"
password="$(printf '%s' "${APPLE_CERTIFICATE_PASSWORD:-}" | tr -d '\r\n')"
api_key="$(printf '%s' "${APPLE_API_KEY:-}" | tr -d '[:space:]')"
api_issuer="$(printf '%s' "${APPLE_API_ISSUER:-}" | tr -d '[:space:]')"
api_p8="$(printf '%s' "${APPLE_API_KEY_P8:-}" | tr -d '\r')"

if [ -z "$certificate" ]; then
  if [ "$check" = 1 ]; then
    echo "::error::APPLE_CERTIFICATE is not set; nothing to check."
    exit 1
  fi
  echo "::notice::APPLE_CERTIFICATE is not set; the macOS build stays unsigned."
  exit 0
fi

# --- certificate -> keychain -------------------------------------------------
keychain="$tmp/blenderbase-signing.keychain-db"
keychain_password="$(openssl rand -hex 24)"
p12="$tmp/developer-id.p12"

printf '%s' "$certificate" | base64 --decode > "$p12"
security delete-keychain "$keychain" 2>/dev/null || true
security create-keychain -p "$keychain_password" "$keychain"
# No options: the keychain never locks on a timer or on sleep during the build.
security set-keychain-settings "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$p12" -k "$keychain" -P "$password" -T /usr/bin/codesign -T /usr/bin/productbuild
rm -f "$p12"
# Lets codesign use the key without the "allow access" dialog nobody can click.
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$keychain_password" "$keychain" >/dev/null

# Tauri calls codesign with the identity name only, so the keychain has to be
# on the search list, ahead of the ones already there.
existing=()
while IFS= read -r line; do
  line="${line#"${line%%[![:space:]]*}"}"
  line="${line#\"}"
  line="${line%\"}"
  if [ -n "$line" ]; then existing+=("$line"); fi
done < <(security list-keychains -d user)
security list-keychains -d user -s "$keychain" ${existing[@]+"${existing[@]}"}

identity="$(security find-identity -v -p codesigning "$keychain" | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -n 1)"
if [ -z "$identity" ]; then
  echo "::error::APPLE_CERTIFICATE holds no valid Developer ID Application identity."
  # Without -v the list includes rejected identities and the reason.
  security find-identity -p codesigning "$keychain" || true
  exit 1
fi
echo "Signing identity: $identity"

# --- prove the identity signs ------------------------------------------------
# Same options Tauri uses: hardened runtime and a secure timestamp, both
# required by notarization.
probe="$tmp/sign-probe"
printf 'int main(void) { return 0; }\n' | clang -x c - -o "$probe"
codesign --force --sign "$identity" --options runtime --timestamp "$probe"
codesign --verify --strict --verbose=2 "$probe"
codesign --display --verbose=2 "$probe" 2>&1 | grep -E '^(Authority|TeamIdentifier|Timestamp)=' || true

# --- notarization key --------------------------------------------------------
key_path=""
if [ -n "$api_key" ] && [ -n "$api_issuer" ] && [ -n "$api_p8" ]; then
  key_dir="$tmp/private_keys"
  mkdir -p "$key_dir"
  chmod 700 "$key_dir"
  key_path="$key_dir/AuthKey_$api_key.p8"
  printf '%s\n' "$api_p8" > "$key_path"
  chmod 600 "$key_path"
  # Cheapest authenticated call: lists earlier submissions, fails on a wrong
  # key, key id or issuer.
  xcrun notarytool history --key "$key_path" --key-id "$api_key" --issuer "$api_issuer" >/dev/null
  echo "Notarization key $api_key accepted by Apple."
elif [ "$check" = 1 ]; then
  echo "::error::APPLE_API_KEY, APPLE_API_ISSUER and APPLE_API_KEY_P8 must all be set."
  exit 1
else
  echo "::warning::Notarization secrets are incomplete; the app will be signed but not notarized, and Gatekeeper will still block it."
fi

# --- full round trip (Sign check only) ---------------------------------------
if [ "$check" = 1 ]; then
  zip="$tmp/sign-probe.zip"
  ditto -c -k "$probe" "$zip"
  echo "Submitting the probe for notarization..."
  # notarytool exits 0 for a finished submission whatever the verdict, so the
  # status is read from its JSON.
  result="$(xcrun notarytool submit "$zip" --key "$key_path" --key-id "$api_key" --issuer "$api_issuer" \
    --wait --timeout 45m --output-format json)" || true
  echo "$result"
  if ! printf '%s' "$result" | grep -Eq '"status"[[:space:]]*:[[:space:]]*"Accepted"'; then
    id="$(printf '%s' "$result" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
    if [ -n "$id" ]; then
      xcrun notarytool log "$id" --key "$key_path" --key-id "$api_key" --issuer "$api_issuer" || true
    fi
    echo "::error::Notarization did not come back Accepted. A team's very first submission can sit in Apple's queue for hours; if the status above is 'In Progress', run the check again later before changing anything."
    exit 1
  fi
  echo "Notarization round trip: Accepted."
fi

# --- hand over to the Tauri build --------------------------------------------
if [ -n "${GITHUB_ENV:-}" ]; then
  {
    echo "APPLE_SIGNING_IDENTITY=$identity"
    if [ -n "$key_path" ]; then
      echo "APPLE_API_KEY=$api_key"
      echo "APPLE_API_ISSUER=$api_issuer"
      echo "APPLE_API_KEY_PATH=$key_path"
    fi
  } >> "$GITHUB_ENV"
fi
