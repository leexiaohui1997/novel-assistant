/**
 * 输入事件分发
 *
 * 根据 InputEvent.inputType 路由到对应操作函数，
 * 使用 handler record 模式替代 switch，保持圈复杂度 < 5。
 * 纯函数，只操作 Paragraph[] 数据，不涉及 DOM / React。
 */

import { findParagraphIndex } from './document'
import { deleteText, insertText, mergeNodes, splitNode } from './operations'

import type { Paragraph } from './document'
import type { EditorSelection } from './selection'

/** 输入操作结果 */
export interface InputResult {
  doc: Paragraph[]
  nextSelection: EditorSelection
}

/**
 * 根据 inputType 分发到对应操作函数
 *
 * 每个 handler 返回新文档和下一次光标位置，或 null 表示不处理。
 *
 * @param doc - 当前文档
 * @param sel - 当前光标位置
 * @param inputType - InputEvent.inputType
 * @param data - InputEvent.data（可为 null）
 * @returns 操作结果，null 表示不处理
 */
export function dispatchInput(
  doc: Paragraph[],
  sel: EditorSelection,
  inputType: string,
  data: string | null,
): InputResult | null {
  const handlers: Record<string, () => InputResult | null> = {
    insertText: () => ({
      doc: insertText(doc, sel.nodeId, sel.offset, data ?? ''),
      nextSelection: { nodeId: sel.nodeId, offset: sel.offset + (data?.length ?? 0) },
    }),

    insertParagraph: () => handleSplit(doc, sel),
    insertLineBreak: () => handleSplit(doc, sel),

    deleteContentBackward: () => handleDeleteBackward(doc, sel),
    deleteContentForward: () => ({
      doc: deleteText(doc, sel.nodeId, sel.offset, 'forward'),
      nextSelection: sel,
    }),
  }

  return handlers[inputType]?.() ?? null
}

/** 拆分段落并定位到新段首 */
function handleSplit(doc: Paragraph[], sel: EditorSelection): InputResult {
  const newDoc = splitNode(doc, sel.nodeId, sel.offset)
  const idx = findParagraphIndex(doc, sel.nodeId)
  return { doc: newDoc, nextSelection: { nodeId: newDoc[idx + 1].id, offset: 0 } }
}

/** Backspace：段首合并上一段，否则删除前一个字符 */
function handleDeleteBackward(doc: Paragraph[], sel: EditorSelection): InputResult | null {
  if (sel.offset === 0) {
    const newDoc = mergeNodes(doc, sel.nodeId)
    if (newDoc === doc) return null // 第一段，无法合并
    const idx = findParagraphIndex(doc, sel.nodeId)
    return {
      doc: newDoc,
      nextSelection: { nodeId: newDoc[idx - 1].id, offset: doc[idx - 1].text.length },
    }
  }
  return {
    doc: deleteText(doc, sel.nodeId, sel.offset, 'backward'),
    nextSelection: { nodeId: sel.nodeId, offset: sel.offset - 1 },
  }
}
