pub mod cms;

// Re-export public interface
pub use cms::{
    cms_dao::{find_by_slug, find_by_tags, init_cms_dao, resource_read},
    resources_list_handler,
};

// Define a wrapper function with the proper type for the router
pub async fn resources_list(
    request: Option<crate::types::ListResourcesRequest>,
) -> rpc_router::HandlerResult<crate::types::ListResourcesResult> {
    let resources = resources_list_handler(request).await?;
    Ok(crate::types::ListResourcesResult {
        resources,
        next_cursor: None, // No pagination for now
    })
}
