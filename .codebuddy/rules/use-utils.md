---
trigger: always
---

# 工具函数使用规范

## 核心原则

在开发过程中，**必须优先检查和使用现有的工具函数**，避免重复造轮子。

## 使用前必查

在实现任何功能前，先查阅对应的工具函数文档，确认是否已有可用的工具函数。

### 检查步骤

#### TypeScript/JavaScript (前端)

1. 查看 `src/utils/README.md` 中的工具函数索引
2. 根据功能需求匹配对应的工具函数
3. 阅读工具函数的完整实现和文档注释
4. 确认工具函数满足需求后直接使用

#### Rust (后端)

1. 查看 `src-tauri/src/utils/README.md` 中的工具函数和宏索引
2. 根据功能需求匹配对应的工具函数或宏
3. 阅读工具函数的完整实现和文档注释
4. 确认工具函数满足需求后直接使用

## 禁止行为

❌ **不要重复实现已有的工具函数**

```typescript
// ❌ 错误：自己实现递归查找
const findItem = (items, path) => {
  for (const item of items) {
    if (item.path === path) return item
    if (item.children) {
      const found = findItem(item.children, path)
      if (found) return found
    }
  }
  return null
}

// ✅ 正确：使用现有工具函数
import { deepFindArr } from '@/utils/array'
const foundItem = deepFindArr<MenuItem>(items, (item) => item.path === path)
```

❌ **不要使用 console.log**

```typescript
// ❌ 错误
console.log('debug info')

// ✅ 正确
import { logger } from '@/utils/logger'
logger.debug('debug info')
```

❌ **不要硬编码应用信息**

```typescript
// ❌ 错误
const appName = 'Novel Assistant'

// ✅ 正确
import { APP_NAME } from '@/utils/constants'
```

## 新增工具函数规范

当确实需要新增工具函数时：

### TypeScript/JavaScript (前端)

1. **位置**: 放在 `src/utils/` 目录下
2. **命名**: 使用有意义的文件名（如 `array.ts`, `string.ts`, `date.ts`）
3. **类型**: 必须使用 TypeScript，提供完整的类型定义
4. **文档**: 添加详细的 JSDoc 注释，包含：
   - 函数功能描述
   - 参数说明
   - 返回值说明
   - 使用示例
5. **更新索引**: 同步更新 `src/utils/README.md`
6. **泛型优先**: 尽可能使用泛型提高复用性

### Rust (后端)

1. **位置**: 放在 `src-tauri/src/utils/` 目录下
2. **命名**: 使用有意义的文件名（如 `macros.rs`, `converters.rs`）
3. **类型**: 提供完整的类型定义和 trait 实现
4. **文档**: 添加详细的 Rustdoc 注释，包含：
   - 函数/宏功能描述
   - 参数说明
   - 返回值说明
   - 使用示例
5. **更新索引**: 同步更新 `src-tauri/src/utils/README.md`
6. **宏优先**: 对于重复的模式，考虑使用宏来简化代码

### 示例

```typescript
/**
 * 函数功能描述
 *
 * @param param1 - 参数1说明
 * @param param2 - 参数2说明
 * @returns 返回值说明
 *
 * @example
 * // 使用示例
 * const result = myFunction(arg1, arg2)
 */
export function myFunction<T>(param1: T[], param2: string): T | null {
  // 实现
}
```

## 导入规范

### TypeScript/JavaScript (前端)

```typescript
// ✅ 正确的导入方式
import { deepFindArr } from '@/utils/array'
import { logger } from '@/utils/logger'
import { APP_NAME } from '@/utils/constants'

// ❌ 避免直接从子目录导入
import { deepFindArr } from '@/utils/array/array' // 如果未来有子目录
```

### Rust (后端)

```rust
// ✅ 正确的导入方式
use crate::string_enum;  // 导入宏
use crate::utils::macros;  // 导入模块

// ✅ 使用宏
string_enum! {
    pub enum Status {
        Active,
        Inactive,
    }
}
```

## 代码审查要点

在 Code Review 时重点检查：

### TypeScript/JavaScript (前端)

- [ ] 是否使用了现有的工具函数
- [ ] 是否有重复实现的逻辑
- [ ] 是否正确使用了 logger 而非 console
- [ ] 新增的工具函数是否有完整文档
- [ ] 是否更新了 `src/utils/README.md` 索引

### Rust (后端)

- [ ] 是否使用了现有的工具函数或宏
- [ ] 是否有重复实现的逻辑（特别是枚举的字符串转换）
- [ ] 新增的工具函数/宏是否有完整文档
- [ ] 是否更新了 `src-tauri/src/utils/README.md` 索引
- [ ] 是否优先考虑使用宏来简化重复代码

## 优势

使用统一的工具函数可以：

✅ **提高代码质量**: 经过测试的可靠实现  
✅ **减少代码重复**: 避免多处实现相同逻辑  
✅ **便于维护**: 修改一处，全局生效  
✅ **提升性能**: 优化的通用实现  
✅ **统一风格**: 保持代码一致性  
✅ **AI 辅助**: AI 能更好地推荐和使用现有工具
