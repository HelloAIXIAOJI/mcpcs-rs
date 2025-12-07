# 🎯 Prompt 配置指南

## 概述

Prompt 功能提供智能 LLM 对话模板生成，支持参数化模板、自动渲染和结构化消息输出，让 AI 交互更加专业和高效。

## 配置文件位置
```
mcpcs-server(.exe)  
prompt.json             ← 提示词配置文件
```

## 配置文件格式

```json
{
  "prompts": [
    {
      "name": "code_review",
      "title": "AI代码审查",
      "description": "分析代码质量并提供改进建议", 
      "arguments": [
        {
          "name": "code",
          "description": "要审查的代码",
          "required": true,
          "type": "string"
        },
        {
          "name": "language", 
          "description": "编程语言 (可选)",
          "required": false,
          "type": "string"
        }
      ],
      "template": "请审查以下 {{language}} 代码并提供改进建议：\n\n{{code}}\n\n重点关注：\n- 代码质量和可读性\n- 性能优化\n- 最佳实践\n- 潜在问题"
    }
  ]
}
```

## 字段详解

### 提示词条目 (PromptEntry)
| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | String | ✅ | 提示词唯一标识符 |
| `title` | String | ❌ | 提示词显示标题 |
| `description` | String | ❌ | 提示词功能描述 |
| `arguments` | Array | ✅ | 参数定义列表 |
| `template` | String | ✅ | 模板字符串 (支持 {{参数}} 替换) |

### 参数定义 (PromptArgument)
| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | String | ✅ | 参数名称 |
| `description` | String | ❌ | 参数说明 |
| `required` | Boolean | ✅ | 是否为必需参数 |
| `type` | String | ✅ | 参数类型："string", "number", "boolean" |

## 模板语法

### 基本替换
使用 `{{参数名}}` 进行占位符替换：

```json
{
  "template": "将以下 {{language}} 代码从 {{from_style}} 风格转换为 {{to_style}} 风格：\n\n{{code}}"
}
```

### 模板示例
```json
{
  "name": "translate_text",
  "template": "请将以下{{source_lang}}文本翻译为{{target_lang}}:\n\n{{text}}\n\n翻译要求:\n- 保持原意准确\n- 语言地道自然\n- 保持{{tone}}语调"
}
```

### 多行模板
```json
{
  "template": "# 代码审查报告\n\n## 代码内容\n```{{language}}\n{{code}}\n```\n\n## 分析要点\n{{focus}}\n\n## 详细评估\n请从以下角度进行分析..."
}
```

## 内置提示词模板

### 1. 代码审查助手
```json
{
  "name": "code_review",
  "title": "AI代码审查助手",
  "description": "分析代码质量并提供专业的改进建议",
  "arguments": [
    {
      "name": "code",
      "description": "要审查的代码片段",
      "required": true,
      "type": "string"
    },
    {
      "name": "language",
      "description": "编程语言(如: Python, Rust, JavaScript)",
      "required": false,
      "type": "string"
    },
    {
      "name": "focus",
      "description": "审查重点(如: security, performance, readability)",
      "required": false,
      "type": "string"
    }
  ],
  "template": "请作为资深{{language}}开发工程师，审查以下代码并提供改进建议：\n\n```{{language}}\n{{code}}\n```\n\n审查重点：{{focus}}\n\n请从以下方面进行分析：\n- **代码质量**: 可读性、maintainability\n- **性能优化**: 算法效率、资源使用\n- **最佳实践**: 编码规范、设计模式\n- **安全性**: 潜在的安全风险\n- **错误处理**: 异常情况的处理\n\n请提供具体的修改建议和解释说明。"
}
```

### 2. 代码解释专家
```json
{
  "name": "explain_code",
  "title": "代码解释专家",
  "description": "详细解释代码的工作原理和逻辑流程",
  "arguments": [
    {
      "name": "code",
      "description": "需要解释的代码",
      "required": true,
      "type": "string"
    },
    {
      "name": "level",
      "description": "解释深度: beginner, intermediate, advanced",
      "required": false,
      "type": "string"
    }
  ],
  "template": "请详细解释以下代码的工作原理(面向{{level}}级别)：\n\n```\n{{code}}\n```\n\n请按照以下结构进行解释：\n\n## 代码概述\n简要说明这段代码的主要功能\n\n## 逐行分析\n详细解释每一行或代码块的作用\n\n## 执行流程\n描述代码的执行顺序和逻辑流向\n\n## 关键概念\n解释涉及的重要编程概念或设计模式\n\n## 使用场景\n说明这种代码结构的适用场景"
}
```

