# 工具函数索引

本文档列出项目中所有可用的 Rust 工具函数和宏，供 AI 助手参考使用。

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
