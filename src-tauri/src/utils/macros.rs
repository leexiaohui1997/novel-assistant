/// 为枚举生成字符串转换相关的通用实现
///
/// 这个宏提供两种模式，按 **宏体首行是否显式写 `lower_case;`** 区分：
///
/// ## 默认模式（零破坏、与历史行为兼容）
///
/// 生成以下实现（字面量 = 成员名原样，如 `Active → "Active"`）：
///
/// - `as_str(&self) -> &'static str`
/// - `std::fmt::Display`
/// - `std::str::FromStr`
///
/// **不会** 生成 `Serialize` / `Deserialize`，以便使用方自行 `#[derive(Serialize, Deserialize)]`
/// 并通过 `#[serde(rename_all = "...")]` 等属性灵活控制序列化格式。
///
/// ### 使用示例
///
/// ```rust
/// use crate::string_enum;
///
/// string_enum! {
///     #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
///     #[serde(rename_all = "snake_case")]
///     pub enum Status {
///         Active,   // Display / FromStr 使用 "Active"；serde 由 derive + rename_all 决定
///         Inactive,
///     }
/// }
/// ```
///
/// ## `lower_case` 模式
///
/// 在宏体首行写上 `lower_case;`，告知宏 **统一生成全小写字面量**，覆盖：
///
/// - `as_str(&self) -> &'static str`（小写）
/// - `std::fmt::Display`（小写）
/// - `std::str::FromStr`（小写，未知值返回 `Err(format!("无效的 {枚举名} 值: {s}"))`）
/// - `serde::Serialize`（小写）
/// - `serde::Deserialize`（小写）
///
/// 适用于需要与主流大模型 API 的 role 字段（`"system"` / `"user"` / `"assistant"`）等
/// 外部契约严格对齐的枚举。**此模式下使用方禁止再 `#[derive(Serialize, Deserialize)]`**，
/// 否则会与宏生成的 `impl` 冲突。
///
/// ### 使用示例
///
/// ```rust
/// use crate::string_enum;
///
/// string_enum! {
///     lower_case;
///     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
///     pub enum MessageType {
///         System,     // → "system"
///         User,       // → "user"
///         Assistant,  // → "assistant"
///     }
/// }
/// ```
#[macro_export]
macro_rules! string_enum {
    // ========== 分支 1：lower_case 模式 ==========
    (
        lower_case;
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),+
        }

        impl $name {
            /// 将枚举值转换为全小写字符串字面量
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        $name::$variant => $crate::string_enum!(@to_lower $variant),
                    )+
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        v if v == $crate::string_enum!(@to_lower $variant) => Ok($name::$variant),
                    )+
                    _ => Err(format!("无效的 {} 值: {}", stringify!($name), s)),
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = <String as serde::Deserialize>::deserialize(deserializer)?;
                <$name as std::str::FromStr>::from_str(&s).map_err(serde::de::Error::custom)
            }
        }
    };

    // ========== 分支 2：默认模式（保持历史行为不变） ==========
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),+
        }

        impl $name {
            /// 将枚举值转换为成员名字符串（原样大小写）
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        $name::$variant => stringify!($variant),
                    )+
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => Ok($name::$variant),
                    )+
                    _ => Err(format!("无效的 {} 值: {}", stringify!($name), s)),
                }
            }
        }
    };

    // ========== 内部辅助：将成员名 ident 转为全小写的 &'static str ==========
    //
    // 利用 `const { ... }`（Rust 1.79+）内联常量块，在编译期一次性生成小写字面量，
    // 产出的引用拥有 `'static` 生命周期，且零运行时开销。
    (@to_lower $variant:ident) => {
        const {
            const SRC: &str = stringify!($variant);
            const LEN: usize = SRC.len();
            const LOWER_BYTES: [u8; LEN] = {
                let mut out = [0u8; LEN];
                let bytes = SRC.as_bytes();
                let mut i = 0;
                while i < LEN {
                    let b = bytes[i];
                    out[i] = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                    i += 1;
                }
                out
            };
            // SAFETY：仅对 ASCII 字母做 +32 位操作，结果仍是合法 UTF-8。
            // 枚举成员名在 Rust 中必须是合法标识符，只会包含 ASCII 字母 / 数字 / 下划线。
            unsafe { std::str::from_utf8_unchecked(&LOWER_BYTES) }
        }
    };
}
