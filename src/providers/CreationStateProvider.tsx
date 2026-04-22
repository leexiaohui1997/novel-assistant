import { useEffect, useCallback, useState, useMemo } from 'react'
import { useMatches } from 'react-router-dom'

import { CreationStateContext } from './CreationStateContext'

import type { CreationStateContextType, GetState, SetState } from './CreationStateContext'

import PageLoading from '@/components/PageLoading'
import {
  getCreationState,
  getDefaultCreationState,
  upsertCreationState,
  type CreationState,
} from '@/services/creationStateService'
import { getNovelById, Novel } from '@/services/novelService'
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
  const [loading, setLoading] = useState(true)
  const matches = useMatches()
  const currentMatch = useMemo(() => matches[matches.length - 1], [matches])
  const [novelInfo, setNovelInfo] = useState<Novel>()

  /**
   * 刷新小说信息（更新后重新拉取）
   */
  const refreshNovelInfo = useCallback(async () => {
    const record = await getNovelById(novelId, { withTags: true })
    setNovelInfo(record)
  }, [novelId])

  useEffect(() => {
    // 初始化状态
    const initState = async () => {
      const record = await getCreationState(novelId)
      setStateValue(record || getDefaultCreationState())
    }

    // 初始化小说信息
    const initNovelInfo = async () => {
      await refreshNovelInfo()
    }

    Promise.all([initState(), initNovelInfo()]).then(() => {
      setLoading(false)
    })
  }, [novelId, refreshNovelInfo])

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

  const contextValue: CreationStateContextType = useMemo(
    () => ({
      novelId,
      novelInfo: novelInfo as Novel,
      refreshNovelInfo,
      state,
      getState,
      setState,
      matches,
      currentMatch,
    }),
    [novelId, novelInfo, refreshNovelInfo, state, getState, setState, matches, currentMatch],
  )

  if (loading) {
    return <PageLoading />
  }

  return (
    <CreationStateContext.Provider value={contextValue}>{children}</CreationStateContext.Provider>
  )
}
