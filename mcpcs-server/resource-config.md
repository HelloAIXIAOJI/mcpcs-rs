# 📁 Resource 配置指南

## 概述

Resource 功能允许服务器管理和暴露文本内容和文件引用，客户端可以通过 MCP 协议访问这些资源。

## 配置文件位置
```
mcpcs-server(.exe)
resource.json          ← 资源配置文件
```

## 配置文件格式

```json
{
  "resources": [
    {
      "uri": "file://docs/README.md",
      "name": "项目文档",
      "description": "项目的主要文档文件",
      "resource_type": "Text",
      "content": "# 项目标题\n\n这是项目的主要文档..."
    },
    {
      "uri": "file://assets/logo.png", 
      "name": "项目Logo",
      "description": "公司品牌Logo图片",
      "resource_type": "File",
      "file_path": "C:/Projects/assets/logo.png"
    }
  ]
}
```

## 字段详解

### 资源条目 (ResourceEntry)
| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `uri` | String | ✅ | 资源的唯一标识符 (URI 格式) |
| `name` | String | ✅ | 资源的显示名称 |
| `description` | String | ❌ | 资源的详细描述 |
| `resource_type` | String | ✅ | 资源类型："Text" 或 "File" |
| `content` | String | 条件* | 文本内容 (仅当 resource_type="Text") |
| `file_path` | String | 条件* | 文件路径 (仅当 resource_type="File") |

*条件必需：根据 `resource_type` 决定

## 资源类型

### 1. 文本资源 (Text)
直接在配置中存储文本内容：

```json
{
  "uri": "memory://notes/meeting-2024-12-07",
  "name": "会议记录",
  "description": "2024年12月7日产品会议记录",
  "resource_type": "Text",
  "content": "# 产品会议记录\n\n## 参会人员\n- 张三 (产品经理)\n- 李四 (技术负责人)\n\n## 讨论议题\n1. 新功能开发计划\n2. 技术架构优化"
}
```

**适用场景：**
- 会议记录、笔记
- 配置模板
- 文档片段
- 临时内容

### 2. 文件资源 (File)
引用本地文件系统中的文件：

```json
{
  "uri": "file://configs/database.yml",
  "name": "数据库配置", 
  "description": "生产环境数据库配置文件",
  "resource_type": "File",
  "file_path": "/opt/app/configs/database.yml"
}
```

**适用场景：**
- 配置文件
- 代码示例
- 文档文件
- 日志文件

## URI 命名约定

### 推荐的 URI 格式

#### 文件引用
- **文档**: `file://docs/filename.ext`
- **配置**: `file://configs/filename.ext`  
- **代码**: `file://src/path/filename.ext`
- **日志**: `file://logs/filename.ext`

#### 内存内容
- **笔记**: `memory://notes/category/name`
- **想法**: `memory://ideas/project-name`
- **会议**: `memory://meetings/2024-12-07`

#### 临时内容
- **草稿**: `temp://drafts/document-name`
- **缓存**: `temp://cache/data-type`

### URI 最佳实践
1. **使用描述性路径** - `memory://projects/mcp-server/roadmap`
2. **保持层次结构** - `file://docs/api/v1/endpoints.md`
3. **避免特殊字符** - 使用 `-` 而不是空格
4. **包含日期信息** - `memory://reports/2024/q4-summary`

## 配置示例

### 完整示例
```json
{
  "resources": [
    {
      "uri": "memory://welcome/intro",
      "name": "服务器介绍",
      "description": "MCPCS 服务器功能介绍",
      "resource_type": "Text",
      "content": "🎉 欢迎使用 MCPCS 服务器！\n\n功能特性：\n- MCP 协议支持\n- SSH REPL 接口\n- 工具和资源管理\n- 提示词模板"
    },
    {
      "uri": "file://docs/api-spec.yaml",
      "name": "API 规范",
      "description": "OpenAPI 3.0 规范文档",
      "resource_type": "File",
      "file_path": "/project/docs/openapi.yaml"
    },
    {
      "uri": "memory://notes/architecture",
      "name": "系统架构笔记",
      "description": "技术架构设计要点",
      "resource_type": "Text",
      "content": "# 系统架构\n\n## 核心组件\n1. MCP 服务器\n2. SSH REPL\n3. 资源管理器\n4. 提示词引擎\n\n## 设计原则\n- 模块化设计\n- 异步处理\n- 配置驱动"
    },
    {
      "uri": "file://examples/hello-world.py",
      "name": "Python 示例",
      "description": "简单的 Hello World 程序",
      "resource_type": "File", 
      "file_path": "./examples/hello.py"
    },
    {
      "uri": "temp://scratch/todo",
      "name": "待办事项",
      "description": "临时待办清单",
      "resource_type": "Text",
      "content": "# TODO\n\n- [ ] 完善文档\n- [ ] 添加单元测试\n- [ ] 性能优化\n- [ ] 安全审计"
    }
  ]
}
```

