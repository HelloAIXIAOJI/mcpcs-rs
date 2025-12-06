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
}
