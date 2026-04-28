import { Descendant, Text } from 'slate'

import type { NovelElement } from '@/types/slate'

/**
 * 默认初始值
 */
export const initialValue: Descendant[] = [
  {
    type: 'paragraph',
    children: [{ text: '' }],
  } as NovelElement,
]

/**
 * 将字符串反序列化为 Slate 节点结构
 *
 * @param text - 要转换的文本字符串
 * @returns Slate 节点数组
 *
 * @example
 * ```typescript
 * const nodes = deserializeValue('Hello World')
 * // [{ type: 'paragraph', children: [{ text: 'Hello World' }] }]
 * ```
 */
export function deserializeValue(text: string): Descendant[] {
  if (!text) return initialValue
  return text.split('\n').map((line) => ({
    type: 'paragraph',
    children: [{ text: line }],
  })) as NovelElement[]
}

/**
 * 将 Slate 节点结构序列化为字符串
 *
 * @param nodes - Slate 节点数组
 * @returns 纯文本字符串
 *
 * @example
 * ```typescript
 * const text = serializeValue(nodes)
 * // 'Hello World'
 * ```
 */
export function serializeValue(nodes: Descendant[]): string {
  /**
   * 递归提取节点中的文本内容
   *
   * @param node - Slate 节点
   * @returns 节点中的文本内容
   */
  function extractText(node: Descendant): string {
    // 如果是文本节点，直接返回文本
    if (Text.isText(node)) {
      return node.text
    }

    // 如果是元素节点，递归处理所有子节点
    if ('children' in node) {
      return (node.children as Descendant[]).map(extractText).join('')
    }

    return ''
  }

  // 处理顶层节点数组，用换行符连接
  return nodes.map(extractText).join('\n')
}
