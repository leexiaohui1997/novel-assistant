import { useContext } from 'react'

import type { EditorContextType } from '@/providers/EditorContext'

import { EditorContext } from '@/providers/EditorContext'

/**
 * 使用编辑器的 Hook
 *
 * @returns 编辑器上下文（open / close）
 * @throws 如果在 EditorProvider 外部使用会抛出错误
 */
export const useEditor = (): EditorContextType => {
  const context = useContext(EditorContext)

  if (!context) {
    throw new Error('useEditor 必须在 EditorProvider 内部使用')
  }

  return context
}
