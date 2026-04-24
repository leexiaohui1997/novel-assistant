/**
 * 选区模型工具
 *
 * 将 DOM 选区（window.getSelection）与文档模型（Paragraph[]）桥接，
 * 提供模型坐标系下的光标读写能力。
 */

/** 选区端点 — 模型坐标系下的一个点 */
export interface Point {
  /** 段落节点 ID（对应 data-node-id） */
  nodeId: string
  /** 段落内偏移量（字符级，0 起始） */
  offset: number
}

/** 编辑器选区 — 用模型坐标表示的锚点 + 焦点 */
export interface EditorSelection {
  /** 锚点 — 选区起始端（鼠标按下处） */
  anchor: Point
  /** 焦点 — 选区结束端（鼠标松开处），与 anchor 相同时为光标 */
  focus: Point
}

/**
 * 判断选区是否为折叠光标
 *
 * @param sel - 编辑器选区
 * @returns anchor 与 focus 重合时返回 true
 *
 * @example
 * isCollapsed({ anchor: { nodeId: 'p1', offset: 3 }, focus: { nodeId: 'p1', offset: 3 } })
 * // => true
 */
export function isCollapsed(sel: EditorSelection): boolean {
  return sel.anchor.nodeId === sel.focus.nodeId && sel.anchor.offset === sel.focus.offset
}

const NODE_ID_ATTR = 'data-node-id'

/**
 * 从当前 DOM 选区反查模型坐标
 *
 * 将 DOM Selection 的 anchorNode/focusNode 映射为模型中的
 * `{ anchor, focus }` 双端选区。collapsed 选区时两者重合。
 * 当选区不存在或不在编辑区域内时返回 null。
 *
 * @param container - 编辑器根容器元素
 * @returns 模型选区，或 null
 *
 * @example
 * const sel = getSelectionFromDOM(editorEl)
 * // => { anchor: { nodeId: 'a1b2c3', offset: 2 }, focus: { nodeId: 'a1b2c3', offset: 5 } }
 */
export function getSelectionFromDOM(container: HTMLElement): EditorSelection | null {
  const domSel = window.getSelection()
  if (!domSel || domSel.rangeCount === 0) return null

  const anchor = resolvePoint(container, domSel.anchorNode, domSel.anchorOffset)
  if (!anchor) return null

  // collapsed 选区时 focus === anchor，无需重复计算
  if (domSel.isCollapsed) return { anchor, focus: anchor }

  const focus = resolvePoint(container, domSel.focusNode, domSel.focusOffset)
  if (!focus) return null

  return { anchor, focus }
}

/**
 * 将 DOM 中的一个节点 + 偏移解析为模型 Point
 *
 * 向上查找带 data-node-id 的祖先段落，累加前序文本节点长度得到模型偏移。
 */
function resolvePoint(container: HTMLElement, node: Node | null, offset: number): Point | null {
  if (!node) return null

  const target = node instanceof Text ? node.parentElement : (node as HTMLElement)
  const paragraphEl = target?.closest(`[${NODE_ID_ATTR}]`) as HTMLElement | null
  if (!paragraphEl || !container.contains(paragraphEl)) return null

  const nodeId = paragraphEl.getAttribute(NODE_ID_ATTR)
  if (!nodeId) return null

  return { nodeId, offset: calcOffset(paragraphEl, node, offset) }
}

/**
 * 将模型选区应用到 DOM
 *
 * collapsed 选区设置单点光标，非 collapsed 选区设置跨段落 Range。
 * 若未找到对应 DOM 节点则静默跳过。
 *
 * @param container - 编辑器根容器元素
 * @param selection - 模型选区
 *
 * @example
 * setDOMSelection(editorEl, { anchor: { nodeId: 'a1b2c3', offset: 2 }, focus: { nodeId: 'a1b2c3', offset: 5 } })
 */
export function setDOMSelection(container: HTMLElement, selection: EditorSelection): void {
  const anchorEl = container.querySelector(`[${NODE_ID_ATTR}="${selection.anchor.nodeId}"]`)
  const focusEl = container.querySelector(`[${NODE_ID_ATTR}="${selection.focus.nodeId}"]`)
  if (!anchorEl || !focusEl) return

  const anchorTarget = findTextOffset(anchorEl, selection.anchor.offset)
  const range = document.createRange()
  range.setStart(anchorTarget.node, anchorTarget.offset)

  if (isCollapsed(selection)) {
    range.collapse(true)
  } else {
    const focusTarget = findTextOffset(focusEl, selection.focus.offset)
    range.setEnd(focusTarget.node, focusTarget.offset)
  }

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
