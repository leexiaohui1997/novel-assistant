# 工具函数索引

本文档列出项目中所有可用的工具函数，供 AI 助手参考使用。

## 数组工具 (array.ts)

### deepFindArr

在嵌套数组结构中深度查找满足条件的第一个元素
文件：`src/utils/array.ts`

### deepFindArrWithPath

在嵌套数组结构中深度查找，并返回从根到目标元素的完整路径
文件：`src/utils/array.ts`

---

## 日志工具 (logger.ts)

### logger.debug

输出调试级别日志
文件：`src/utils/logger.ts`

### logger.info

输出信息级别日志
文件：`src/utils/logger.ts`

### logger.warn

输出警告级别日志
文件：`src/utils/logger.ts`

### logger.error

输出错误级别日志
文件：`src/utils/logger.ts`

---

## 常量定义 (constants.ts)

### APP_NAME

应用名称常量
文件：`src/utils/constants.ts`

### APP_VERSION

应用版本常量
文件：`src/utils/constants.ts`

---

## 数字工具 (number.ts)

### numToCn

将阿拉伯数字转为中文数字（支持 1~999，超出范围返回原数字字符串）
文件：`src/utils/number.ts`

---

## 键盘工具 (keyboard.ts)

### getShortcutText

从键盘事件中提取快捷键组合文本（如 "Ctrl + C"）
文件：`src/utils/keyboard.ts`

---

## 错误工具 (error.ts)

### getErrorMsg

从未知类型的错误中提取可读的错误消息字符串。优先返回 Error 实例的 message，其他类型则转为字符串
文件：`src/utils/error.ts`

---

## 日期工具 (date.ts)

### formatDateTime

将 ISO 字符串 / Date / 时间戳格式化为日期时间字符串（默认 `YYYY-MM-DD HH:mm:ss`），无效值返回兜底
文件：`src/utils/date.ts`

### formatDate

将 ISO 字符串 / Date / 时间戳格式化为日期字符串（`YYYY-MM-DD`）
文件：`src/utils/date.ts`

---

**使用说明：**

- 需要导入对应文件才能使用这些函数
- 所有工具函数都位于 `src/utils/` 目录下
- 新增工具函数时请同步更新此文档
