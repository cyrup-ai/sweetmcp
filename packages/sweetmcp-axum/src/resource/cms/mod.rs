pub mod cms_dao;
pub mod models;

use log::error;
use rpc_router::{HandlerError, HandlerResult};
use tokio_stream::StreamExt;

use crate::types::ListResourcesRequest;

// Handler adapter for resources_list that works with the Router
pub async fn resources_list_handler(
    request: Option<ListResourcesRequest>,
) -> HandlerResult<Vec<crate::types::Resource>> {
    let stream = cms_dao::resources_list(request);
    let mut resources = Vec::new();

    tokio::pin!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(resource) => resources.push(resource),
            Err(e) => {
                error!("Error retrieving resource: {}", e);
                return Err(HandlerError::new(format!(
                    "Failed to retrieve resources: {}",
                    e
                )));
            }
        }
    }

    Ok(resources)
}
