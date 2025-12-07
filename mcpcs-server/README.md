# MCPCS Server Documentation

MCPCS Server 是一个支持 MCP (Model Context Protocol) 的服务器，提供 SSH REPL 接口和丰富的工具、资源、提示词功能。

## 快速开始

```bash
# 启动服务器
cargo run --bin mcpcs-server

# 连接到 SSH REPL
ssh localhost -p <显示的端口号>
```

服务器启动时会随机选择端口 (12312-12412)，并在控制台显示 SSH 端口信息。

## 功能特性

- **🔧 工具 (Tools)**: 4个内置工具 - random, random_arg, sshp, getsay
- **📁 资源 (Resources)**: 支持文本和二进制文件管理
- **🎯 提示词 (Prompts)**: 智能 LLM 对话模板生成
- **🖥️ SSH REPL**: 交互式命令行界面
- **📄 JSON 配置**: 持久化存储和动态重载

## 配置文件

服务器在可执行文件同目录下创建配置文件：

- **resource.json** - 资源配置 ([详细配置指南](resource-config.md))
- **prompt.json** - 提示词配置 ([详细配置指南](prompt-config.md))
- **example_resource.json** - 资源配置示例
- **example_prompt.json** - 提示词配置示例

## SSH REPL 命令

### 基本命令
```bash
/say <content>          # 设置 say 内容
/help                   # 显示帮助信息  
/exit                   # 退出连接
```

### 资源管理
```bash
/resource list                          # 列出所有资源
/resource add text <uri> <content>      # 添加文本资源
/resource add file <uri> <path>         # 添加文件资源
/resource rm <uri>                      # 删除资源
/resource reload                        # 重新加载配置
```

### 提示词管理
```bash
/prompt list                            # 列出所有提示词
/prompt show <name>                     # 显示提示词详情
/prompt test <name> [args...]           # 测试提示词渲染
/prompt rm <name>                       # 删除提示词
/prompt reload                          # 重新加载配置
```

## MCP 客户端使用

### 工具调用
```bash
# 内置工具
random                  # 生成随机数 (1-1000)
random_arg min=10 max=50# 生成指定范围随机数
sshp                    # 获取 SSH 端口号
getsay                  # 获取 say 内容
```

### 资源访问
```bash
/list resource          # 列出可用资源
/read resource <uri>    # 读取资源内容
/info resource <uri>    # 查看资源信息
/down resource <uri> <file> # 下载资源到文件
```

### 提示词使用
```bash
/list prompt            # 列出可用提示词
/info prompt <name>     # 查看提示词详情
/use prompt <name> [key=value...] # 生成提示词消息
```

## 配置示例

### 基础资源配置 (resource.json)
```json
{
  "resources": [
    {
      "uri": "memory://welcome/greeting",
      "name": "欢迎信息", 
      "description": "服务器欢迎和基本信息",
      "resource_type": "Text",
      "content": "欢迎使用 MCPCS 服务器！"
    }
  ]
}
```

### 基础提示词配置 (prompt.json)
```json
{
  "prompts": [
    {
      "name": "code_review",
      "title": "代码审查助手",
      "description": "分析代码并提供改进建议",
      "arguments": [
        {
          "name": "code",
          "description": "要审查的代码",
          "required": true,
          "type": "string"
        }
      ],
      "template": "请审查以下代码并提供改进建议：\n\n{{code}}"
    }
  ]
}
```

## 详细配置指南

- **📁 [Resource 配置详解](resource-config.md)** - 文本和文件资源管理完整指南
- **🎯 [Prompt 配置详解](prompt-config.md)** - AI 对话模板创建和使用指南

## 服务器管理

### 日志监控
服务器日志包含：
- SSH 连接状态
- MCP 协议交互
- 配置文件变更
- 错误和警告信息

### 动态重载
```bash
/resource reload    # 重载资源配置
/prompt reload      # 重载提示词配置
```

### 端口管理
- 自动选择端口范围: 12312-12412
- 启动时显示实际端口号
- 支持多客户端并发连接

## 注意事项

1. **文件权限** - 确保服务器有权限读取配置的文件路径
2. **端口访问** - 确保 SSH 端口没有被防火墙阻止  
3. **配置备份** - 重要配置更改前请备份 JSON 文件
4. **资源大小** - 避免添加过大的文件资源影响性能
5. **模板验证** - 提示词模板中的 {{参数}} 必须在 arguments 中定义

## 故障排除

**常见问题：**
- 配置文件格式错误 → 检查 JSON 语法
- 文件路径无法访问 → 验证路径和权限
- SSH 连接失败 → 检查端口和防火墙设置

**获取帮助：**
- 使用 `/help` 命令查看可用命令
- 查看控制台日志获取详细错误信息
- 参考示例配置文件了解正确格式

---

**更多详细信息请参考具体的配置指南文档！**
