import { createContext } from 'react'

import type { Chapter } from '@/services/chapterService'
import type { Novel } from '@/services/novelService'

/** 打开编辑器的参数 */
export interface EditorOpenOptions {
  /** 书信息（必传） */
  novel: Novel
  /** 章节信息（可选，新建时可不传） */
  chapter?: Chapter
  /**
   * 打开时默认选中的分卷序号（sequence）
   *
   * - 新建章节：作为分卷选择器初始值，用户可手动切换
   * - 编辑章节：作为只读回显值，选择器禁用
   * - 未传入时默认为 1（DEFAULT_VOLUME_SEQUENCE）
   */
  sequence?: number
  /** 提交回调 */
  onSubmit?: () => void
}

/** 编辑器上下文接口 */
export interface EditorContextType {
  /** 打开编辑器弹窗 */
  open: (options: EditorOpenOptions) => void
  /** 关闭编辑器弹窗 */
  close: () => void
}

/** 编辑器上下文 */
export const EditorContext = createContext<EditorContextType | null>(null)
