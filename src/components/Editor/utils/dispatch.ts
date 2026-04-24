/**
 * 输入事件分发
 *
 * 根据 InputEvent.inputType 路由到对应操作函数，
 * 使用 handler record 模式替代 switch，保持圈复杂度 < 5。
 * 纯函数，只操作 Paragraph[] 数据，不涉及 DOM / React。
 */

import { findParagraphIndex } from './document'
import { deleteRange, deleteText, insertText, mergeNodes, splitNode } from './operations'
import { isCollapsed } from './selection'

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
 * 非 collapsed 选区时先折叠（删除选中内容），再执行对应操作。
 *
 * @param doc - 当前文档
 * @param sel - 当前选区
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
    insertText: () => handleInsertText(doc, sel, data ?? ''),

    insertParagraph: () => handleSplit(doc, sel),
    insertLineBreak: () => handleSplit(doc, sel),

    deleteContentBackward: () => handleDeleteBackward(doc, sel),
    deleteContentForward: () => handleDeleteForward(doc, sel),
    deleteContent: () => handleDeleteContent(doc, sel),
  }

  return handlers[inputType]?.() ?? null
}

/** 插入文本：非 collapsed 先折叠，再在光标处插入 */
function handleInsertText(doc: Paragraph[], sel: EditorSelection, text: string): InputResult {
  if (!isCollapsed(sel)) {
    const result = deleteRange(doc, sel)
    if (result) return handleInsertText(result.doc, result.nextSelection, text)
  }

  const { nodeId, offset } = sel.anchor
  return {
    doc: insertText(doc, nodeId, offset, text),
    nextSelection: {
      anchor: { nodeId, offset: offset + text.length },
      focus: { nodeId, offset: offset + text.length },
    },
  }
}

/** 拆分段落并定位到新段首 */
function handleSplit(doc: Paragraph[], sel: EditorSelection): InputResult {
  if (!isCollapsed(sel)) {
    const result = deleteRange(doc, sel)
    if (result) return handleSplit(result.doc, result.nextSelection)
  }

  const { nodeId, offset } = sel.anchor
  const newDoc = splitNode(doc, nodeId, offset)
  const idx = findParagraphIndex(doc, nodeId)
  const nextNodeId = newDoc[idx + 1].id
  const nextPoint = { nodeId: nextNodeId, offset: 0 }
  return { doc: newDoc, nextSelection: { anchor: nextPoint, focus: nextPoint } }
}

/** Backspace：非 collapsed 折叠，段首合并上一段，否则删除前一个字符 */
function handleDeleteBackward(doc: Paragraph[], sel: EditorSelection): InputResult | null {
  if (!isCollapsed(sel)) return handleDeleteContent(doc, sel)

  const { nodeId, offset } = sel.anchor
  if (offset === 0) {
    const newDoc = mergeNodes(doc, nodeId)
    if (newDoc === doc) return null // 第一段，无法合并
    const idx = findParagraphIndex(doc, nodeId)
    const prevPoint = { nodeId: newDoc[idx - 1].id, offset: doc[idx - 1].text.length }
    return { doc: newDoc, nextSelection: { anchor: prevPoint, focus: prevPoint } }
  }

  const prevPoint = { nodeId, offset: offset - 1 }
  return {
    doc: deleteText(doc, nodeId, offset, 'backward'),
    nextSelection: { anchor: prevPoint, focus: prevPoint },
  }
}

/** Delete：非 collapsed 折叠，否则删除后一个字符 */
function handleDeleteForward(doc: Paragraph[], sel: EditorSelection): InputResult | null {
  if (!isCollapsed(sel)) return handleDeleteContent(doc, sel)

  const { nodeId, offset } = sel.anchor
  const point = { nodeId, offset }
  return {
    doc: deleteText(doc, nodeId, offset, 'forward'),
    nextSelection: { anchor: point, focus: point },
  }
}

/** 选区删除（deleteContent inputType，或 Backspace/Delete 在有选区时） */
function handleDeleteContent(doc: Paragraph[], sel: EditorSelection): InputResult | null {
  return deleteRange(doc, sel)
}
