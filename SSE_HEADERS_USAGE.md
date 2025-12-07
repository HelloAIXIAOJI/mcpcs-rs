# SSE Headers Support

MCP-CS-RS现在完全支持SSE连接中的HTTP headers配置，包括身份验证和自定义headers。

## 配置示例

### 1. 基本SSE连接（无headers）
```json
{
  "mcpServers": {
    "basic-sse": {
      "transport": "sse",
      "url": "http://localhost:12121/sse"
    }
  }
}
```

### 2. 带Bearer Token认证的SSE连接
```json
{
  "mcpServers": {
    "auth-sse": {
      "transport": "sse",
      "url": "http://localhost:12121/sse",
      "auth_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    }
  }
}
```

### 3. 带自定义Headers的SSE连接
```json
{
  "mcpServers": {
    "custom-headers-sse": {
      "transport": "sse",
      "url": "http://localhost:12121/sse",
      "headers": {
        "X-API-Key": "your-api-key-here",
        "X-Client-Version": "1.0.0",
        "X-Environment": "production"
      }
    }
  }
}
```

### 4. 完整配置（认证+自定义headers）
```json
{
  "mcpServers": {
    "full-config-sse": {
      "transport": "sse",
      "url": "http://localhost:12121/sse",
      "auth_token": "your-bearer-token",
      "headers": {
        "X-API-Key": "your-api-key",
        "X-Client-Version": "1.0.0",
        "User-Agent": "mcpcs-client/1.0"
      }
    }
  }
}
```

## 命令行使用

### 基本SSE连接
```bash
cargo run --bin mcpcs-client sse --url http://localhost:12121/sse
```

### 启动服务器（SSE模式）
```bash
cargo run --bin mcpcs-server sse --port 12121
```

## Headers详细说明

### 支持的Header类型

1. **认证Headers**
   - `auth_token`: 自动添加为 `Authorization: Bearer {token}`
   - 优先级: 运行时传入 > 配置文件中的token

2. **自定义Headers**
   - 所有在 `headers` 对象中定义的键值对都会被添加到请求中
   - 支持任意HTTP header名称和值

3. **内置Headers**（自动处理）
   - `Accept: text/event-stream` (SSE连接)
   - `Content-Type: application/json` (消息POST)
   - `Last-Event-ID`: 断点续传支持

### 使用场景

- **API认证**: 通过Bearer token或API key认证
- **版本控制**: 通过User-Agent或X-Version header标识客户端版本
- **环境标识**: 区分开发/测试/生产环境
- **调试追踪**: 添加请求ID或调试标识
- **负载均衡**: 通过自定义header进行路由控制

## 实现原理

当配置了 `auth_token` 或 `headers` 时，客户端会：

1. 创建自定义的 `CustomSseClient` 实现
2. 在每个HTTP请求中自动添加配置的headers
3. 支持Bearer token认证
4. 保持与标准SSE客户端的兼容性

这样可以灵活地与各种需要认证或自定义headers的SSE服务器集成。
