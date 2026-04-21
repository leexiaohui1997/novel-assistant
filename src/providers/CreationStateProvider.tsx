import { useEffect, useCallback, useState } from 'react'

import { CreationStateContext } from './CreationStateContext'

import type { CreationStateContextType, GetState, SetState } from './CreationStateContext'

import {
  getCreationState,
  getDefaultCreationState,
  upsertCreationState,
  type CreationState,
} from '@/services/creationStateService'
import { logger } from '@/utils/logger'

/**
 * 创作状态 Provider 属性
 */
interface CreationStateProviderProps {
  novelId: string
  children: React.ReactNode
}

/**
 * 创作状态 Provider 组件
 *
 * @param novelId - 小说 ID
 * @param children - 子组件
 */
export const CreationStateProvider: React.FC<CreationStateProviderProps> = ({
  novelId,
  children,
}) => {
  const [state, setStateValue] = useState<CreationState>(getDefaultCreationState())

  // 初始化状态
  useEffect(() => {
    const initState = async () => {
      try {
        const record = await getCreationState(novelId)
        setStateValue(record || getDefaultCreationState())
      } catch (error) {
        logger.error('获取创作状态记录失败:', error)
      }
    }

    initState()
  }, [novelId])

  // 同步状态到数据库（防抖 500ms）
  useEffect(() => {
    if (Object.keys(state).length === 0) return

    const timer = setTimeout(async () => {
      try {
        await upsertCreationState(novelId)
      } catch (error) {
        logger.error('持久化创作状态失败:', error)
      }
    }, 500)

    return () => clearTimeout(timer)
  }, [novelId, state])

  /**
   * 获取指定 key 的状态值
   */
  const getState: GetState = useCallback(
    (key) => {
      return state[key]
    },
    [state],
  )

  /**
   * 设置状态值
   */
  /**
   * 设置状态值（支持重载）
   */
  const setState: SetState = useCallback(
    (keyOrObj: string | Partial<CreationState>, value?: unknown) => {
      if (typeof keyOrObj === 'string') {
        setStateValue((prev) => ({ ...prev, [keyOrObj]: value }))
      } else {
        setStateValue((prev) => ({ ...prev, ...keyOrObj }))
      }
    },
    [],
  )

  const contextValue: CreationStateContextType = {
    state,
    getState,
    setState,
  }

  return (
    <CreationStateContext.Provider value={contextValue}>{children}</CreationStateContext.Provider>
  )
}
