//! Guards the in-app update path end to end at build time:
//! the NSIS installer we ship must be signed by the very key that
//! `tauri.conf.json` bakes into the app, or every client's update fails
//! signature verification and silently never installs.
//!
//! Skips (rather than fails) when the release bundle hasn't been built, so a
//! fresh checkout running `cargo test` isn't broken by a missing artifact.

use minisign_verify::{PublicKey, Signature};
use std::path::PathBuf;

fn conf_pubkey() -> String {
    let conf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string("tauri.conf.json").unwrap()).unwrap();
    conf["plugins"]["updater"]["pubkey"].as_str().unwrap().to_string()
}

fn app_version() -> String {
    let conf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string("tauri.conf.json").unwrap()).unwrap();
    conf["version"].as_str().unwrap().to_string()
}

/// The config pubkey is stored base64-of-the-minisign-pubkey-file.
fn decode_pubkey(b64: &str) -> PublicKey {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(b64).expect("pubkey is base64");
    let text = String::from_utf8(decoded).expect("pubkey file is utf-8");
    PublicKey::decode(text.trim()).expect("valid minisign public key")
}

#[test]
fn shipped_installer_is_signed_by_the_key_the_app_trusts() {
    let version = app_version();
    let installer = PathBuf::from(format!(
        "target/release/bundle/nsis/Korio_{version}_x64-setup.exe"
    ));
    let sig_path = installer.with_extension("exe.sig");
    if !installer.exists() || !sig_path.exists() {
        eprintln!("skipping: no release bundle for v{version} (run `npm run tauri build`)");
        return;
    }

    let pk = decode_pubkey(&conf_pubkey());

    // The `.sig` asset is base64 of the minisign signature file.
    use base64::Engine;
    let sig_text = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(std::fs::read_to_string(&sig_path).unwrap().trim())
            .expect("sig asset is base64"),
    )
    .expect("signature file is utf-8");
    let sig = Signature::decode(sig_text.trim()).expect("valid minisign signature");

    let bytes = std::fs::read(&installer).unwrap();
    pk.verify(&bytes, &sig, false)
        .expect("installer signature must verify against the pubkey in tauri.conf.json");
}

#[test]
fn a_tampered_installer_is_rejected() {
    let version = app_version();
    let installer = PathBuf::from(format!(
        "target/release/bundle/nsis/Korio_{version}_x64-setup.exe"
    ));
    let sig_path = installer.with_extension("exe.sig");
    if !installer.exists() || !sig_path.exists() {
        return;
    }

    use base64::Engine;
    let pk = decode_pubkey(&conf_pubkey());
    let sig_text = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(std::fs::read_to_string(&sig_path).unwrap().trim())
            .unwrap(),
    )
    .unwrap();
    let sig = Signature::decode(sig_text.trim()).unwrap();

    let mut bytes = std::fs::read(&installer).unwrap();
    bytes[0] ^= 0xFF; // flip a byte — this is the attack the signature exists to stop
    assert!(pk.verify(&bytes, &sig, false).is_err(), "a modified installer must not verify");
}
