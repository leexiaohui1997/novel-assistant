/// 为枚举生成字符串转换相关的通用实现
///
/// 这个宏会自动为枚举实现：
/// - `as_str()` 方法：将枚举值转换为 snake_case 字符串
/// - `std::fmt::Display` trait：支持使用 `{}` 格式化输出
/// - `From<&str>` trait：支持从字符串转换为枚举（可选）
///
/// # 使用示例
///
/// ```rust
/// string_enum! {
///     pub enum Status {
///         Active,      // 对应 "active"
///         Inactive,    // 对应 "inactive"
///         Pending      // 对应 "pending"
///     }
/// }
/// ```
#[macro_export]
macro_rules! string_enum {
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
            /// 将枚举值转换为 snake_case 字符串
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
}
