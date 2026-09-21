//! Transfer codes: a setup file travels through the relay (see `transfer-relay/`) under a code
//! such as `brave-otter-4412`. The code never leaves the two computers. From it the app
//! derives, with Argon2id, a 256-bit id the relay stores the file under and a key the file is
//! encrypted with, so the relay and anyone who copies its bucket hold nothing readable and
//! cannot tell which object belongs to which code.

use std::{
    io::{Read, Write},
    path::Path,
};

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::{Rng, RngExt};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::wordlist::TRANSFER_WORDS;

/// Where the relay lives; `BLENDERBASE_TRANSFER_RELAY` overrides it (local development).
pub const TRANSFER_RELAY_DEFAULT: &str = "https://blenderbase-transfer.physicaladdons.workers.dev";
/// Words in a code, plus a four-digit number: 3 × log2(1296) + log2(10000) ≈ 44 bits.
const CODE_WORDS: usize = 3;
const CODE_NUMBER_DIGITS: usize = 4;
/// Argon2id parameters: 64 MiB, three passes. Around a tenth of a second per guess on a desktop,
/// which puts a 44-bit code beyond reach even against a copied bucket.
const KDF_MEMORY_KIB: u32 = 64 * 1024;
const KDF_PASSES: u32 = 3;
const KDF_LANES: u32 = 1;
/// Fixed, versioned salt: the code itself is the random part, and both computers must derive
/// the same values from it.
const KDF_SALT: &[u8] = b"blenderbase-transfer-v1";
/// Uploads above this go as multipart, in parts of `PART_SIZE`; the relay's one-shot limit is 95 MiB.
const ONE_SHOT_LIMIT: u64 = 90 * 1024 * 1024;
const PART_SIZE: u64 = 64 * 1024 * 1024;

const FILE_MAGIC: &[u8; 4] = b"BBTX";
const FILE_VERSION: u8 = 1;
const CHUNK_SIZE: usize = 1024 * 1024;
const NONCE_PREFIX_LEN: usize = 8;
const TAG_LEN: usize = 16;

/// What both computers derive from a code.
#[derive(Debug, Clone)]
pub struct TransferKeys {
    /// 64 hex digits; the relay's object name.
    pub id: String,
    pub key: [u8; 32],
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferReceipt {
    pub size: u64,
    pub expires: String,
}

pub fn relay_url() -> String {
    std::env::var("BLENDERBASE_TRANSFER_RELAY")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| String::from(TRANSFER_RELAY_DEFAULT))
        .trim_end_matches('/')
        .to_string()
}

/// A new code: three words from the list and a four-digit number, all from the OS random source.
pub fn generate_code() -> String {
    let mut rng = rand::rng();
    let mut parts: Vec<String> = (0..CODE_WORDS)
        .map(|_| TRANSFER_WORDS[rng.random_range(0..TRANSFER_WORDS.len())].to_string())
        .collect();
    parts.push(format!("{:04}", rng.random_range(0..10_000u32)));
    parts.join("-")
}

/// The code as typed, brought to its one canonical form: lowercase words separated by single
/// dashes, then the number. Spaces, dots and commas between parts are accepted.
pub fn normalise_code(typed: &str) -> Result<String, String> {
    let parts: Vec<String> = typed
        .to_lowercase()
        .split(|c: char| c == '-' || c.is_whitespace() || c == '.' || c == ',' || c == '_')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();
    if parts.len() != CODE_WORDS + 1 {
        return Err(format!(
            "A transfer code is {} words and a {}-digit number, like brave-otter-4412",
            CODE_WORDS, CODE_NUMBER_DIGITS
        ));
    }
    for word in &parts[..CODE_WORDS] {
        if !TRANSFER_WORDS.contains(&word.as_str()) {
            return Err(format!("'{}' is not a word a transfer code can contain; check the spelling", word));
        }
    }
    let number = &parts[CODE_WORDS];
    if number.len() != CODE_NUMBER_DIGITS || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("A transfer code ends with a {}-digit number", CODE_NUMBER_DIGITS));
    }
    Ok(parts.join("-"))
}

