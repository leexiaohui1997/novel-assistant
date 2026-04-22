import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

/**
 * 目标读者类型
 */
export type TargetAudience = 'male' | 'female' | 'both'

/**
 * 标签类型
 */
export type TagType = 'main_category' | 'theme' | 'character' | 'plot'

/**
 * 标签类型中文映射
 */
export const TAG_TYPE_LABELS: Record<TagType, string> = {
  main_category: '主分类',
  theme: '主题',
  character: '角色',
  plot: '情节',
}

/**
 * 标签实体
 */
export interface Tag {
  id: number
  name: string
  tagType: TagType
  targetAudience: TargetAudience
  description?: string
  createdAt: string
}

/**
 * 根据目标读者获取标签列表
 */
export async function getTagsByAudience(
  targetAudience: Exclude<TargetAudience, 'both'>,
): Promise<Tag[]> {
  try {
    const tags = await invoke<Tag[]>('get_tags_by_audience', {
      targetAudience,
    })
    return tags
  } catch (error) {
    logger.error('获取标签失败:', error)
    throw error
  }
}

/**
 * 根据 ID 列表获取标签
 */
export async function getTagsByIds(ids: number[]): Promise<Tag[]> {
  if (ids.length === 0) {
    return []
  }

  try {
    const tags = await invoke<Tag[]>('get_tags_by_ids', {
      ids,
    })
    return tags
  } catch (error) {
    logger.error('获取标签失败:', error)
    throw error
  }
}
