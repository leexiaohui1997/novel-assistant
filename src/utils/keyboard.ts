/**
 * 键盘工具函数
 *
 * 提供键盘事件相关的解析/处理能力。
 */

/** 修饰键的 key 值集合，用于判断当前按键是否为纯修饰键 */
const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta'])

/** 修饰键的显示名称映射（如 Control → Ctrl） */
const MODIFIER_LABELS: Record<string, string> = {
  Control: 'Ctrl',
  Shift: 'Shift',
  Alt: 'Alt',
  Meta: 'Meta',
}

/** 修饰键的拼接顺序：Ctrl → Shift → Alt → Meta */
const MODIFIER_ORDER = ['Control', 'Shift', 'Alt', 'Meta'] as const

/** 修饰键到事件对象布尔属性的映射（如 Control → ctrlKey） */
const MODIFIER_PROPS = {
  Control: 'ctrlKey',
  Shift: 'shiftKey',
  Alt: 'altKey',
  Meta: 'metaKey',
} as const

/**
 * 从键盘事件中提取快捷键组合文本
 *
 * 按 Ctrl → Shift → Alt → Meta 顺序拼接修饰键，再拼接主按键。
 * - 当 `e.key` 为空白时回退使用 `e.code`
 * - 单字符按键仅在存在修饰键时转为大写
 * - 纯修饰键按下时只返回修饰键名称
 *
 * @typeParam T - 事件对象类型，需包含 key 及可选的 code、ctrlKey 等属性
 * @param e - 键盘事件对象（兼容 KeyboardEvent 及类似结构）
 * @returns 快捷键组合文本，如 "Ctrl + C"、"Shift + Alt + F5"、"Shift"
 *
 * @example
 * getShortcutText(new KeyboardEvent('keydown', { key: 'c', ctrlKey: true }))
 * // 'Ctrl + C'
 *
 * getShortcutText(new KeyboardEvent('keydown', { key: 'Shift' }))
 * // 'Shift'
 */
export function getShortcutText<
  T extends {
    key: string
    code?: string
    ctrlKey?: boolean
    shiftKey?: boolean
    altKey?: boolean
    metaKey?: boolean
  },
>(e: T): string {
  const parts: string[] = []
  // key 为空白时回退到 code（如某些特殊按键场景）
  const key = (e.key.trim() ? e.key : e.code) || ''

  for (const mod of MODIFIER_ORDER) {
    const isPressed = e[MODIFIER_PROPS[mod]]
    if (isPressed) {
      parts.push(MODIFIER_LABELS[mod])
    }
  }

  // 非修饰键才追加：单字符 + 有修饰键时大写，否则保留原样
  if (!MODIFIER_KEYS.has(key)) {
    parts.push(key.length === 1 && parts.length > 0 ? key.toUpperCase() : key)
  }

  return parts.join(' + ')
}
