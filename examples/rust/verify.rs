// Copyright 2025 Cloudflare, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use web_bot_auth::{
    SignatureAgentLink, WebBotAuthSignedMessage, WebBotAuthVerifier,
    components::{CoveredComponent, DerivedComponent, HTTPField},
    keyring::{Algorithm, KeyRing},
    message_signatures::SignedMessage,
};

struct MySignedMsg;

impl SignedMessage for MySignedMsg {
    fn fetch_all_signature_headers(&self) -> Vec<String> {
        vec!["sig1=:GXzHSRZ9Sf6WwLOZjxAhfE6WEUPfDMrVBJITsL2sbG8gtcZgqKe2Yn7uavk0iNQrfcPzgGq8h8Pk5osNGqdtCw==:".to_owned()]
    }
    fn fetch_all_signature_inputs(&self) -> Vec<String> {
        vec![r#"sig1=("@authority" "signature-agent");alg="ed25519";keyid="poqkLGiymh_W0uP6PZFw-dvez3QJT5SolqXBCW38r0U";nonce="ZO3/XMEZjrvSnLtAP9M7jK0WGQf3J+pbmQRUpKDhF9/jsNCWqUh2sq+TH4WTX3/GpNoSZUa8eNWMKqxWp2/c2g==";tag="web-bot-auth";created=1749332605;expires=1749332615"#.to_owned()]
    }
    fn lookup_component(&self, name: &CoveredComponent) -> Option<String> {
        match name {
            CoveredComponent::Derived(DerivedComponent::Authority { .. }) => {
                Some("example.com".to_string())
            }
            CoveredComponent::HTTP(HTTPField { name, .. }) => {
                if name == "signature-agent" {
                    return Some(String::from("\"https://myexample.com\""));
                }
                None
            }
            _ => None,
        }
    }
}

impl WebBotAuthSignedMessage for MySignedMsg {
    fn fetch_all_signature_agents(&self) -> Vec<String> {
        vec!["\"https://myexample.com\"".into()]
    }
}

fn main() {
    // Verifying a Web Bot Auth message
    let public_key = [
        0x26, 0xb4, 0x0b, 0x8f, 0x93, 0xff, 0xf3, 0xd8, 0x97, 0x11, 0x2f, 0x7e, 0xbc, 0x58, 0x2b,
        0x23, 0x2d, 0xbd, 0x72, 0x51, 0x7d, 0x08, 0x2f, 0xe8, 0x3c, 0xfb, 0x30, 0xdd, 0xce, 0x43,
        0xd1, 0xbb,
    ];
    let mut keyring = KeyRing::default();
    keyring.import_raw(
        "poqkLGiymh_W0uP6PZFw-dvez3QJT5SolqXBCW38r0U".to_string(),
        Algorithm::Ed25519,
        public_key.to_vec(),
    );
    let test = MySignedMsg {};
    let verifier = WebBotAuthVerifier::parse(&test).unwrap();
    let advisory = verifier
        .get_parsed_label()
        .base
        .parameters
        .details
        .possibly_insecure(|_| false);
    for url in verifier.get_signature_agents().iter() {
        assert_eq!(
            url,
            &SignatureAgentLink::External("https://myexample.com".into())
        )
    }
    // Since the expiry date is in the past.
    assert!(advisory.is_expired.unwrap_or(true));
    assert!(!advisory.nonce_is_invalid.unwrap_or(true));
    assert!(verifier.verify(&keyring, None).is_ok());
}
