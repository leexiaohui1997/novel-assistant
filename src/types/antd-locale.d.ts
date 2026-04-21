/**
 * Ant Design Locale 模块类型声明
 *
 * 修复 Ant Design locale 文件的类型定义问题
 * 实际运行时导出的是 { default: Locale }，但官方类型定义错误
 */

import type { Locale } from 'antd/es/locale'

declare module 'antd/locale/zh_CN' {
  const zhCN: { default: Locale }
  export = zhCN
}

declare module 'antd/locale/en_US' {
  const enUS: { default: Locale }
  export = enUS
}
