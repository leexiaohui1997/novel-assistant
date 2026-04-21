import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

/**
 * 创作状态类型
 * 当前为空对象，后续可根据需要扩展字段
 */
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface CreationState {}

/**
 * 创作状态记录
 *
 * 对应后端 creation_states 表的查询结果，继承自 CreationState
 */
export interface CreationStateRecord extends CreationState {
  /** 小说 ID */
  novelId: string
  /** 创建时间 */
  createdAt: string
  /** 更新时间 */
  updatedAt: string
}

/**
 * 获取指定小说的创作状态记录
 *
 * @param novelId - 小说 ID
 * @returns 创作状态记录，如果不存在则返回 null
 */
export const getCreationState = async (novelId: string): Promise<CreationStateRecord | null> => {
  try {
    const result = await invoke<CreationStateRecord | null>('get_creation_state', {
      novelId,
    })
    return result
  } catch (error) {
    logger.error('获取创作状态失败:', error)
    return null
  }
}

/**
 * 创建或更新指定小说的创作状态记录
 *
 * @param novelId - 小说 ID
 */
export const upsertCreationState = async (novelId: string): Promise<void> => {
  try {
    await invoke('upsert_creation_state', { novelId })
  } catch (error) {
    logger.error('创建或更新创作状态失败:', error)
    throw error
  }
}

/**
 * 获取默认的创作状态
 *
 * @returns 默认的创作状态
 */
export const getDefaultCreationState = (): CreationState => ({})
