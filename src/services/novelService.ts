import { invoke } from '@tauri-apps/api/core'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

export interface Tag {
  id: number
  name: string
}

export interface Novel {
  id: string
  title: string
  targetReader: string
  description: string
  coverImage?: string
  createdAt: string
  updatedAt: string
}

export interface CreateNovelParams {
  title: string
  target_reader: string
  tag_ids: number[]
  description: string
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
export async function getNovels(): Promise<Novel[]> {
  try {
    logger.debug('调用获取小说列表 API')

    const result = await invoke<Novel[]>('get_novels')

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
): Promise<PaginatedResult<Novel>> {
  try {
    logger.debug('调用分页获取小说列表 API:', { page, pageSize })

    const result = await invoke<PaginatedResult<Novel>>('get_novels_with_pagination', {
      page,
      pageSize,
    })

    logger.debug('获取到小说列表:', result.data.length, '条，总数:', result.total)
    return result
  } catch (error) {
    logger.error('分页获取小说列表失败:', error)
    throw error
  }
}
