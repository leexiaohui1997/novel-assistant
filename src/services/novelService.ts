import { invoke } from '@tauri-apps/api/core'

import { Tag } from './tagService'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

export interface Novel {
  id: string
  title: string
  targetReader: string
  description: string
  coverImage?: string
  createdAt: string
  updatedAt: string
  /** 关联的标签列表，仅在 withTags=true 时有值 */
  tags: Tag[]
}

export interface CreateNovelParams {
  title: string
  targetReader: string
  tagIds: number[]
  description: string
}

export interface UpdateNovelParams {
  title: string
  targetReader: string
  tagIds: number[]
  description: string
}

interface FilterTagOptions {
  /** 是否包含标签 */
  withTags?: boolean
}

/**
 * 创建新小说
 */
export async function createNovel(params: CreateNovelParams): Promise<Novel> {
  try {
    logger.debug('调用创建小说 API:', params)

    const result = await invoke<Novel>('create_novel', {
      novel: params,
    })

    logger.debug('小说创建成功:', result)
    return result
  } catch (error) {
    logger.error('创建小说失败:', error)
    throw error
  }
}

/**
 * 获取所有小说（旧接口，保留兼容）
 */
export async function getNovels(options?: FilterTagOptions): Promise<Novel[]> {
  try {
    logger.debug('调用获取小说列表 API')

    const withTags = options?.withTags ?? false
    const result = await invoke<Novel[]>('get_novels', { withTags })

    logger.debug('获取到小说列表:', result.length, '条')
    return result
  } catch (error) {
    logger.error('获取小说列表失败:', error)
    throw error
  }
}

/**
 * 分页获取小说列表
 */
export async function getNovelsWithPagination(
  page: number,
  pageSize: number,
  filters?: FilterTagOptions,
): Promise<PaginatedResult<Novel>> {
  const withTags = filters?.withTags ?? false

  try {
    logger.debug('调用分页获取小说列表 API:', { page, pageSize, withTags })

    const result = await invoke<PaginatedResult<Novel>>('get_novels_with_pagination', {
      page,
      pageSize,
      withTags,
    })

    logger.debug('获取到小说列表:', result.data.length, '条，总数:', result.total)
    return result
  } catch (error) {
    logger.error('分页获取小说列表失败:', error)
    throw error
  }
}

/**
 * 根据 ID 获取小说信息
 */
export async function getNovelById(id: string, options?: FilterTagOptions): Promise<Novel> {
  try {
    logger.debug('调用获取小说详情 API:', id)

    const withTags = options?.withTags ?? false
    const result = await invoke<Novel>('get_novel_by_id', { id, withTags })

    logger.debug('获取到小说详情:', result)
    return result
  } catch (error) {
    logger.error('获取小说详情失败:', error)
    throw error
  }
}

/**
 * 更新小说信息
 */
export async function updateNovel(id: string, params: UpdateNovelParams): Promise<Novel> {
  try {
    logger.debug('调用更新小说 API:', id, params)

    const result = await invoke<Novel>('update_novel', { id, novel: params })

    logger.debug('小说更新成功:', result)
    return result
  } catch (error) {
    logger.error('更新小说失败:', error)
    throw error
  }
}

/**
 * 删除小说
 */
export async function deleteNovel(id: string): Promise<void> {
  try {
    logger.debug('调用删除小说 API:', id)

    await invoke<void>('delete_novel', { id })

    logger.debug('小说删除成功')
  } catch (error) {
    logger.error('删除小说失败:', error)
    throw error
  }
}