/// Argon2id over the canonical code gives 64 bytes: the first half names the object (after one
/// more hash, so the name reveals nothing about the key), the second half is the key.
pub fn derive_keys(code: &str) -> Result<TransferKeys, String> {
    let code = normalise_code(code)?;
    let params = Params::new(KDF_MEMORY_KIB, KDF_PASSES, KDF_LANES, Some(64))
        .map_err(|e| format!("Key derivation is misconfigured: {}", e))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut material = [0u8; 64];
    argon
        .hash_password_into(code.as_bytes(), KDF_SALT, &mut material)
        .map_err(|e| format!("Could not derive the transfer key: {}", e))?;
    let id = format!("{:x}", Sha256::digest(&material[..32]));
    let mut key = [0u8; 32];
    key.copy_from_slice(&material[32..]);
    Ok(TransferKeys { id, key })
}

fn chunk_nonce(prefix: &[u8; NONCE_PREFIX_LEN], index: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..NONCE_PREFIX_LEN].copy_from_slice(prefix);
    nonce[NONCE_PREFIX_LEN..].copy_from_slice(&index.to_le_bytes());
    nonce
}

/// Each chunk's associated data carries its index and whether it is the last, so chunks
/// cannot be reordered, dropped or cut off without the tag failing.
fn chunk_aad(index: u32, last: bool) -> [u8; 5] {
    let mut aad = [0u8; 5];
    aad[..4].copy_from_slice(&index.to_le_bytes());
    aad[4] = u8::from(last);
    aad
}

/// Encrypts `input` into `output` chunk by chunk and returns the output size. Blocking file work.
pub fn encrypt_file(key: &[u8; 32], input: &Path, output: &Path) -> Result<u64, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Bad key: {}", e))?;
    let mut prefix = [0u8; NONCE_PREFIX_LEN];
    rand::rng().fill_bytes(&mut prefix);
    let total = std::fs::metadata(input)
        .map_err(|e| format!("Could not read {}: {}", input.display(), e))?
        .len();
    let mut reader = std::fs::File::open(input).map_err(|e| format!("Could not read {}: {}", input.display(), e))?;
    let mut writer = std::fs::File::create(output).map_err(|e| format!("Could not create {}: {}", output.display(), e))?;
    let mut header = Vec::with_capacity(4 + 1 + 4 + NONCE_PREFIX_LEN);
    header.extend_from_slice(FILE_MAGIC);
    header.push(FILE_VERSION);
    header.extend_from_slice(&(CHUNK_SIZE as u32).to_le_bytes());
    header.extend_from_slice(&prefix);
    writer.write_all(&header).map_err(|e| format!("Could not write {}: {}", output.display(), e))?;
    let mut written = header.len() as u64;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut consumed = 0u64;
    let mut index = 0u32;
    loop {
        let read = read_full(&mut reader, &mut buffer).map_err(|e| format!("Could not read {}: {}", input.display(), e))?;
        consumed += read as u64;
        let last = consumed >= total;
        let ciphertext = cipher
            .encrypt(&Nonce::from(chunk_nonce(&prefix, index)), Payload { msg: &buffer[..read], aad: &chunk_aad(index, last) })
            .map_err(|_| String::from("Encryption failed"))?;
        writer
            .write_all(&(ciphertext.len() as u32).to_le_bytes())
            .and_then(|_| writer.write_all(&ciphertext))
            .map_err(|e| format!("Could not write {}: {}", output.display(), e))?;
        written += 4 + ciphertext.len() as u64;
        index = index.checked_add(1).ok_or_else(|| String::from("The file is too large to encrypt"))?;
        if last {
            break;
        }
    }
    writer.flush().map_err(|e| format!("Could not write {}: {}", output.display(), e))?;
    Ok(written)
}

