import { useContext } from 'react'

import type { CreationStateContextType } from '@/providers/CreationStateContext'

import { CreationStateContext } from '@/providers/CreationStateContext'

/**
 * 使用创作状态的 Hook
 *
 * @returns 创作状态和操作方法
 * @throws 如果在 CreationStateProvider 外部使用会抛出错误
 */
export const useCreationState = (): CreationStateContextType => {
  const context = useContext(CreationStateContext)

  if (!context) {
    throw new Error('useCreationState 必须在 CreationStateProvider 内部使用')
  }

  return context
}
