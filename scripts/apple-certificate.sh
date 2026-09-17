#!/usr/bin/env bash
# Creates the Apple "Developer ID Application" certificate material without a
# Mac. Apple's instructions use Keychain Access for both steps; openssl does the
# same from Git Bash on Windows (or any shell).
#
#   scripts/apple-certificate.sh request [email]
#       Writes a private key and a certificate signing request. Upload the
#       request at https://developer.apple.com/account/resources/certificates/add
#       (Developer ID Application, G2 Sub-CA); only the Account Holder can.
#
#   scripts/apple-certificate.sh package path/to/developerID_application.cer
#       Pairs the certificate Apple returns with the key and writes the two
#       values the release workflow reads as repository secrets:
#         developer-id.p12.base64  -> APPLE_CERTIFICATE
#         p12-password.txt         -> APPLE_CERTIFICATE_PASSWORD
#
# Everything lands in ~/.apple-signing (override with APPLE_SIGNING_DIR), outside
# the repository and outside Dropbox. The private key leaves this machine only
# inside the password-protected .p12. The certificate is valid for five years;
# keep the folder, a lost key means revoking and issuing a new certificate.
set -euo pipefail

common_name="SIA Physical Software"
country="LV"
# Issuer of current Developer ID certificates. Bundled into the .p12 so the
# runner's keychain can build the chain without relying on what Xcode installed.
intermediate_url="https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer"

dir="${APPLE_SIGNING_DIR:-$HOME/.apple-signing}"
mkdir -p "$dir"
chmod 700 "$dir" 2>/dev/null || true
# Git Bash: the native openssl wants C:/... paths, also inside `file:` arguments.
if command -v cygpath >/dev/null 2>&1; then dir="$(cygpath -m "$dir")"; fi

key="$dir/developer-id.key"
csr="$dir/developer-id.certSigningRequest"
pem="$dir/developer-id.pem"
chain="$dir/developer-id-chain.pem"
p12="$dir/developer-id.p12"
password_file="$dir/p12-password.txt"

fail() { echo "error: $*" >&2; exit 1; }

case "${1:-}" in
  request)
    [ -e "$key" ] && fail "$key exists already. A new key would orphan the certificate issued for it; move the folder away first if that is intended."
    email="${2:-support@physicaladdons.com}"
    # A config file instead of -subj: Git Bash rewrites "/CN=..." into a path.
    cnf="$dir/request.cnf"
    cat > "$cnf" <<EOF
[req]
prompt = no
distinguished_name = dn
[dn]
emailAddress = $email
CN = $common_name
C = $country
EOF
    # Apple accepts RSA 2048 only for Developer ID requests.
    openssl req -new -newkey rsa:2048 -nodes -keyout "$key" -out "$csr" -config "$cnf"
    rm -f "$cnf"
    chmod 600 "$key" 2>/dev/null || true
    echo "Private key: $key"
    echo "Upload this: $csr"
    ;;

  package)
    cer="${2:-}"
    [ -n "$cer" ] && [ -f "$cer" ] || fail "usage: $0 package path/to/developerID_application.cer"
    [ -f "$key" ] || fail "$key not found; the certificate must be issued for a request made by '$0 request' on this machine."
    if command -v cygpath >/dev/null 2>&1; then cer="$(cygpath -m "$cer")"; fi

    # Apple serves DER; accept PEM too.
    openssl x509 -inform DER -in "$cer" -out "$pem" 2>/dev/null || openssl x509 -in "$cer" -out "$pem"

    cert_pub="$(openssl x509 -in "$pem" -noout -pubkey | openssl sha256)"
    key_pub="$(openssl pkey -in "$key" -pubout | openssl sha256)"
    [ "$cert_pub" = "$key_pub" ] || fail "this certificate was not issued for $key"

    subject="$(openssl x509 -in "$pem" -noout -subject -nameopt RFC2253)"
    echo "Certificate: $subject"
    openssl x509 -in "$pem" -noout -enddate
    case "$subject" in
      *"Developer ID Application"*) ;;
      *) fail "not a Developer ID Application certificate; Gatekeeper accepts no other type for apps distributed outside the App Store." ;;
    esac

    chain_args=()
    if curl -fsSL "$intermediate_url" -o "$dir/intermediate.cer" \
      && openssl x509 -inform DER -in "$dir/intermediate.cer" -out "$chain" 2>/dev/null \
      && [ "$(openssl x509 -in "$pem" -noout -issuer_hash)" = "$(openssl x509 -in "$chain" -noout -subject_hash)" ]; then
      chain_args=(-certfile "$chain")
      echo "Issuer bundled: $(openssl x509 -in "$chain" -noout -subject -nameopt RFC2253)"
    else
      echo "warning: could not fetch or match the issuing certificate; packaging the leaf alone." >&2
    fi
    rm -f "$dir/intermediate.cer"

    # The password goes from openssl to the file and from the file to openssl;
    # it is never printed. No line ending in the file (openssl on Windows would
    # write CR LF), so its whole content is the secret to paste.
    openssl rand -hex 24 | tr -d '\r\n' > "$password_file"
    chmod 600 "$password_file" 2>/dev/null || true
    # SHA1/3DES on purpose: the `security import` on macOS runners rejects the
    # AES/SHA-256 containers OpenSSL 3 writes by default ("MAC verification
    # failed"). The protection that matters is the GitHub secret store.
    openssl pkcs12 -export -inkey "$key" -in "$pem" ${chain_args[@]+"${chain_args[@]}"} \
      -name "Developer ID Application" \
      -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 \
      -passout "file:$password_file" -out "$p12"
    openssl base64 -A -in "$p12" -out "$p12.base64"

    echo
    echo "Repository secrets (Settings > Secrets and variables > Actions):"
    echo "  APPLE_CERTIFICATE           contents of $p12.base64"
    echo "  APPLE_CERTIFICATE_PASSWORD  contents of $password_file"
    ;;

  *)
    sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
    ;;
esac