### 3. 智能翻译助手
```json
{
  "name": "translate",
  "title": "专业翻译助手",
  "description": "在不同语言之间进行高质量翻译",
  "arguments": [
    {
      "name": "text",
      "description": "要翻译的文本内容",
      "required": true,
      "type": "string"
    },
    {
      "name": "from",
      "description": "源语言(如: zh-CN, en-US, ja-JP)",
      "required": true,
      "type": "string"
    },
    {
      "name": "to",
      "description": "目标语言(如: zh-CN, en-US, ja-JP)",
      "required": true,
      "type": "string"
    },
    {
      "name": "domain",
      "description": "领域类型: general, technical, business, academic",
      "required": false,
      "type": "string"
    }
  ],
  "template": "请将以下{{domain}}领域的{{from}}文本翻译为{{to}}：\n\n**原文：**\n{{text}}\n\n**翻译要求：**\n- 保持原意的准确性\n- 使用地道的目标语言表达\n- 保持专业术语的一致性\n- 适应目标语言的表达习惯\n- 保留原文的语气和风格\n\n**翻译结果：**\n[在此提供翻译]"
}
```

### 4. 调试问题分析师
```json
{
  "name": "debug_helper",
  "title": "调试问题分析师",
  "description": "帮助分析和解决代码中的错误和问题",
  "arguments": [
    {
      "name": "error",
      "description": "错误信息或异常内容",
      "required": true,
      "type": "string"
    },
    {
      "name": "code",
      "description": "相关的代码片段",
      "required": false,
      "type": "string"
    },
    {
      "name": "context",
      "description": "错误发生的上下文环境",
      "required": false,
      "type": "string"
    }
  ],
  "template": "请帮助分析以下编程问题：\n\n**错误信息：**\n{{error}}\n\n**相关代码：**\n```\n{{code}}\n```\n\n**运行环境：**\n{{context}}\n\n请按以下步骤进行分析：\n\n## 1. 错误诊断\n分析错误的根本原因\n\n## 2. 问题定位\n指出具体的问题所在\n\n## 3. 解决方案\n提供详细的修复步骤\n\n## 4. 预防措施\n建议如何避免类似问题\n\n## 5. 最佳实践\n相关的编程最佳实践建议"
}
```

### 5. API 设计顾问
```json
{
  "name": "api_design",
  "title": "API设计顾问",
  "description": "设计RESTful API和接口规范",
  "arguments": [
    {
      "name": "purpose",
      "description": "API的主要功能和目标",
      "required": true,
      "type": "string"
    },
    {
      "name": "resources",
      "description": "涉及的主要资源类型",
      "required": false,
      "type": "string"
    },
    {
      "name": "style",
      "description": "API风格: REST, GraphQL, gRPC",
      "required": false,
      "type": "string"
    }
  ],
  "template": "请设计一个{{style}}风格的API来满足以下需求：\n\n**功能目标：**\n{{purpose}}\n\n**涉及资源：**\n{{resources}}\n\n请提供完整的API设计方案：\n\n## 1. API概述\n- 功能描述\n- 设计原则\n- 技术选型\n\n## 2. 接口规范\n- 端点定义\n- HTTP方法\n- 请求/响应格式\n- 状态码设计\n\n## 3. 数据模型\n- 资源模型定义\n- 字段说明\n- 关系设计\n\n## 4. 安全考虑\n- 认证授权\n- 数据验证\n- 错误处理\n\n## 5. 文档示例\n- 请求示例\n- 响应示例\n- 错误示例"
}
```

## REPL 命令

### 提示词管理
```bash
# 查看提示词
/prompt list                                # 列出所有提示词
/prompt show <name>                         # 显示提示词详情

# 测试提示词
/prompt test <name> [key=value...]          # 测试提示词渲染

# 管理配置
/prompt rm <name>                           # 删除提示词
/prompt reload                              # 重新加载配置
```

### 使用示例
```bash
# 查看所有提示词
/prompt list

# 查看特定提示词详情
/prompt show code_review

# 测试提示词生成
/prompt test code_review code="def hello(): print('world')" language="Python"

# 测试翻译提示词
/prompt test translate text="Hello World" from="en" to="zh"

# 重新加载配置
/prompt reload
```

## 客户端使用

