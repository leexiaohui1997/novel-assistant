/**
 * 选区模型工具
 *
 * 将 DOM 选区（window.getSelection）与文档模型（Paragraph[]）桥接，
 * 提供模型坐标系下的光标读写能力。
 */

/** 编辑器选区 — 用模型坐标表示的光标位置 */
export interface EditorSelection {
  /** 段落节点 ID（对应 data-node-id） */
  nodeId: string
  /** 段落内偏移量（字符级，0 起始） */
  offset: number
}

const NODE_ID_ATTR = 'data-node-id'

/**
 * 从当前 DOM 选区反查模型坐标
 *
 * 向上查找最近带 `data-node-id` 的祖先元素，
 * 将 DOM offset 转换为模型中的 `{ nodeId, offset }`。
 * 当选区不存在或不在编辑区域内时返回 null。
 *
 * @param container - 编辑器根容器元素
 * @returns 模型坐标，或 null
 *
 * @example
 * const sel = getSelectionFromDOM(editorEl)
 * // => { nodeId: 'a1b2c3...', offset: 5 }
 */
export function getSelectionFromDOM(container: HTMLElement): EditorSelection | null {
  const domSel = window.getSelection()
  if (!domSel || domSel.rangeCount === 0) return null

  const range = domSel.getRangeAt(0)
  const node = range.startContainer

  // 文本节点向上找带 data-node-id 的祖先
  const target = node instanceof Text ? node.parentElement : (node as HTMLElement)
  const paragraphEl = target?.closest(`[${NODE_ID_ATTR}]`) as HTMLElement | null
  if (!paragraphEl || !container.contains(paragraphEl)) return null

  const nodeId = paragraphEl.getAttribute(NODE_ID_ATTR)
  if (!nodeId) return null

  // 计算 offset：若起点在段落元素自身上则用 range.startOffset，
  // 否则累加前序文本节点的长度
  const offset = calcOffset(paragraphEl, node, range.startOffset)

  return { nodeId, offset }
}

/**
 * 将模型坐标应用到 DOM 选区
 *
 * 在指定段落节点内定位文本偏移，设置 collapsed 光标。
 * 若未找到对应 DOM 节点则静默跳过。
 *
 * @param container - 编辑器根容器元素
 * @param selection - 模型坐标
 *
 * @example
 * setDOMSelection(editorEl, { nodeId: 'a1b2c3...', offset: 5 })
 */
export function setDOMSelection(container: HTMLElement, selection: EditorSelection): void {
  const paragraphEl = container.querySelector(`[${NODE_ID_ATTR}="${selection.nodeId}"]`)
  if (!paragraphEl) return

  const target = findTextOffset(paragraphEl, selection.offset)

  const range = document.createRange()
  range.setStart(target.node, target.offset)
  range.collapse(true)

  const domSel = window.getSelection()
  if (domSel) {
    domSel.removeAllRanges()
    domSel.addRange(range)
  }
}

/**
 * 计算段落内模型偏移量
 *
 * 从段落元素的子节点中累加前序文本节点长度，
 * 将 DOM (node, offset) 映射为模型 offset。
 * 空段落（仅含 <br>）时光标落在段落元素自身，直接返回 0。
 */
function calcOffset(paragraphEl: HTMLElement, startNode: Node, startOffset: number): number {
  if (startNode === paragraphEl) return 0

  let offset = 0
  const walker = document.createTreeWalker(paragraphEl, NodeFilter.SHOW_TEXT)
  let current = walker.nextNode() as Text | null

  while (current) {
    if (current === startNode) return offset + startOffset
    offset += current.length
    current = walker.nextNode() as Text | null
  }

  return offset
}

/**
 * 在段落元素内定位到指定模型偏移的文本节点和偏移
 *
 * 遍历子文本节点，累加长度直到达到目标偏移。
 */
function findTextOffset(
  paragraphEl: Element,
  targetOffset: number,
): { node: Text; offset: number } {
  const walker = document.createTreeWalker(paragraphEl, NodeFilter.SHOW_TEXT)
  let remaining = targetOffset

  let current = walker.nextNode() as Text | null
  while (current) {
    if (remaining <= current.length) {
      return { node: current, offset: remaining }
    }
    remaining -= current.length
    current = walker.nextNode() as Text | null
  }

  // 空段落（仅含 <br>）：将光标放在段落元素自身，offset = 0
  return { node: paragraphEl as unknown as Text, offset: 0 }
}
