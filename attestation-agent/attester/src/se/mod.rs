// Copyright (C) Copyright IBM Corp. 2024
//
// SPDX-License-Identifier: Apache-2.0
//

use super::Attester;
use crate::se::seattest::FakeSeAttest;
use crate::se::seattest::SeImplAttester;
use anyhow::*;
use base64::prelude::*;
use kbs_types::TeePubKey;
use serde::{Deserialize, Serialize};
use serde_json;

pub mod seattest;

// TODO move this to lib?
#[derive(Serialize, Deserialize, Debug)]
struct RuntimeData {
    #[serde(alias = "tee-pubkey")]
    tee_pubkey: TeePubKey,
    nonce: String,
    #[serde(alias = "extra-params")]
    extra_params: String,
}

pub fn detect_platform() -> bool {
    // TODO replace FakeSeAttest with real crate
    let attester = FakeSeAttest::default();
    attester.is_se_guest()
}

#[derive(Serialize, Deserialize)]
struct SeEvidence {
    quote: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct SeAttester {}

#[async_trait::async_trait]
impl Attester for SeAttester {
    async fn get_evidence(&self, runtime_data: Vec<u8>) -> Result<String> {
        // TODO replace FakeSeAttest with real crate
        let attester = FakeSeAttest::default();

        let data: RuntimeData = serde_json::from_str(
            std::str::from_utf8(&runtime_data[..]).expect("Found invalid UTF-8"),
        )?;

        let attestation_request_bin = data.extra_params.into_bytes();

        let userdata = "userdata".as_bytes().to_vec();
        let evidence = attester.perform(attestation_request_bin, userdata).await?;

        Ok(BASE64_STANDARD.encode(evidence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_se_get_evidence() {
        let attester = SeAttester::default();
        let report_data: Vec<u8> = vec![0; 64];

        let evidence = attester.get_evidence(report_data).await;
        assert!(evidence.is_ok());
    }
}
