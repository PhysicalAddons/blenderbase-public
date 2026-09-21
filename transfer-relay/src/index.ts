/**
 * Blenderbase transfer relay.
 *
 * Holds encrypted setup files for a few days so a transfer code typed on another computer
 * can fetch them. The relay never sees a code or a key: the app derives a 256-bit id and an
 * encryption key from the code on the sending computer, uploads ciphertext under the id, and
 * the receiving computer derives the same id from the same code. A stolen bucket holds
 * nothing readable, and an id cannot be guessed.
 *
 * Routes (all under /v1/transfers):
 *   PUT    /:id                                  one-shot upload (body up to ONE_SHOT_LIMIT)
 *   POST   /:id/multipart                        start a multipart upload  -> { uploadId }
 *   PUT    /:id/multipart/:uploadId/:part        upload one part           -> { etag }
 *   POST   /:id/multipart/:uploadId/complete     finish: { parts: [{ partNumber, etag }] }
 *   DELETE /:id/multipart/:uploadId              abort
 *   HEAD   /:id                                  size and dates in headers
 *   GET    /:id                                  the ciphertext
 *   DELETE /:id                                  remove (the receiver does this once it has the file)
 *
 * Objects expire after TTL_DAYS. The bucket's lifecycle rule deletes them; the relay also
 * refuses and removes an expired object it is asked for, in case the rule lags.
 */

export interface Env {
  TRANSFERS: R2Bucket;
  /** Cloudflare's rate-limiting binding; optional so local development works without it. */
  LIMITER?: { limit(options: { key: string }): Promise<{ success: boolean }> };
  /** Days a transfer stays available. Default 7. */
  TTL_DAYS?: string;
  /** Largest transfer accepted, in bytes. Default 2 GiB. */
  MAX_BYTES?: string;
}

const ID_PATTERN = /^[0-9a-f]{64}$/;
const UPLOAD_ID_PATTERN = /^[A-Za-z0-9_\-=.]{1,512}$/;
/** Cloudflare's per-request body limit is 100 MB on the free plan; one-shot uploads stay under it. */
const ONE_SHOT_LIMIT = 95 * 1024 * 1024;
const PART_LIMIT = 95 * 1024 * 1024;
const MAX_PARTS = 40;

const json = (status: number, body: unknown, headers: Record<string, string> = {}): Response =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", ...headers } });

const error = (status: number, message: string): Response => json(status, { error: message });

const ttlMs = (env: Env): number => Number(env.TTL_DAYS || "7") * 24 * 60 * 60 * 1000;
const maxBytes = (env: Env): number => Number(env.MAX_BYTES || String(2 * 1024 * 1024 * 1024));

const expiryOf = (object: R2Object, env: Env): Date => {
  const created = object.customMetadata?.created ? new Date(object.customMetadata.created) : object.uploaded;
  return new Date(created.getTime() + ttlMs(env));
};

const clientKey = (request: Request): string => request.headers.get("cf-connecting-ip") ?? "unknown";

async function rateLimited(request: Request, env: Env): Promise<boolean> {
  if (!env.LIMITER) {
    return false;
  }
  const { success } = await env.LIMITER.limit({ key: clientKey(request) });
  return !success;
}

