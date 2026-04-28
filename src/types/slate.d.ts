/**
 * Slate 编辑器自定义类型定义
 *
 * 扩展 Slate 的默认类型系统，定义项目中使用的自定义元素和文本节点类型
 */

import { BaseEditor, Descendant } from 'slate'
import { HistoryEditor } from 'slate-history'
import { ReactEditor } from 'slate-react'

/**
 * 小说文本节点类型
 *
 * 表示编辑器中的基本文本单元，包含文本内容和可选的格式属性
 */
export type NovelText = { text: string }

/**
 * 小说元素节点类型
 *
 * 表示编辑器中的结构元素（如段落、标题等）
 * 目前仅支持段落类型，后续可扩展标题、列表、引用等
 */
export type NovelElement = {
  type: 'paragraph'
  children: NovelText[]
}

/**
 * 声明模块扩展，将自定义类型注入 Slate 的类型系统
 */
declare module 'slate' {
  interface CustomTypes {
    Editor: BaseEditor & ReactEditor & HistoryEditor
    Element: NovelElement
    Text: NovelText
  }
}

/**
 * 导出常用的类型别名，方便在其他文件中使用
 */
export type EditorNode = NovelElement | NovelText
export type EditorDescendant = Descendant