## REPL 命令

### 资源管理命令
```bash
# 查看资源
/resource list                              # 列出所有资源
/resource info <uri>                        # 显示资源详情

# 添加资源
/resource add text <uri> <content>          # 添加文本资源  
/resource add file <uri> <file_path>        # 添加文件资源

# 管理资源
/resource rm <uri>                          # 删除资源
/resource reload                            # 重新加载配置
```

### 使用示例
```bash
# 添加文本资源
/resource add text memory://notes/idea "新功能：AI 代码生成"

# 添加文件资源
/resource add file file://configs/nginx.conf /etc/nginx/nginx.conf

# 查看所有资源
/resource list

# 删除资源
/resource rm temp://old-note

# 重新加载配置文件
/resource reload
```

## 客户端访问

客户端通过 MCP 协议访问资源：

```bash
# 客户端 REPL 命令
/list resource                              # 列出可用资源
/read resource memory://notes/meeting       # 读取资源内容
/info resource file://docs/api.md           # 查看资源信息
/down resource file://logo.png ./logo.png   # 下载资源
```

## 最佳实践

### 1. 组织结构
```json
{
  "resources": [
    // 系统文档
    {"uri": "file://docs/readme.md", ...},
    {"uri": "file://docs/api-reference.md", ...},
    
    // 配置文件
    {"uri": "file://configs/app.toml", ...},
    {"uri": "file://configs/database.yml", ...},
    
    // 代码示例
    {"uri": "file://examples/basic.py", ...},
    {"uri": "file://examples/advanced.rs", ...},
    
    // 工作笔记
    {"uri": "memory://notes/meeting-2024-12-07", ...},
    {"uri": "memory://ideas/product-features", ...}
  ]
}
```

### 2. 命名规范
- **使用小写字母和连字符**
- **包含日期信息** (对于时间相关内容)
- **保持路径结构清晰**
- **避免过长的名称**

### 3. 内容管理
- **文本资源** - 适合短小的配置和笔记
- **文件资源** - 适合较大的文档和代码文件
- **定期清理** - 删除过时的临时资源
- **备份重要** - 重要资源应有本地文件备份

### 4. 安全注意事项
- **文件权限** - 确保服务器有权限读取文件路径
- **敏感信息** - 避免在资源中存储密码、密钥等敏感信息
- **路径验证** - 使用绝对路径，避免路径遍历攻击
- **内容大小** - 避免添加过大的文件影响性能

## 故障排除

### 常见问题

#### 1. 文件无法访问
```
错误: Failed to read file: /path/to/file.txt
```
**解决方案：**
- 检查文件是否存在
- 验证服务器进程的文件权限
- 使用绝对路径

#### 2. 配置格式错误
```
错误: Error parsing JSON: expected ','
```
**解决方案：**
- 验证 JSON 格式正确性
- 检查字段名称和类型
- 使用 JSON 验证工具

#### 3. URI 重复
```
错误: Resource with URI 'file://docs/readme.md' already exists
```
**解决方案：**
- 确保每个 URI 唯一
- 删除重复的资源条目
- 使用不同的 URI 路径

## 高级用法

### 1. 动态路径
```json
{
  "uri": "file://logs/app.log",
  "name": "应用日志",
  "description": "当前应用程序日志文件",
  "resource_type": "File",
  "file_path": "/var/log/app/$(date +%Y%m%d).log"
}
```

### 2. 多环境配置
```json
{
  "resources": [
    {
      "uri": "file://configs/dev.toml",
      "name": "开发环境配置",
      "resource_type": "File",
      "file_path": "./config/development.toml"
    },
    {
      "uri": "file://configs/prod.toml", 
      "name": "生产环境配置",
      "resource_type": "File",
      "file_path": "./config/production.toml"
    }
  ]
}
```

### 3. 版本控制
```json
{
  "uri": "memory://docs/changelog-v1.2.0",
  "name": "更新日志 v1.2.0",
  "description": "版本 1.2.0 的变更记录",
  "resource_type": "Text",
  "content": "# v1.2.0 更新日志\n\n## 新增功能\n- 支持文件资源\n- 改进错误处理\n\n## 修复问题\n- 修复内存泄漏\n- 优化性能"
}
```

这样就可以高效管理和共享各种类型的资源内容了！
