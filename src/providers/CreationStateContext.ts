import { createContext } from 'react'
import { UIMatch } from 'react-router-dom'

import { CreationState } from '@/services/creationStateService'

/**
 * 获取指定 key 的状态值
 */
export type GetState = <K extends keyof CreationState>(key: K) => CreationState[K]

/**
 * 设置状态值（支持重载）
 */
export interface SetState {
  <K extends keyof CreationState>(state: K, value: CreationState[K]): void
  <K extends keyof CreationState>(state: Pick<CreationState, K>): void
}

/**
 * 创作状态上下文接口
 */
export interface CreationStateContextType {
  /** 小说 ID */
  novelId: string
  /** 当前状态 */
  state: CreationState
  /** 获取指定 key 的状态值 */
  getState: GetState
  /** 设置状态值（支持重载） */
  setState: SetState
  /** 当前匹配的路由 */
  matches: UIMatch[]
  /** 当前匹配的路由 */
  currentMatch: UIMatch
}

/**
 * 创作状态上下文
 */
export const CreationStateContext = createContext<CreationStateContextType | null>(null)
