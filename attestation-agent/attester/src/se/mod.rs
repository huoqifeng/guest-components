// Copyright (C) Copyright IBM Corp. 2024
//
// SPDX-License-Identifier: Apache-2.0
//

use super::Attester;
use anyhow::*;
use log::debug;

pub mod ibmse;

pub fn detect_platform() -> bool {
    ibmse::is_se_guest()
}

#[derive(Debug, Default)]
pub struct SeAttester {}

#[async_trait::async_trait]
impl Attester for SeAttester {
    async fn get_evidence(&self, attestation_request: Vec<u8>) -> Result<String> {
        let userdata = ibmse::calc_userdata()?;
        debug!("userdata json: {:#?}", &userdata.clone());
        // attestation_request is serialized SeAttestationRequest String bytes
        let evidence = ibmse::perform(&attestation_request, &userdata)?;
        debug!("response json: {:#?}", evidence.clone());
        serde_json::to_string(&evidence).context("Serialize SE evidence failed")
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