### MCP 客户端命令
```bash
# 发现提示词
/list prompt                                # 列出可用提示词

# 查看提示词信息
/info prompt code_review                    # 查看提示词详情  

# 使用提示词
/use prompt code_review code="your_code_here" language="Python"
/use prompt translate text="Hello" from="en" to="zh" domain="technical"
```

### 参数传递格式
```bash
# 基本格式
/use prompt name key=value

# 带引号的值
/use prompt code_review code="def func(): return 42" language="Python"

# 多个参数
/use prompt api_design purpose="用户管理" resources="User, Role" style="REST"
```

## 高级模板技巧

### 1. 条件内容
```json
{
  "template": "分析以下{{#if language}}{{language}}{{else}}代码{{/if}}：\n\n{{code}}\n\n{{#if focus}}重点关注: {{focus}}{{/if}}"
}
```

### 2. 列表处理
```json
{
  "template": "请审查以下文件：\n{{#each files}}\n- {{this}}\n{{/each}}\n\n提供整体评估。"
}
```

### 3. 格式化输出
```json
{
  "template": "# {{title}}\n\n**类型**: {{type}}\n**优先级**: {{priority}}\n\n## 描述\n{{description}}\n\n## 行动项\n{{actions}}"
}
```

## 最佳实践

### 1. 命名规范
- **使用小写字母和下划线**: `code_review`, `translate_text`
- **描述功能**: `debug_helper`, `api_design`
- **保持简洁**: 避免过长的名称

### 2. 参数设计
- **必需参数**: 核心功能必需的输入
- **可选参数**: 增强功能的配置项
- **类型明确**: 正确设置参数类型
- **描述清晰**: 提供有用的参数说明

### 3. 模板编写
- **结构清晰**: 使用标题和分段
- **指令明确**: 给 AI 清晰的任务描述
- **格式统一**: 保持一致的输出格式
- **例子丰富**: 在模板中包含示例

### 4. 测试验证
```bash
# 测试基本功能
/prompt test code_review code="print('hello')" language="Python"

# 测试边界条件
/prompt test translate text="" from="en" to="zh"

# 测试参数组合
/prompt test api_design purpose="用户系统" style="GraphQL"
```

### 5. 版本管理
```json
{
  "name": "code_review_v2",
  "title": "代码审查 v2.0",
  "description": "增强版代码审查，支持多语言和自定义规则",
  "arguments": [...]
}
```

## 故障排除

### 常见问题

#### 1. 模板语法错误
```
错误: Template rendering failed: Missing argument 'code'
```
**解决方案：**
- 检查所有 `{{参数名}}` 是否在 arguments 中定义
- 验证必需参数是否正确传递

#### 2. 参数类型不匹配
```
错误: Invalid argument type: expected string, got number
```
**解决方案：**
- 确保参数类型与定义一致
- 检查客户端传递的参数格式

#### 3. 配置文件格式错误
```
错误: Error parsing JSON: invalid escape sequence
```
**解决方案：**
- 使用正确的 JSON 转义序列
- 多行文本使用 `\n` 表示换行

#### 4. 模板渲染失败
```
错误: Template contains undefined variables
```
**解决方案：**
- 检查模板中的占位符是否都有对应参数
- 为可选参数提供默认值处理

## 性能优化

### 1. 模板缓存
- 服务器自动缓存解析后的模板
- 配置变更后自动重新加载

### 2. 参数验证
- 在渲染前验证所有必需参数
- 提供清晰的错误信息

### 3. 内容长度
- 合理控制模板长度
- 避免过于复杂的逻辑

## 扩展功能

### 1. 自定义函数
```json
{
  "template": "当前时间: {{now()}}\n处理文件: {{filename(code)}}\n\n{{code}}"
}
```

### 2. 多语言支持
```json
{
  "name": "greeting",
  "arguments": [
    {"name": "lang", "description": "语言代码", "required": true, "type": "string"}
  ],
  "template": "{{#switch lang}}\n{{#case 'zh'}}你好！{{/case}}\n{{#case 'en'}}Hello!{{/case}}\n{{#default}}Hello!{{/default}}\n{{/switch}}"
}
```

### 3. 模板继承
```json
{
  "name": "base_analysis",
  "template": "# {{title}}\n\n## 分析对象\n{{content}}\n\n## 详细分析\n{{analysis_content}}"
}
```

这样就可以创建强大而灵活的 AI 对话模板系统了！
