# 配置文件迁移指南 📋

## 🎯 **问题解决**

如果你遇到这个错误：
```
Failed to parse config: missing field `transport` at line X column Y
```

**不用担心！** 我们添加了完全的向后兼容性支持。

## ✅ **两种格式都支持**

### **旧格式（仍然有效）** ⭐
```json
{
  "mcpServers": {
    "my-server": {
      "command": "node",
      "args": ["server.js"],
      "env": {
        "NODE_ENV": "production"
      }
    }
  }
}
```
**不需要修改，依然可以正常使用！**

### **新格式（推荐）** ⭐
```json
{
  "mcpServers": {
    "child-process-server": {
      "transport": "child-process",
      "command": "node", 
      "args": ["server.js"],
      "env": {
        "NODE_ENV": "production"
      }
    },
    "sse-server": {
      "transport": "sse",
      "url": "http://localhost:12121/sse"
    },
    "sse-with-auth": {
      "transport": "sse", 
      "url": "https://api.example.com/mcp",
      "auth_token": "your-token",
      "headers": {
        "X-API-Key": "your-key"
      }
    }
  }
}
```

## 🔄 **迁移选项**

### **选项1：保持不变** ✨
```bash
# 你的旧配置文件无需修改，直接使用
cargo run --bin mcpcs-client repl
```

### **选项2：逐步迁移** 🚀
保留旧配置，逐个添加新格式的服务器：
```json
{
  "mcpServers": {
    "old-server": {
      "command": "python",
      "args": ["server.py"]
    },
    "new-sse-server": {
      "transport": "sse",
      "url": "http://localhost:12121/sse"
    }
  }
}
```

### **选项3：完全迁移** 🎯
为所有服务器添加`transport`字段：
```json
{
  "mcpServers": {
    "my-server": {
      "transport": "child-process",  // 添加这行
      "command": "node",
      "args": ["server.js"]
    }
  }
}
```

## 📖 **新功能优势**

### **SSE支持** 🌐
```json
{
  "transport": "sse",
  "url": "https://mcp.context7.com/mcp",
  "headers": {
    "CONTEXT7_API_KEY": "YOUR_API_KEY"
  }
}
```

### **Headers认证** 🔐
```json
{
  "transport": "sse",
  "url": "https://api.service.com/mcp", 
  "auth_token": "bearer-token",
  "headers": {
    "X-API-Key": "api-key",
    "User-Agent": "mcpcs-client/1.0"
  }
}
```

## 🚀 **测试配置**

### **验证配置文件**
```bash
# 测试配置加载
cargo run --bin mcpcs-client repl
```

### **检查连接**
```bash
# 在REPL中运行
/list mcp
```

## ❓ **常见问题**

### **Q: 我需要修改现有配置吗？**
**A:** 不需要！旧格式完全兼容。

### **Q: 新功能如何使用？**
**A:** 添加`"transport": "sse"`即可使用SSE和headers功能。

### **Q: 可以混合使用吗？**
**A:** 可以！同一个配置文件中可以混合新旧格式。

### **Q: Context7如何配置？**
**A:** 使用提供的`context7-config.json`模板，替换API key即可。

## 🆘 **需要帮助？**

如果仍有问题：
1. 检查JSON语法是否正确
2. 确认文件编码为UTF-8
3. 查看控制台错误输出的具体行号
4. 参考`sse-test-config.json`示例

**记住：不破坏现有配置是我们的承诺！** ✨
