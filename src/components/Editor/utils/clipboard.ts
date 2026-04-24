/**
 * 剪贴板操作函数
 *
 * 纯函数，处理文本的复制提取与粘贴插入，
 * 为编辑器提供 Ctrl+C/V/X 的数据层支持。
 */

import { createParagraph, findParagraphIndex } from './document'
import { deleteRange, normalizeSelection } from './operations'
import { isCollapsed } from './selection'

import type { Paragraph } from './document'
import type { EditorSelection } from './selection'

/**
 * 从文档选区中提取纯文本
 *
 * 段内选区裁剪对应区间，跨段选区用换行符连接。
 * collapsed 选区返回空字符串。
 *
 * @param doc - 文档段落数组
 * @param sel - 编辑器选区
 * @returns 选中区域的纯文本
 */
export function getSelectedText(doc: Paragraph[], sel: EditorSelection): string {
  if (isCollapsed(sel)) return ''

  const { start, end } = normalizeSelection(doc, sel)
  const startIdx = findParagraphIndex(doc, start.nodeId)
  const endIdx = findParagraphIndex(doc, end.nodeId)
  if (startIdx === -1 || endIdx === -1) return ''

  if (startIdx === endIdx) {
    return doc[startIdx].text.slice(start.offset, end.offset)
  }

  const lines: string[] = [doc[startIdx].text.slice(start.offset)]
  for (let i = startIdx + 1; i < endIdx; i++) {
    lines.push(doc[i].text)
  }
  lines.push(doc[endIdx].text.slice(0, end.offset))

  return lines.join('\n')
}

/** 剪贴板操作结果 */
export interface ClipboardResult {
  doc: Paragraph[]
  nextSelection: EditorSelection
}

/**
 * 粘贴纯文本到文档
 *
 * 单行粘贴在光标处直接插入；多行粘贴拆分当前段落，
 * 中间行生成新段落，末行拼接光标后文本。
 * 非折叠选区时先删除选中内容再粘贴。
 *
 * @param doc - 文档段落数组
 * @param sel - 编辑器选区
 * @param text - 粘贴的纯文本
 * @returns 操作结果，null 表示无需操作
 */
export function pasteText(
  doc: Paragraph[],
  sel: EditorSelection,
  text: string,
): ClipboardResult | null {
  if (!text) return null

  // 非折叠选区先删除
  let currentDoc = doc
  let currentSel = sel
  if (!isCollapsed(sel)) {
    const deleted = deleteRange(doc, sel)
    if (!deleted) return null
    currentDoc = deleted.doc
    currentSel = deleted.nextSelection
  }

  const lines = text.split('\n')
  const { nodeId, offset } = currentSel.anchor
  const idx = findParagraphIndex(currentDoc, nodeId)
  if (idx === -1) return null

  // 单行粘贴：直接插入文本
  if (lines.length === 1) {
    const paragraph = currentDoc[idx]
    const newText = paragraph.text.slice(0, offset) + lines[0] + paragraph.text.slice(offset)
    const newPoint = { nodeId, offset: offset + lines[0].length }
    return {
      doc: currentDoc.map((p, i) => (i === idx ? { ...p, text: newText } : p)),
      nextSelection: { anchor: newPoint, focus: newPoint },
    }
  }

  // 多行粘贴：拆分当前段落
  const paragraph = currentDoc[idx]
  const head = paragraph.text.slice(0, offset)
  const tail = paragraph.text.slice(offset)

  const firstLine = { ...paragraph, text: head + lines[0] }
  const middleLines = lines.slice(1, -1).map((line) => createParagraph(line))
  const lastParagraph = createParagraph(lines[lines.length - 1] + tail)

  const newDoc = [
    ...currentDoc.slice(0, idx),
    firstLine,
    ...middleLines,
    lastParagraph,
    ...currentDoc.slice(idx + 1),
  ]

  const newPoint = { nodeId: lastParagraph.id, offset: lines[lines.length - 1].length }
  return {
    doc: newDoc,
    nextSelection: { anchor: newPoint, focus: newPoint },
  }
}
