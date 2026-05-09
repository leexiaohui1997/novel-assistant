# 工具函数索引

本文档列出项目中所有可用的 Rust 工具函数和宏，供 AI 助手参考使用。

## 分页工具 (pagination.rs)

### PaginationParams

分页查询参数结构体
文件：`src-tauri/src/utils/pagination.rs`

**字段：**

- `page: i64` - 页码（从 1 开始）
- `page_size: i64` - 每页数量

**使用示例：**

```rust
use crate::utils::pagination::PaginationParams;

let params = PaginationParams {
    page: 1,
    page_size: 10,
};
```

---

### PaginatedResult<T>

分页响应结果泛型结构体
文件：`src-tauri/src/utils/pagination.rs`

**字段：**

- `data: Vec<T>` - 数据列表
- `total: i64` - 总数量

**使用示例：**

```rust
use crate::utils::pagination::PaginatedResult;

let result: PaginatedResult<User> = PaginatedResult {
    data: vec![user1, user2],
    total: 100,
};
```

---

### query_with_pagination

通用分页查询函数，支持任意类型的数据库查询
文件：`src-tauri/src/utils/pagination.rs`

**参数：**

- `pool: &SqlitePool` - 数据库连接池
- `params: &PaginationParams` - 分页参数
- `count_sql: &str` - 查询总数的 SQL 语句
- `data_sql: &str` - 查询数据的 SQL 语句（必须包含 LIMIT 和 OFFSET 占位符）

**返回值：**

- `Result<PaginatedResult<T>, DbError>` - 分页结果

**使用示例：**

```rust
use crate::utils::pagination::{query_with_pagination, PaginationParams};

async fn find_users(pool: &SqlitePool, page: i64, page_size: i64) -> Result<PaginatedResult<User>, DbError> {
    let params = PaginationParams { page, page_size };

    query_with_pagination::<User>(
        pool,
        &params,
        "SELECT COUNT(*) FROM users",
        "SELECT * FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
    ).await
}
```

---

## 宏工具 (macros.rs)

### string_enum!

为枚举自动生成字符串转换相关的实现（as_str、Display、FromStr）
文件：`src-tauri/src/utils/macros.rs`

**两种模式：**

- **默认模式**：生成 `as_str` / `Display` / `FromStr`，字面量 = 成员名原样（如 `Active → "Active"`）。**不**生成 serde 实现，serde 由使用方自行 `#[derive]` 并用 `#[serde(rename_all = ...)]` 控制。
- **`lower_case` 模式**：在宏体首行写 `lower_case;`，统一按**全小写成员名**生成 `as_str` / `Display` / `FromStr` / `Serialize` / `Deserialize` 五项。适用于要与外部契约（如大模型 API 的 `role` 字段 `"system"` / `"user"` / `"assistant"`）严格对齐的枚举。**该模式下禁止再自行 `#[derive(Serialize, Deserialize)]`**，否则会与宏生成的 `impl` 冲突。

**使用示例（默认模式）：**

```rust
use crate::string_enum;

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Status {
        Active,
        Inactive,
    }
}

// 自动生成的方法
let status = Status::Active;
println!("{}", status);           // 输出: Active
println!("{}", status.as_str());  // 输出: Active
let parsed: Status = "Active".parse().unwrap();
```

**使用示例（`lower_case` 模式）：**

```rust
use crate::string_enum;

string_enum! {
    lower_case;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MessageType {
        System,
        User,
        Assistant,
    }
}

// 字面量全部小写：Display / FromStr / serde 统一口径
let mt = MessageType::Assistant;
assert_eq!(mt.as_str(), "assistant");
assert_eq!(mt.to_string(), "assistant");
let parsed: MessageType = "user".parse().unwrap();
let json = serde_json::to_string(&MessageType::System).unwrap();  // "\"system\""
```

---

## Markdown 工具 (markdown.rs)

### walk_code_blocks

基于 `markdown-rs` 解析 Markdown AST，以回调方式遍历所有围栏代码块（`fenced code block`），并按调用方的筛选与处理闭包执行动作。
文件：`src-tauri/src/utils/markdown.rs`

**参数：**

- `content: &str` - Markdown 源文本
- `label_filter: impl Fn(&str, &str) -> bool` - 筛选闭包，入参依次为 `label`（`lang` 字段）、`info`（`meta` 字段），返回 `true` 时进入处理阶段
- `content_handle: impl FnMut(&str, &str, &str)` - 处理闭包，入参依次为 `label`、`info`、`content`（代码块正文，不含围栏）

**返回值：**

- `Result<(), MarkdownWalkError>` - `Ok(())` 表示遍历完成；`Err` 表示解析阶段出错（包装为 `MarkdownWalkError::ParseFailed(String)`）

**语义说明：**

- 代码块无语言标记时，`label` 为 `""`；无 info string 时 `info` 为 `""`
- 会递归进入 list、blockquote 等容器节点内部的代码块
- 不处理行内代码（`InlineCode`）
- `content` 为 `markdown-rs` 的 `value` 字段原始值，不 trim

**使用示例：**

````rust
use crate::utils::markdown::walk_code_blocks;

let md = "```outline\nHello\n```\n```other\nSkip\n```";
let mut hits: Vec<String> = Vec::new();
walk_code_blocks(
    md,
    |label, _info| label == "outline",
    |_label, _info, content| hits.push(content.to_string()),
)
.unwrap();
assert_eq!(hits, vec!["Hello".to_string()]);
````

---

**使用说明：**

- 使用 `use crate::string_enum;` 导入宏
- 所有工具宏都位于 `src-tauri/src/utils/` 目录下
- 新增工具函数时请同步更新此文档
- 优先使用现有工具，避免重复实现

## 子索引

- AI v2 工具函数索引：`src-tauri/src/ai-v2/utils/README.md`（如 `find_recommended_model`）