/// Decrypts a file written by [`encrypt_file`]. Any change to the ciphertext, a missing chunk or
/// a cut-off end fails and leaves no output behind.
pub fn decrypt_file(key: &[u8; 32], input: &Path, output: &Path) -> Result<u64, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Bad key: {}", e))?;
    let mut reader = std::fs::File::open(input).map_err(|e| format!("Could not read {}: {}", input.display(), e))?;
    let mut header = [0u8; 4 + 1 + 4 + NONCE_PREFIX_LEN];
    reader
        .read_exact(&mut header)
        .map_err(|_| String::from("This is not a Blenderbase transfer"))?;
    if &header[..4] != FILE_MAGIC || header[4] != FILE_VERSION {
        return Err(String::from("This is not a Blenderbase transfer, or it was made by a newer version"));
    }
    let chunk_size = u32::from_le_bytes([header[5], header[6], header[7], header[8]]) as usize;
    if chunk_size == 0 || chunk_size > 64 * 1024 * 1024 {
        return Err(String::from("The transfer has an unusable chunk size"));
    }
    let mut prefix = [0u8; NONCE_PREFIX_LEN];
    prefix.copy_from_slice(&header[9..]);
    let mut writer = std::fs::File::create(output).map_err(|e| format!("Could not create {}: {}", output.display(), e))?;
    let outcome = decrypt_chunks(&cipher, &prefix, chunk_size, &mut reader, &mut writer);
    drop(writer);
    if outcome.is_err() {
        let _ = std::fs::remove_file(output);
    }
    outcome
}

fn decrypt_chunks(
    cipher: &Aes256Gcm,
    prefix: &[u8; NONCE_PREFIX_LEN],
    chunk_size: usize,
    reader: &mut std::fs::File,
    writer: &mut std::fs::File,
) -> Result<u64, String> {
    let mut written = 0u64;
    let mut index = 0u32;
    let mut length = [0u8; 4];
    loop {
        if reader.read_exact(&mut length).is_err() {
            return Err(String::from("The transfer is incomplete: the end is missing"));
        }
        let length = u32::from_le_bytes(length) as usize;
        if length < TAG_LEN || length > chunk_size + TAG_LEN {
            return Err(String::from("The transfer is damaged"));
        }
        let mut ciphertext = vec![0u8; length];
        reader
            .read_exact(&mut ciphertext)
            .map_err(|_| String::from("The transfer is incomplete: a chunk is cut off"))?;
        // The last chunk is the one that decrypts under `last = true`; every chunk is tried as
        // a middle chunk first, so a middle chunk that pretends to be last is caught too.
        let nonce = Nonce::from(chunk_nonce(prefix, index));
        let (plaintext, last) = match cipher.decrypt(&nonce, Payload { msg: &ciphertext, aad: &chunk_aad(index, false) }) {
            Ok(p) => (p, false),
            Err(_) => match cipher.decrypt(&nonce, Payload { msg: &ciphertext, aad: &chunk_aad(index, true) }) {
                Ok(p) => (p, true),
                Err(_) => return Err(String::from("The transfer could not be decrypted: wrong code, or the file was changed")),
            },
        };
        writer.write_all(&plaintext).map_err(|e| format!("Could not write the setup file: {}", e))?;
        written += plaintext.len() as u64;
        index = index.checked_add(1).ok_or_else(|| String::from("The transfer is too large"))?;
        if last {
            break;
        }
    }
    let mut trailing = [0u8; 1];
    if reader.read(&mut trailing).map_err(|e| format!("Could not read the transfer: {}", e))? != 0 {
        return Err(String::from("The transfer has data after its end"));
    }
    writer.flush().map_err(|e| format!("Could not write the setup file: {}", e))?;
    Ok(written)
}

fn read_full(reader: &mut std::fs::File, buffer: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        let n = reader.read(&mut buffer[filled..])?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}

/// One line per step for the status bar.
pub type TransferProgress = std::sync::Arc<dyn Fn(String) + Send + Sync>;

/// Sends a request, and once more on a fresh connection when the first attempt fails to
/// send at all: a pooled connection the relay closed while a large file was being decrypted
/// fails on its first reuse. Only requests without a large body come through here.
async fn send_retrying(request: reqwest::RequestBuilder) -> Result<reqwest::Response, String> {
    let again = request.try_clone();
    match request.send().await {
        Ok(response) => Ok(response),
        Err(first) => {
            if let Some(again) = again {
                if let Ok(response) = again.send().await {
                    return Ok(response);
                }
            }
            Err(format!("Could not reach the transfer relay: {}", first))
        }
    }
}

async fn relay_error(response: reqwest::Response, what: &str) -> String {
    let status = response.status();
    let detail = response
        .json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| status.to_string());
    format!("{}: {}", what, detail)
}

