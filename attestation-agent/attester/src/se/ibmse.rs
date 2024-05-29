// Copyright (C) Copyright IBM Corp. 2024
//
// SPDX-License-Identifier: Apache-2.0
//

// Copyright (C) Copyright IBM Corp. 2024
//
// SPDX-License-Identifier: Apache-2.0
//

use anyhow::{anyhow, Result};
use pv::{
    request::BootHdrTags,
    uv::{AttestationCmd, ConfigUid, UvDevice},
};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_with::{base64::Base64, serde_as};

#[allow(unreachable_code)]
pub fn is_se_guest() -> bool {
    #[cfg(not(target_arch = "s390x"))]
    return false;

    let v = std::fs::read("/sys/firmware/uv/prot_virt_guest").unwrap_or_else(|_| vec![0]);
    let v: u8 = String::from_utf8_lossy(&v[..1]).parse().unwrap_or(0);
    v == 1
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    #[serde_as(as = "Base64")]
    image_btph: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SeAttestationRequest {
    #[serde_as(as = "Base64")]
    request_blob: Vec<u8>,
    measurement_size: u32,
    additional_size: u32,
    #[serde_as(as = "Base64")]
    encr_measurement_key: Vec<u8>,
    #[serde_as(as = "Base64")]
    encr_request_nonce: Vec<u8>,
    #[serde_as(as = "Base64")]
    image_hdr_tags: BootHdrTags,
}

impl SeAttestationRequest {
    pub fn from_slice(request: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(request).unwrap())
    }

    pub fn from_string(request: &str) -> Result<Self> {
        Ok(serde_json::from_str(request).unwrap())
    }

    pub fn to_uvc(&self, user_data: &[u8]) -> Result<AttestationCmd> {
        let cmd = AttestationCmd::new_request(
            self.request_blob.clone().into(),
            Some(user_data.to_vec()),
            self.measurement_size,
            self.additional_size,
        )?;
        Ok(cmd)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeAttestationResponse {
    #[serde_as(as = "Base64")]
    measurement: Vec<u8>,
    #[serde_as(as = "Base64")]
    additional_data: Vec<u8>,
    #[serde_as(as = "Base64")]
    user_data: Vec<u8>,
    #[serde_as(as = "Base64")]
    cuid: ConfigUid,
    #[serde_as(as = "Base64")]
    encr_measurement_key: Vec<u8>,
    #[serde_as(as = "Base64")]
    encr_request_nonce: Vec<u8>,
    #[serde_as(as = "Base64")]
    image_hdr_tags: BootHdrTags,
}

impl SeAttestationResponse {
    pub fn create(
        measurement: &[u8],
        additional_data: &[u8],
        user_data: &[u8],
        cuid: &ConfigUid,
        encr_measurement_key: &[u8],
        encr_request_nonce: &[u8],
        image_hdr_tags: &BootHdrTags,
    ) -> Result<Self> {
        Ok(Self {
            measurement: measurement.to_vec(),
            additional_data: additional_data.to_vec(),
            user_data: user_data.to_vec(),
            cuid: *cuid,
            encr_measurement_key: encr_measurement_key.to_vec(),
            encr_request_nonce: encr_request_nonce.to_vec(),
            image_hdr_tags: *image_hdr_tags,
        })
    }
}

pub fn calc_userdata() -> Result<UserData> {
    // TODO, calculate userdata based on the boot partition
    let image_btph = "ddddddd";
    Ok(UserData {
        image_btph: image_btph.into(),
    })
}

pub fn perform(req: &[u8], userdata: &UserData) -> Result<SeAttestationResponse> {
    // req is serialized SeAttestationRequest String bytes
    let req_str = std::str::from_utf8(req)?;
    let request = SeAttestationRequest::from_string(req_str)?;
    let user_data = serde_json::to_vec(userdata)?;
    let mut uvc: AttestationCmd = request.to_uvc(&user_data)?;
    let uv = UvDevice::open()?;
    uv.send_cmd(&mut uvc)?;

    let measurement = uvc.measurement().to_vec().clone();
    let cuid = uvc.cuid();
    let additional_data = uvc
        .additional_owned()
        .ok_or(anyhow!("Failed to get additinal data."))?;
    let encr_measurement_key = &request.encr_measurement_key;
    let encr_request_nonce = &request.encr_request_nonce;
    let image_hdr_tags = &request.image_hdr_tags;

    SeAttestationResponse::create(
        &measurement,
        &additional_data,
        &user_data,
        cuid,
        encr_measurement_key,
        encr_request_nonce,
        image_hdr_tags,
    )
}
