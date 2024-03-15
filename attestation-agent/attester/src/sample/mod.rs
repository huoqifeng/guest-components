// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use super::Attester;
use anyhow::*;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};

// Sample attester is always supported
pub fn detect_platform() -> bool {
    true
}

// A simple example of TEE evidence.
#[derive(Serialize, Deserialize, Debug)]
struct SampleQuote {
    svn: String,
    report_data: String,
}

#[derive(Debug, Default)]
pub struct SampleAttester {}

#[async_trait::async_trait]
impl Attester for SampleAttester {
    async fn get_evidence(&self, report_data: Vec<u8>) -> Result<String> {
        let report_data_str = String::from_utf8(report_data)?;
        let mut hasher = Sha384::new();
        hasher.update(report_data_str);
        let ehd = hasher.finalize().to_vec();

        let evidence = SampleQuote {
            svn: "1".to_string(),
            report_data: base64::engine::general_purpose::STANDARD.encode(ehd),
        };

        serde_json::to_string(&evidence).map_err(|_| anyhow!("Serialize sample evidence failed"))
    }
}
