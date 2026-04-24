/**
 * 快照栈（Undo / Redo）
 *
 * 采用 past / present / future 三栈结构，
 * 所有函数为纯函数，返回新的 HistoryState，不 mutate 输入。
 */

import type { Paragraph } from './document'
import type { EditorSelection } from './selection'

/** 快照 — 文档 + 选区的不可变记录 */
export interface Snapshot {
  doc: Paragraph[]
  selection: EditorSelection
}

/** 快照栈状态 */
export interface HistoryState {
  past: Snapshot[]
  present: Snapshot
  future: Snapshot[]
}

/** 历史栈最大深度 */
const MAX_HISTORY = 100

/**
 * 初始化快照栈
 *
 * @param initial - 初始快照（文档 + 选区）
 * @returns 初始 HistoryState
 */
export function createHistory(initial: Snapshot): HistoryState {
  return { past: [], present: initial, future: [] }
}

/**
 * 推入新快照（编辑操作后调用）
 *
 * 当前 present 压入 past，新快照成为 present，future 清空。
 * past 超过 MAX_HISTORY 时丢弃最旧记录。
 *
 * @param history - 当前快照栈
 * @param snapshot - 新快照
 * @returns 更新后的 HistoryState
 */
export function pushUndo(history: HistoryState, snapshot: Snapshot): HistoryState {
  const past = [...history.past, history.present]
  if (past.length > MAX_HISTORY) past.shift()

  return { past, present: snapshot, future: [] }
}

/**
 * 撤销：回退到上一个快照
 *
 * past 末尾弹出到 present，旧 present 推入 future 头部。
 *
 * @param history - 当前快照栈
 * @returns 更新后的 HistoryState，past 为空时返回原状态
 */
export function undo(history: HistoryState): HistoryState {
  if (history.past.length === 0) return history

  const past = [...history.past]
  const prev = past.pop()!
  const future = [history.present, ...history.future]

  return { past, present: prev, future }
}

/**
 * 重做：前进到下一个快照
 *
 * future 头部弹出作为 present，旧 present 推入 past。
 *
 * @param history - 当前快照栈
 * @returns 更新后的 HistoryState，future 为空时返回原状态
 */
export function redo(history: HistoryState): HistoryState {
  if (history.future.length === 0) return history

  const future = [...history.future]
  const next = future.shift()!
  const past = [...history.past, history.present]

  return { past, present: next, future }
}
