import { createContext } from 'react'

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
  /** 当前状态 */
  state: CreationState
  /** 获取指定 key 的状态值 */
  getState: GetState
  /** 设置状态值（支持重载） */
  setState: SetState
}

/**
 * 创作状态上下文
 */
export const CreationStateContext = createContext<CreationStateContextType | null>(null)
