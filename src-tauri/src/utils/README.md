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

**使用示例：**

```rust
use crate::string_enum;

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

---

**使用说明：**

- 使用 `use crate::string_enum;` 导入宏
- 所有工具宏都位于 `src-tauri/src/utils/` 目录下
- 新增工具函数时请同步更新此文档
- 优先使用现有工具，避免重复实现
