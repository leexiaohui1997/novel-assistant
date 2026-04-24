/**
 * 文档模型工具
 *
 * 提供 Document Model 的核心类型定义与操作函数，
 * 将扁平文本转换为结构化的 Paragraph 数组，为编辑器内核提供数据层基础。
 */

/** 段落节点 — 文档的最小结构单元 */
export interface Paragraph {
  /** 节点唯一标识，用于 React key 和 DOM 反查 */
  id: string
  /** 段落文本内容 */
  text: string
}

/**
 * 创建一个段落节点
 *
 * @param text - 段落文本，默认为空字符串
 * @returns 带有唯一 id 的 Paragraph 对象
 *
 * @example
 * const p = createParagraph('你好世界')
 * // => { id: 'a1b2c3...', text: '你好世界' }
 */
export function createParagraph(text = ''): Paragraph {
  return { id: crypto.randomUUID(), text }
}

/**
 * 将纯文本解析为 Paragraph 数组
 *
 * 按换行符拆分文本，每一行生成一个段落节点。
 * 空字符串会生成一个空段落（保证文档至少有一个节点）。
 *
 * @param content - 纯文本内容
 * @returns Paragraph 数组
 *
 * @example
 * parseDocument('第一段\n第二段')
 * // => [{ id: '...', text: '第一段' }, { id: '...', text: '第二段' }]
 */
export function parseDocument(content: string): Paragraph[] {
  if (content === '') return [createParagraph()]
  return content.split('\n').map((line) => createParagraph(line))
}

/**
 * 将 Paragraph 数组还原为纯文本
 *
 * @param doc - Paragraph 数组
 * @returns 用换行符连接的纯文本
 *
 * @example
 * toPlainText([{ id: '1', text: '第一段' }, { id: '2', text: '第二段' }])
 * // => '第一段\n第二段'
 */
export function toPlainText(doc: Paragraph[]): string {
  return doc.map((p) => p.text).join('\n')
}

/**
 * 根据 ID 查找段落索引
 *
 * @param doc - Paragraph 数组
 * @param nodeId - 目标段落 ID
 * @returns 索引位置，未找到返回 -1
 *
 * @example
 * findParagraphIndex(doc, 'a1b2c3')
 * // => 2
 */
export function findParagraphIndex(doc: Paragraph[], nodeId: string): number {
  return doc.findIndex((p) => p.id === nodeId)
}