/// Uploads an encrypted file under the transfer's id: in one request when it is small enough,
/// otherwise in parts. Returns what the relay reports about the stored object.
pub async fn upload_transfer(
    client: &reqwest::Client,
    relay: &str,
    keys: &TransferKeys,
    file: &Path,
    progress: &TransferProgress,
) -> Result<TransferReceipt, String> {
    let size = std::fs::metadata(file).map_err(|e| format!("Could not read {}: {}", file.display(), e))?.len();
    let base = format!("{}/v1/transfers/{}", relay, keys.id);
    if size <= ONE_SHOT_LIMIT {
        progress(format!("Uploading {} MB…", size / 1024 / 1024));
        let bytes = tokio::fs::read(file).await.map_err(|e| format!("Could not read {}: {}", file.display(), e))?;
        let response = client
            .put(&base)
            .header(reqwest::header::CONTENT_LENGTH, size)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(bytes)
            .send()
            .await
            .map_err(|e| format!("Could not reach the transfer relay: {}", e))?;
        if !response.status().is_success() {
            return Err(relay_error(response, "The relay refused the upload").await);
        }
        let body: serde_json::Value = response.json().await.map_err(|e| format!("Unreadable answer from the relay: {}", e))?;
        return Ok(TransferReceipt {
            size,
            expires: body.get("expires").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        });
    }

    let started = send_retrying(client.post(format!("{}/multipart", base))).await?;
    if !started.status().is_success() {
        return Err(relay_error(started, "The relay refused the upload").await);
    }
    let started: serde_json::Value = started.json().await.map_err(|e| format!("Unreadable answer from the relay: {}", e))?;
    let upload_id = started
        .get("uploadId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| String::from("The relay did not start the upload"))?
        .to_string();
    let part_count = size.div_ceil(PART_SIZE);
    let mut parts: Vec<serde_json::Value> = Vec::with_capacity(part_count as usize);
    let mut reader = tokio::fs::File::open(file).await.map_err(|e| format!("Could not read {}: {}", file.display(), e))?;
    for number in 1..=part_count {
        progress(format!("Uploading part {} of {}…", number, part_count));
        let mut buffer = Vec::with_capacity(PART_SIZE as usize);
        let mut limited = tokio::io::AsyncReadExt::take(&mut reader, PART_SIZE);
        tokio::io::AsyncReadExt::read_to_end(&mut limited, &mut buffer)
            .await
            .map_err(|e| format!("Could not read {}: {}", file.display(), e))?;
        let length = buffer.len() as u64;
        let response = client
            .put(format!("{}/multipart/{}/{}", base, upload_id, number))
            .header(reqwest::header::CONTENT_LENGTH, length)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(buffer)
            .send()
            .await
            .map_err(|e| format!("Could not reach the transfer relay: {}", e))?;
        if !response.status().is_success() {
            let message = relay_error(response, "The relay refused a part of the upload").await;
            let _ = client.delete(format!("{}/multipart/{}", base, upload_id)).send().await;
            return Err(message);
        }
        let part: serde_json::Value = response.json().await.map_err(|e| format!("Unreadable answer from the relay: {}", e))?;
        parts.push(serde_json::json!({ "partNumber": number, "etag": part.get("etag").and_then(|v| v.as_str()).unwrap_or_default() }));
    }
    let completed = send_retrying(
        client
            .post(format!("{}/multipart/{}/complete", base, upload_id))
            .json(&serde_json::json!({ "parts": parts })),
    )
    .await?;
    if !completed.status().is_success() {
        return Err(relay_error(completed, "The relay could not finish the upload").await);
    }
    let completed: serde_json::Value = completed.json().await.map_err(|e| format!("Unreadable answer from the relay: {}", e))?;
    Ok(TransferReceipt {
        size,
        expires: completed.get("expires").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    })
}

/// Downloads the encrypted file for a transfer into `destination` and returns its size.
pub async fn download_transfer(
    client: &reqwest::Client,
    relay: &str,
    keys: &TransferKeys,
    destination: &Path,
    progress: &TransferProgress,
) -> Result<u64, String> {
    let response = send_retrying(client.get(format!("{}/v1/transfers/{}", relay, keys.id))).await?;
    if !response.status().is_success() {
        return Err(relay_error(response, "The relay has nothing for this code").await);
    }
    let total = response.content_length().unwrap_or(0);
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
    }
    let mut file = tokio::fs::File::create(destination)
        .await
        .map_err(|e| format!("Could not create {}: {}", destination.display(), e))?;
    let mut stream = response.bytes_stream();
    let mut received = 0u64;
    let mut last_percent = u64::MAX;
    while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
        let chunk = chunk.map_err(|e| format!("The download broke off: {}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("Could not write {}: {}", destination.display(), e))?;
        received += chunk.len() as u64;
        if total > 0 {
            let percent = received * 100 / total;
            if percent != last_percent && percent % 5 == 0 {
                last_percent = percent;
                progress(format!("Downloading… {}%", percent));
            }
        }
    }
    tokio::io::AsyncWriteExt::flush(&mut file)
        .await
        .map_err(|e| format!("Could not write {}: {}", destination.display(), e))?;
    if total > 0 && received != total {
        return Err(String::from("The download broke off before the end"));
    }
    Ok(received)
}

/// Asks the relay to drop the object; the receiving computer does this once it has the file.
pub async fn delete_transfer(client: &reqwest::Client, relay: &str, keys: &TransferKeys) -> Result<(), String> {
    let response = send_retrying(client.delete(format!("{}/v1/transfers/{}", relay, keys.id))).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(relay_error(response, "The relay did not remove the transfer").await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-transfer-{}-{}", label, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn codes_have_the_expected_shape_and_normalise() {
        let code = generate_code();
        let parts: Vec<&str> = code.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert!(parts[..3].iter().all(|w| TRANSFER_WORDS.contains(w)));
        assert_eq!(parts[3].len(), 4);
        assert_eq!(normalise_code(&code).unwrap(), code);
        assert_eq!(normalise_code(&code.to_uppercase().replace('-', " ")).unwrap(), code);
        assert_eq!(normalise_code(" Acid, Acorn . acre 0042 ").unwrap(), "acid-acorn-acre-0042");
        assert!(normalise_code("acid-acorn-0042").is_err(), "three words are required");
        assert!(normalise_code("acid-acorn-zzzzz-0042").is_err(), "unknown word");
        assert!(normalise_code("acid-acorn-acre-42").is_err(), "four digits are required");
    }

    #[test]
    fn keys_follow_the_code_and_only_the_code() {
        let a = derive_keys("acid-acorn-acre-0042").unwrap();
        let again = derive_keys("ACID acorn acre 0042").unwrap();
        let b = derive_keys("acid-acorn-acre-0043").unwrap();
        assert_eq!(a.id, again.id);
        assert_eq!(a.key, again.key);
        assert_ne!(a.id, b.id);
        assert_ne!(a.key, b.key);
        assert_eq!(a.id.len(), 64);
        assert_ne!(format!("{:x}", Sha256::digest(a.key)), a.id, "the id is not a hash of the key");
    }

    #[test]
    fn encrypted_files_round_trip_and_refuse_tampering() {
        let dir = temp_dir("crypt");
        let plain = dir.join("setup.bbsetup");
        // Longer than two chunks, and not a multiple of the chunk size.
        let content: Vec<u8> = (0..(2 * CHUNK_SIZE + 12345)).map(|i| (i % 251) as u8).collect();
        std::fs::write(&plain, &content).unwrap();
        let keys = derive_keys("acid-acorn-acre-0042").unwrap();
        let encrypted = dir.join("setup.enc");
        let size = encrypt_file(&keys.key, &plain, &encrypted).unwrap();
        assert_eq!(size, std::fs::metadata(&encrypted).unwrap().len());
        assert!(size > content.len() as u64);

        let restored = dir.join("restored.bbsetup");
        assert_eq!(decrypt_file(&keys.key, &encrypted, &restored).unwrap(), content.len() as u64);
        assert_eq!(std::fs::read(&restored).unwrap(), content);

        let wrong = derive_keys("acid-acorn-acre-0043").unwrap();
        let refused = decrypt_file(&wrong.key, &encrypted, &dir.join("wrong.bbsetup")).unwrap_err();
        assert!(refused.contains("wrong code"), "{}", refused);
        assert!(!dir.join("wrong.bbsetup").exists(), "nothing is left behind");

        let mut bytes = std::fs::read(&encrypted).unwrap();
        let flipped = bytes.len() / 2;
        bytes[flipped] ^= 0x01;
        std::fs::write(dir.join("tampered.enc"), &bytes).unwrap();
        assert!(decrypt_file(&keys.key, &dir.join("tampered.enc"), &dir.join("t.bbsetup")).is_err());

        let cut = std::fs::read(&encrypted).unwrap();
        std::fs::write(dir.join("cut.enc"), &cut[..cut.len() - 40]).unwrap();
        let refused = decrypt_file(&keys.key, &dir.join("cut.enc"), &dir.join("c.bbsetup")).unwrap_err();
        assert!(refused.contains("incomplete"), "{}", refused);

        let mut swapped = std::fs::read(&encrypted).unwrap();
        // Drop the first chunk entirely: the second one then arrives as index 0 and fails.
        let header = 4 + 1 + 4 + NONCE_PREFIX_LEN;
        let first_len = u32::from_le_bytes([swapped[header], swapped[header + 1], swapped[header + 2], swapped[header + 3]]) as usize;
        swapped.drain(header..header + 4 + first_len);
        std::fs::write(dir.join("dropped.enc"), &swapped).unwrap();
        assert!(decrypt_file(&keys.key, &dir.join("dropped.enc"), &dir.join("d.bbsetup")).is_err());

        let empty = dir.join("empty.bbsetup");
        std::fs::write(&empty, b"").unwrap();
        encrypt_file(&keys.key, &empty, &dir.join("empty.enc")).unwrap();
        assert_eq!(decrypt_file(&keys.key, &dir.join("empty.enc"), &dir.join("empty2.bbsetup")).unwrap(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Against a running relay (`npm run dev` in `transfer-relay/`, or a deployed one named by
    /// `BLENDERBASE_TRANSFER_RELAY`): upload, download, decrypt, delete.
    #[tokio::test]
    #[ignore = "needs the transfer relay; set BLENDERBASE_TRANSFER_RELAY"]
    async fn a_transfer_goes_through_the_relay() {
        let relay = relay_url();
        let client = reqwest::Client::new();
        let dir = temp_dir("relay");
        let progress: TransferProgress = std::sync::Arc::new(|m: String| println!("  {}", m));
        let big = std::env::var("BLENDERBASE_TEST_BIG_TRANSFER").is_ok();
        // Small: one request. Big: three parts, which also crosses the relay's one-shot limit.
        let content: Vec<u8> = (0..(if big { 150 * 1024 * 1024 } else { 3 * CHUNK_SIZE + 77 })).map(|i| (i % 253) as u8).collect();
        std::fs::write(dir.join("setup.bbsetup"), &content).unwrap();
        let code = generate_code();
        let keys = derive_keys(&code).unwrap();
        encrypt_file(&keys.key, &dir.join("setup.bbsetup"), &dir.join("setup.enc")).unwrap();

        let receipt = upload_transfer(&client, &relay, &keys, &dir.join("setup.enc"), &progress).await.unwrap();
        println!("uploaded {} bytes under {}, expires {}", receipt.size, code, receipt.expires);
        let again = upload_transfer(&client, &relay, &keys, &dir.join("setup.enc"), &progress).await;
        assert!(again.is_err(), "an id cannot be overwritten");

        let received = download_transfer(&client, &relay, &keys, &dir.join("received.enc"), &progress).await.unwrap();
        assert_eq!(received, receipt.size);
        decrypt_file(&keys.key, &dir.join("received.enc"), &dir.join("received.bbsetup")).unwrap();
        assert_eq!(std::fs::read(dir.join("received.bbsetup")).unwrap(), content);

        delete_transfer(&client, &relay, &keys).await.unwrap();
        assert!(download_transfer(&client, &relay, &keys, &dir.join("gone.enc"), &progress).await.is_err());
        let other = derive_keys(&generate_code()).unwrap();
        assert!(download_transfer(&client, &relay, &other, &dir.join("none.enc"), &progress).await.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
