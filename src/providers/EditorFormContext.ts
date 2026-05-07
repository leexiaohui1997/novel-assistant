import { createContext, useContext } from 'react'

import type { Chapter } from '@/services/chapterService'
import type { Novel } from '@/services/novelService'

/** 编辑器表单上下文接口 */
export interface EditorFormContextType {
  /** 当前小说 */
  novel: Novel
  /** 当前章节（新建模式下为 undefined） */
  chapter?: Chapter
  /** 编辑器当前标题 */
  title: string
  /** 编辑器当前正文 */
  content: string
  /** 设置编辑器正文 */
  applyContent: (content: string) => void
}

/** 编辑器表单上下文（仅编辑器弹窗子树内可用） */
export const EditorFormContext = createContext<EditorFormContextType | null>(null)

/**
 * 获取编辑器表单上下文
 *
 * 必须在 EditorFormContext.Provider 子树内使用，否则抛出错误。
 */
export function useEditorForm(): EditorFormContextType {
  const ctx = useContext(EditorFormContext)
  if (!ctx) {
    throw new Error('useEditorForm 必须在 EditorFormContext.Provider 内使用')
  }
  return ctx
}