function declaredLength(request: Request): number | null {
  const header = request.headers.get("content-length");
  if (header === null) {
    return null;
  }
  const length = Number(header);
  return Number.isFinite(length) && length >= 0 ? length : null;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const parts = url.pathname.split("/").filter((p) => p.length > 0);
    if (parts.length < 3 || parts[0] !== "v1" || parts[1] !== "transfers") {
      return error(404, "Not found");
    }
    const id = parts[2];
    if (!ID_PATTERN.test(id)) {
      return error(400, "Not a transfer id");
    }
    if (await rateLimited(request, env)) {
      return error(429, "Too many requests; try again in a minute");
    }
    const rest = parts.slice(3);

    // ---- one object: HEAD, GET, PUT, DELETE
    if (rest.length === 0) {
      switch (request.method) {
        case "HEAD":
        case "GET": {
          const object = await env.TRANSFERS.get(id);
          if (object === null) {
            return error(404, "No transfer with this code");
          }
          const expires = expiryOf(object, env);
          if (expires.getTime() < Date.now()) {
            await env.TRANSFERS.delete(id);
            return error(410, "This transfer has expired");
          }
          const headers = new Headers();
          object.writeHttpMetadata(headers);
          headers.set("content-length", String(object.size));
          headers.set("content-type", "application/octet-stream");
          headers.set("x-transfer-created", (object.customMetadata?.created ?? object.uploaded.toISOString()));
          headers.set("x-transfer-expires", expires.toISOString());
          headers.set("cache-control", "no-store");
          return new Response(request.method === "HEAD" ? null : object.body, { status: 200, headers });
        }
        case "PUT": {
          const length = declaredLength(request);
          if (length === null) {
            return error(411, "Content-Length is required");
          }
          if (length > ONE_SHOT_LIMIT) {
            return error(413, "Too large for one request; use a multipart upload");
          }
          if (length > maxBytes(env)) {
            return error(413, "Too large");
          }
          if ((await env.TRANSFERS.head(id)) !== null) {
            return error(409, "A transfer with this id already exists");
          }
          const created = new Date().toISOString();
          await env.TRANSFERS.put(id, request.body, {
            httpMetadata: { contentType: "application/octet-stream" },
            customMetadata: { created },
          });
          return json(201, { id, created, expires: new Date(Date.now() + ttlMs(env)).toISOString(), size: length });
        }
        case "DELETE": {
          await env.TRANSFERS.delete(id);
          return json(200, { id, deleted: true });
        }
        default:
          return error(405, "Method not allowed");
      }
    }

    // ---- multipart: /:id/multipart[/:uploadId[/:part | /complete]]
    if (rest[0] !== "multipart") {
      return error(404, "Not found");
    }
    if (rest.length === 1 && request.method === "POST") {
      if ((await env.TRANSFERS.head(id)) !== null) {
        return error(409, "A transfer with this id already exists");
      }
      const upload = await env.TRANSFERS.createMultipartUpload(id, {
        httpMetadata: { contentType: "application/octet-stream" },
        customMetadata: { created: new Date().toISOString() },
      });
      return json(201, { id, uploadId: upload.uploadId, partLimit: PART_LIMIT, maxParts: MAX_PARTS });
    }
    const uploadId = rest[1];
    if (!uploadId || !UPLOAD_ID_PATTERN.test(uploadId)) {
      return error(400, "Not an upload id");
    }
    const upload = env.TRANSFERS.resumeMultipartUpload(id, uploadId);
    if (rest.length === 2 && request.method === "DELETE") {
      try {
        await upload.abort();
      } catch {
        // Already gone: the outcome is the same.
      }
      return json(200, { id, aborted: true });
    }
    if (rest.length === 3 && rest[2] === "complete" && request.method === "POST") {
      let body: { parts?: { partNumber: number; etag: string }[] };
      try {
        body = (await request.json()) as typeof body;
      } catch {
        return error(400, "Expected a JSON body with the parts");
      }
      const listed = body.parts ?? [];
      if (listed.length === 0 || listed.length > MAX_PARTS) {
        return error(400, `Expected 1 to ${MAX_PARTS} parts`);
      }
      try {
        const object = await upload.complete(listed.map((p) => ({ partNumber: Number(p.partNumber), etag: String(p.etag) })));
        return json(201, { id, size: object.size, expires: expiryOf(object, env).toISOString() });
      } catch (e) {
        return error(400, `Could not complete the upload: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
    if (rest.length === 3 && request.method === "PUT") {
      const partNumber = Number(rest[2]);
      if (!Number.isInteger(partNumber) || partNumber < 1 || partNumber > MAX_PARTS) {
        return error(400, `Part numbers run from 1 to ${MAX_PARTS}`);
      }
      const length = declaredLength(request);
      if (length === null) {
        return error(411, "Content-Length is required");
      }
      if (length > PART_LIMIT) {
        return error(413, "Part too large");
      }
      if (partNumber * PART_LIMIT > maxBytes(env) + PART_LIMIT) {
        return error(413, "Too large");
      }
      try {
        const part = await upload.uploadPart(partNumber, request.body as ReadableStream);
        return json(200, { partNumber: part.partNumber, etag: part.etag });
      } catch (e) {
        return error(400, `Could not store the part: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
    return error(404, "Not found");
  },
} satisfies ExportedHandler<Env>;
