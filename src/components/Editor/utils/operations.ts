/**
 * 文档操作函数
 *
 * 纯函数，只操作 Paragraph[] 数据，不涉及 DOM。
 * 每次调用返回新的文档数组（不可变更新），为编辑器内核提供计算层。
 */

import { createParagraph, findParagraphIndex } from './document'

import type { Paragraph } from './document'
import type { EditorSelection, Point } from './selection'

/** 删除方向 */
export type DeleteDirection = 'backward' | 'forward'

/**
 * 在指定段落的指定偏移处插入文本
 *
 * @param doc - 文档段落数组
 * @param nodeId - 目标段落 ID
 * @param offset - 插入偏移量（0 起始）
 * @param text - 待插入的文本
 * @returns 更新后的段落数组
 *
 * @example
 * insertText(doc, 'p1', 3, '你好')
 * // 在 p1 段落的第 3 个字符处插入「你好」
 */
export function insertText(
  doc: Paragraph[],
  nodeId: string,
  offset: number,
  text: string,
): Paragraph[] {
  const idx = findParagraphIndex(doc, nodeId)
  if (idx === -1) return doc

  const paragraph = doc[idx]
  const newText = paragraph.text.slice(0, offset) + text + paragraph.text.slice(offset)

  return doc.map((p, i) => (i === idx ? { ...p, text: newText } : p))
}

/**
 * 将一个段落从偏移处拆分为两个
 *
 * 偏移位置之前的文本留在原段落（获得新 ID），
 * 之后的文本生成新段落。
 *
 * @param doc - 文档段落数组
 * @param nodeId - 目标段落 ID
 * @param offset - 拆分偏移量
 * @returns 更新后的段落数组
 *
 * @example
 * splitNode(doc, 'p1', 3)
 * // '你好世界' → '你好' + '世界'
 */
export function splitNode(doc: Paragraph[], nodeId: string, offset: number): Paragraph[] {
  const idx = findParagraphIndex(doc, nodeId)
  if (idx === -1) return doc

  const paragraph = doc[idx]
  const before = createParagraph(paragraph.text.slice(0, offset))
  const after = createParagraph(paragraph.text.slice(offset))

  return [...doc.slice(0, idx), before, after, ...doc.slice(idx + 1)]
}

/**
 * 删除段落内光标前/后的一个字符
 *
 * 段落内删除专用；跨段落合并请使用 mergeNodes。
 * 当无可删除字符时（如段首 Backspace / 段末 Delete）原样返回。
 *
 * @param doc - 文档段落数组
 * @param nodeId - 目标段落 ID
 * @param offset - 光标偏移量
 * @param direction - 删除方向
 * @returns 更新后的段落数组
 *
 * @example
 * deleteText(doc, 'p1', 3, 'backward')
 * // 删除 p1 段落第 2 个字符（光标前一个）
 */
export function deleteText(
  doc: Paragraph[],
  nodeId: string,
  offset: number,
  direction: DeleteDirection,
): Paragraph[] {
  const idx = findParagraphIndex(doc, nodeId)
  if (idx === -1) return doc

  const paragraph = doc[idx]
  const deleteOffset = direction === 'backward' ? offset - 1 : offset

  if (deleteOffset < 0 || deleteOffset >= paragraph.text.length) return doc

  const newText = paragraph.text.slice(0, deleteOffset) + paragraph.text.slice(deleteOffset + 1)

  return doc.map((p, i) => (i === idx ? { ...p, text: newText } : p))
}

/**
 * 将当前段落合并到上一段末尾
 *
 * 第一段不可向上合并，此时原样返回。
 * 合并后上一段文本 = prev.text + curr.text。
 *
 * @param doc - 文档段落数组
 * @param nodeId - 待合并段落 ID（将被消解）
 * @returns 更新后的段落数组
 *
 * @example
 * mergeNodes(doc, 'p2')
 * // ['你好', '世界'] → ['你好世界']
 */
export function mergeNodes(doc: Paragraph[], nodeId: string): Paragraph[] {
  const idx = findParagraphIndex(doc, nodeId)
  if (idx <= 0) return doc

  const prev = doc[idx - 1]
  const curr = doc[idx]
  const merged = { ...prev, text: prev.text + curr.text }

  return [...doc.slice(0, idx - 1), merged, ...doc.slice(idx + 1)]
}

/** deleteRange 的结果 */
export interface DeleteRangeResult {
  doc: Paragraph[]
  nextSelection: EditorSelection
}

/**
 * 删除选区覆盖的文本，返回折叠后的光标位置
 *
 * 三种情况：
 * - collapsed 选区 → 不做任何操作，返回原文档
 * - 段内选区 → 删除选中部分文本
 * - 跨段选区 → 首段保留前半 + 末段保留后半，中间段落全部删除
 *
 * @param doc - 文档段落数组
 * @param sel - 编辑器选区
 * @returns 删除结果（新文档 + 折叠光标），collapsed 时返回 null
 *
 * @example
 * // 段内选区：'你好世界' 选 '好世' → '你界'
 * deleteRange(doc, { anchor: { nodeId: 'p1', offset: 1 }, focus: { nodeId: 'p1', offset: 3 } })
 *
 * // 跨段选区：'你好' + '美丽' + '世界' 选 '好…世' → '你界'
 * deleteRange(doc, { anchor: { nodeId: 'p1', offset: 1 }, focus: { nodeId: 'p3', offset: 1 } })
 */
export function deleteRange(doc: Paragraph[], sel: EditorSelection): DeleteRangeResult | null {
  const { start, end } = normalizeSelection(doc, sel)
  if (start.nodeId === end.nodeId && start.offset === end.offset) return null

  const startIdx = findParagraphIndex(doc, start.nodeId)
  const endIdx = findParagraphIndex(doc, end.nodeId)
  if (startIdx === -1 || endIdx === -1) return null

  // 段内选区：裁剪文本
  if (startIdx === endIdx) {
    const paragraph = doc[startIdx]
    const newText = paragraph.text.slice(0, start.offset) + paragraph.text.slice(end.offset)
    return {
      doc: doc.map((p, i) => (i === startIdx ? { ...p, text: newText } : p)),
      nextSelection: { anchor: start, focus: start },
    }
  }

  // 跨段选区：首段前半 + 末段后半，中间段落删除
  const head = doc[startIdx].text.slice(0, start.offset)
  const tail = doc[endIdx].text.slice(end.offset)
  const merged = { ...doc[startIdx], text: head + tail }

  return {
    doc: [...doc.slice(0, startIdx), merged, ...doc.slice(endIdx + 1)],
    nextSelection: { anchor: start, focus: start },
  }
}

/**
 * 规范化选区方向，确保 start ≤ end（文档序）
 *
 * 同一段落内比较 offset，跨段落比较段落索引。
 *
 * @param doc - 文档段落数组
 * @param sel - 编辑器选区
 * @returns 有序的 { start, end }
 */
export function normalizeSelection(
  doc: Paragraph[],
  sel: EditorSelection,
): { start: Point; end: Point } {
  const anchorIdx = findParagraphIndex(doc, sel.anchor.nodeId)
  const focusIdx = findParagraphIndex(doc, sel.focus.nodeId)

  const anchorBefore =
    anchorIdx < focusIdx || (anchorIdx === focusIdx && sel.anchor.offset <= sel.focus.offset)

  return anchorBefore
    ? { start: sel.anchor, end: sel.focus }
    : { start: sel.focus, end: sel.anchor }
}
