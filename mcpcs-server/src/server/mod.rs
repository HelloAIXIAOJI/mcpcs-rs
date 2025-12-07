pub mod state;

use rmcp::{
    ErrorData as McpError, 
    ServerHandler,
    model::*,
    service::RequestContext,
    RoleServer,
    tool, tool_router,
    handler::server::{tool::{ToolRouter, ToolCallContext}, wrapper::Parameters},
};
use schemars::JsonSchema;
use serde::Deserialize;
use rand::Rng;
use state::ServerState;

#[derive(Clone)]
pub struct McpServer {
    state: ServerState,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RandomArgParams {
    /// Minimum value (inclusive)
    pub min: i32,
    /// Maximum value (inclusive)
    pub max: i32,
}

#[tool_router]
impl McpServer {
    pub fn new(state: ServerState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Generate a random number between 1 and 1000")]
    async fn random(&self) -> Result<CallToolResult, McpError> {
        let num: i32 = rand::thread_rng().gen_range(1..=1000);
        Ok(CallToolResult::success(vec![Content::text(num.to_string())]))
    }

    #[tool(description = "Generate a random number within specified range")]
    async fn random_arg(&self, params: Parameters<RandomArgParams>) -> Result<CallToolResult, McpError> {
        let params = params.0;
        if params.min > params.max {
            return Ok(CallToolResult::error(vec![Content::text(
                "Error: min must be less than or equal to max".to_string()
            )]));
        }
        let num: i32 = rand::thread_rng().gen_range(params.min..=params.max);
        Ok(CallToolResult::success(vec![Content::text(num.to_string())]))
    }

    #[tool(description = "Get the SSH server port")]
    async fn sshp(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            self.state.ssh_port.to_string()
        )]))
    }

    #[tool(description = "Get the latest /say content from REPL")]
    async fn getsay(&self) -> Result<CallToolResult, McpError> {
        let content = self.state.get_say().await;
        if content.is_empty() {
            Ok(CallToolResult::success(vec![Content::text("(no content set)")]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(content)]))
        }
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "mcpcs-server".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            },
            instructions: Some("MCP Server with TCP REPL".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources = match self.state.resources.list_resources() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error loading resources: {}", e);
                vec![]
            }
        };

        let mcp_resources = resources.into_iter().map(|entry| {
            let mime_type = self.detect_mime_type_from_entry(&entry);
            Resource {
                raw: RawResource {
                    uri: entry.uri,
                    name: entry.name,
                    title: None,
                    description: entry.description,
                    mime_type,
                    size: None,
                    icons: None,
                },
                annotations: None,
            }
        }).collect();

        Ok(ListResourcesResult {
            resources: mcp_resources,
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match self.state.resources.get_resource(&request.uri) {
            Ok(Some(entry)) => {
                let content = match &entry.content {
                    crate::resources::ResourceContent::Text { content } => {
                        ResourceContents::TextResourceContents {
                            uri: entry.uri.clone(),
                            mime_type: Some("text/plain".to_string()),
                            text: content.clone(),
                            meta: None,
                        }
                    }
                    crate::resources::ResourceContent::File { path } => {
                        match std::fs::read_to_string(path) {
                            Ok(text) => {
                                let mime_type = self.detect_file_mime_type(std::path::Path::new(path));
                                ResourceContents::TextResourceContents {
                                    uri: entry.uri.clone(),
                                    mime_type: Some(mime_type),
                                    text,
                                    meta: None,
                                }
                            }
                            Err(e) => {
                                return Err(McpError::invalid_params(
                                    format!("Failed to read file {}: {}", path, e),
                                    None,
                                ));
                            }
                        }
                    }
                };

                Ok(ReadResourceResult {
                    contents: vec![content],
                })
            }
            Ok(None) => Err(McpError::invalid_params(
                format!("Resource not found: {}", request.uri),
                None,
            )),
            Err(e) => Err(McpError::internal_error(
                format!("Error accessing resource: {}", e),
                None,
            )),
        }
    }
}

impl McpServer {
    fn detect_mime_type_from_entry(&self, entry: &crate::resources::ResourceEntry) -> Option<String> {
        match &entry.content {
            crate::resources::ResourceContent::Text { .. } => Some("text/plain".to_string()),
            crate::resources::ResourceContent::File { path } => {
                Some(self.detect_file_mime_type(std::path::Path::new(path)))
            }
        }
    }

    fn detect_file_mime_type(&self, path: &std::path::Path) -> String {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" => "text/plain".to_string(),
            "md" => "text/markdown".to_string(),
            "json" => "application/json".to_string(),
            "xml" => "application/xml".to_string(),
            "html" => "text/html".to_string(),
            "css" => "text/css".to_string(),
            "js" => "application/javascript".to_string(),
            "rs" => "text/x-rust".to_string(),
            "py" => "text/x-python".to_string(),
            "java" => "text/x-java".to_string(),
            "cpp" | "cc" | "cxx" => "text/x-c++".to_string(),
            "c" => "text/x-c".to_string(),
            "h" => "text/x-header".to_string(),
            _ => "text/plain".to_string(),
        }
    }
}
