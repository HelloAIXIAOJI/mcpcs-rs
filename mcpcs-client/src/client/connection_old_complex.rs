use crate::config::McpServerConfig;
use anyhow::Result;
use colored::Colorize;
use rmcp::{
    service::{RunningService, ServiceExt},
    transport::{ConfigureCommandExt, TokioChildProcess, SseClientTransport, StreamableHttpClientTransport},
    RoleClient,
};
use std::sync::Arc;
use tokio::process::Command;

use super::ClientManager;

impl ClientManager {
    pub async fn load_from_config(&mut self, config: &crate::config::McpConfig) -> Result<()> {
        self.clients.clear();
        for (name, server_conf) in &config.mcp_servers {
            match self.connect(server_conf).await {
                Ok(client) => {
                    self.clients.insert(name.clone(), Arc::new(client));
                    println!("{} {}", "Connected:".green(), name.cyan());
                }
                Err(e) => {
                    eprintln!("{} '{}': {:#}", "Failed to connect:".red(), name, e);
                }
            }
        }
        Ok(())
    }

    pub async fn connect(&self, config: &McpServerConfig) -> Result<RunningService<RoleClient, ()>> {
        match config {
            McpServerConfig::ChildProcess { command, args, env, .. }
            | McpServerConfig::Legacy { command, args, env } => {
                let cmd = Command::new(command);
                let args = args.clone();
                let env = env.clone();

                let transport = TokioChildProcess::new(cmd.configure(move |c| {
                    c.args(&args);
                    if let Some(e) = &env {
                        c.envs(e);
                    }
                }))?;

                let client = ().serve(transport).await?;
                Ok(client)
            }
            McpServerConfig::Sse { url, auth_token, headers, .. } => {
                use rmcp::transport::sse_client::{SseClientConfig, SseClientTransport};
                
                // 🚀 OpenAI方法：在reqwest客户端层面设置default headers  
                let mut client_builder = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30));
                
                // 如果有自定义headers，设置为default headers
                if let Some(headers) = headers {
                    let mut header_map = reqwest::header::HeaderMap::new();
                    
                    for (name, value) in headers {
                        match (name.parse::<reqwest::header::HeaderName>(), 
                               reqwest::header::HeaderValue::from_str(&value)) {
                            (Ok(header_name), Ok(header_value)) => {
                                header_map.insert(header_name, header_value);
                                eprintln!("🔍 Added SSE header: {} = {}", name, value);
                            }
                            _ => {
                                eprintln!("⚠️ Invalid SSE header: {} = {}", name, value);
                            }
                        }
                    }
                    
                    if !header_map.is_empty() {
                        client_builder = client_builder.default_headers(header_map);
                    }
                }
                
                let http_client = client_builder.build()
                    .map_err(|e| anyhow::anyhow!("Failed to build SSE client: {}", e))?;
                
                // 配置rmcp SSE transport  
                let config = SseClientConfig {
                    sse_endpoint: url.clone().try_into().unwrap(),
                    ..Default::default()
                };
                
                let transport = SseClientTransport::start_with_client(http_client, config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to start SSE transport: {}", e))?;
                
                let client = ().serve(transport).await?;
                Ok(client)
            }
                    use sse_stream::{Sse, Error as SseError};
                    use http::Uri;
                    
                    #[derive(Clone)]
                    struct CustomSseClient {
                        client: reqwest::Client,
                        auth_token: Option<String>,
                        custom_headers: Option<std::collections::HashMap<String, String>>,
                    }
                    
                    impl CustomSseClient {
                        fn new(auth_token: Option<String>, headers: Option<std::collections::HashMap<String, String>>) -> Self {
                            Self {
                                client: reqwest::Client::new(),
                                auth_token,
                                custom_headers: headers,
                            }
                        }
                        
                        fn add_custom_headers(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
                            if let Some(headers) = &self.custom_headers {
                                for (key, value) in headers {
                                    request = request.header(key, value);
                                }
                            }
                            request
                        }
                    }
                    
                    impl SseClient for CustomSseClient {
                        type Error = reqwest::Error;
                        
                        async fn post_message(
                            &self,
                            uri: Uri,
                            message: ClientJsonRpcMessage,
                            mut auth_token: Option<String>,
                        ) -> Result<(), rmcp::transport::sse_client::SseTransportError<Self::Error>> {
                            // Use provided auth_token or fallback to instance token
                            if auth_token.is_none() {
                                auth_token = self.auth_token.clone();
                            }
                            
                            let mut request = self.client.post(uri.to_string());
                            request = self.add_custom_headers(request);
                            
                            if let Some(token) = auth_token {
                                request = request.bearer_auth(token);
                            }
                            
                            let response = request.json(&message).send().await
                                .map_err(rmcp::transport::sse_client::SseTransportError::Client)?;
                            response.error_for_status()
                                .map_err(rmcp::transport::sse_client::SseTransportError::Client)?;
                            Ok(())
                        }
                        
                        async fn get_stream(
                            &self,
                            uri: Uri,
                            _last_event_id: Option<String>,
                            mut auth_token: Option<String>,
                        ) -> Result<BoxStream<'static, Result<Sse, SseError>>, rmcp::transport::sse_client::SseTransportError<Self::Error>> {
                            use futures::StreamExt;
                            use sse_stream::SseStream;
                            
                            // Use provided auth_token or fallback to instance token
                            if auth_token.is_none() {
                                auth_token = self.auth_token.clone();
                            }
                            
                            let mut request = self.client.get(uri.to_string());
                            request = request.header("Accept", "text/event-stream");
                            request = self.add_custom_headers(request);
                            
                            if let Some(token) = auth_token {
                                request = request.bearer_auth(token);
                            }
                            
                            let response = request.send().await
                                .map_err(rmcp::transport::sse_client::SseTransportError::Client)?;
                            let response = response.error_for_status()
                                .map_err(rmcp::transport::sse_client::SseTransportError::Client)?;
                            let event_stream = SseStream::from_byte_stream(response.bytes_stream()).boxed();
                            Ok(event_stream)
                        }
                    }
                    
