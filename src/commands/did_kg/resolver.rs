use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::Context;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};

use crate::commands::settings::Settings;

#[derive(Debug)]
pub struct TwinDelegation {
    did: String,
    key_name: String,
}

#[derive(Debug)]
pub struct TwinDidDocument {
    did: String,
    key_name: String,
    update_time: DateTime<Utc>,
    delegations: Vec<TwinDelegation>,
}

async fn get_did_document_cached(
    settings: &Settings,
    did: &str,
) -> Result<DidDocument, anyhow::Error> {
    let file_path = format!("resolver/{}.json", did);

    let file = File::open(&file_path);

    let doc = match file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let doc: DidDocument =
                serde_json::from_str(&contents).context("failed to parse the did doc")?;

            doc
        }
        Err(_) => {
            let url = format!("{}/1.0/discover/{}", &settings.iotics.resolver_address, did);
            let result = reqwest::get(&url).await.context("failed to get did url")?;
            let json: ResolverResponse = result
                .json()
                .await
                .context(format!("failed to parse did response {}", &url))?;

            // skip the first part - header, ignore the third part - verification
            let token: String = json.token.split('.').skip(1).take(1).collect();

            let token = base64::decode(token).context("failed to decode the token")?;
            let doc: DidDocument =
                serde_json::from_slice(&token).context("failed to parse the did doc")?;

            let mut file = File::create(&file_path)?;
            let value = serde_json::json!(doc);
            file.write_all(value.to_string().as_bytes())?;

            doc
        }
    };

    Ok(doc)
}

pub async fn parse_did_document(
    settings: &Settings,
    did: &str,
) -> Result<TwinDidDocument, anyhow::Error> {
    let result = get_did_document_cached(settings, did).await?;

    let delegate_control = result.doc.delegate_control.unwrap_or_default();
    let delegate_authentication = result.doc.delegate_authentication.unwrap_or_default();

    let doc_delegations = delegate_control
        .iter()
        .chain(delegate_authentication.iter())
        .collect::<Vec<_>>();

    let mut delegations = Vec::new();

    for delegation in doc_delegations {
        let (did, key_name) = delegation
            .controller
            .split_once("#")
            .ok_or_else(|| anyhow::anyhow!("failed to split delegation agent did"))?;

        delegations.push({
            TwinDelegation {
                did: did.to_string(),
                key_name: key_name.to_string(),
            }
        });
    }

    let (did, key_name) = result
        .iss
        .split_once("#")
        .ok_or_else(|| anyhow::anyhow!("failed to split twin did"))?;

    let doc = TwinDidDocument {
        did: did.to_string(),
        key_name: key_name.to_string(),
        update_time: result.doc.update_time,
        delegations,
    };

    Ok(doc)
}

#[derive(serde::Deserialize)]
struct ResolverResponse {
    token: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Delegation {
    controller: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DidDocumentDoc {
    #[serde(rename(serialize = "updateTime", deserialize = "updateTime"))]
    #[serde(with = "ts_milliseconds")]
    update_time: DateTime<Utc>,
    #[serde(rename(serialize = "ioticsDIDType", deserialize = "ioticsDIDType"))]
    iotics_did_type: String,
    #[serde(rename(serialize = "delegateControl", deserialize = "delegateControl"))]
    delegate_control: Option<Vec<Delegation>>,
    #[serde(rename(
        serialize = "delegateAuthentication",
        deserialize = "delegateAuthentication"
    ))]
    delegate_authentication: Option<Vec<Delegation>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DidDocument {
    doc: DidDocumentDoc,
    iss: String,
}
