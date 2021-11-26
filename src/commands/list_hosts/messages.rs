use actix::Message;
use iotics_grpc_client::search::SearchResponsePayload;

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
    pub version: String,
    pub url: String,
    pub twins_count: Option<usize>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HostEmptyResultMessage;