                    let custom_client = CustomSseClient::new(auth_token.clone(), headers.clone());
                    let config = SseClientConfig {
                        sse_endpoint: url.clone().try_into().unwrap(),
                        ..Default::default()
                    };
                    
                    let transport = SseClientTransport::start_with_client(custom_client, config).await
                        .map_err(|e| anyhow::anyhow!("Failed to start custom SSE transport: {}", e))?;
                    
                    let client = ().serve(transport).await?;
                    Ok(client)
                } else {
                    // Use default reqwest client for simple cases
                    let transport = SseClientTransport::start(url.as_str()).await
                        .map_err(|e| anyhow::anyhow!("Failed to start SSE transport: {}", e))?;

                    let client = ().serve(transport).await?;
                    Ok(client)
                }
            }
            McpServerConfig::Http { url, auth_token, headers, stateless, .. } => {
                use rmcp::transport::streamable_http_client::{StreamableHttpClientTransportConfig, StreamableHttpClientTransport};
                
                // 🚀 OpenAI方法：在reqwest客户端层面设置default headers
                let mut client_builder = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30));
                
                // 如果有自定义headers，设置为default headers
                if let Some(headers) = headers {
                    let mut header_map = reqwest::header::HeaderMap::new();
                    
                    for (name, value) in headers {
                        match (name.parse::<reqwest::header::HeaderName>(), 
                               reqwest::header::HeaderValue::from_str(&value)) {
                            (Ok(header_name), Ok(header_value)) => {
                                header_map.insert(header_name, header_value);
                                eprintln!("🔍 Added header: {} = {}", name, value);
                            }
                            _ => {
                                eprintln!("⚠️ Invalid header: {} = {}", name, value);
                            }
                        }
                    }
                    
                    if !header_map.is_empty() {
                        client_builder = client_builder.default_headers(header_map);
                    }
                }
                
                let http_client = client_builder.build()
                    .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;
                
                // 配置rmcp transport
                let mut config = StreamableHttpClientTransportConfig {
                    uri: url.clone().into(),
                    allow_stateless: stateless.unwrap_or(true),
                    ..Default::default()
                };
                
                // 如果有auth_token，设置为Authorization header
                if let Some(token) = auth_token {
                    config.auth_header = Some(format!("Bearer {}", token));
                }
                
                // 使用配置好headers的客户端
                let transport = StreamableHttpClientTransport::with_client(http_client, config);
                let client = ().serve(transport).await?;
                Ok(client)
            }
        }
    }
}
