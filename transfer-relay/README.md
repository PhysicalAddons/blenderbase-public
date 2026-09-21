# Blenderbase transfer relay

A Cloudflare Worker with an R2 bucket. It holds encrypted setup files for seven days so a
transfer code typed on another computer can fetch them.

The relay never sees a code, a key, or a readable setup. The app derives a 256-bit id and an
encryption key from the code on the sending computer, uploads only ciphertext under the id,
and the receiving computer derives the same id from the same code. The routes are listed at
the top of `src/index.ts`.

## Run it locally

```
npm install
npm run dev
```

Serves `http://127.0.0.1:8787` with a simulated bucket. Point the app at it with the
`BLENDERBASE_TRANSFER_RELAY` environment variable.

## Deploy

One-time, in the Cloudflare dashboard: enable R2 for the account (Cloudflare asks for a
payment method; the free tier covers 10 GB and R2 has no egress charges).

```
npx wrangler login
npx wrangler r2 bucket create blenderbase-transfers
npx wrangler r2 bucket lifecycle add blenderbase-transfers expire-after-8-days --expire-days 8 --abort-multipart-days 1
npx wrangler deploy
```

`wrangler deploy` prints the Worker's URL (`https://blenderbase-transfer.<subdomain>.workers.dev`);
that is the relay address the app ships with (`TRANSFER_RELAY_DEFAULT` in
`src-tauri/src/core/setup/transfer.rs`). A freshly created workers.dev subdomain answers
with TLS errors for a minute or two while its certificate is issued. The lifecycle rule is the
backstop that removes objects the relay itself did not get to delete.
