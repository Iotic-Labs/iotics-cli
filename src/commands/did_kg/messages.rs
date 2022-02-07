use actix::Message;

use iotics_grpc_client::search::SearchResponsePayload;

use crate::commands::did_kg::resolver::TwinDidDocument;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ProcessHostMessage {
    pub payload: SearchResponsePayload,
    pub token: String,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HostResultMessage {
    pub host_did: String,
    pub url: String,
    pub twins: Vec<TwinDidDocument>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HostEmptyResultMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GenerateKbMessage;
