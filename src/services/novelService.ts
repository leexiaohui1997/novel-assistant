import { invoke } from '@tauri-apps/api/core'

import { Tag } from './tagService'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

/**
 * 小说统计信息（仅在 withStats=true 时有值）
 */
export interface NovelStats {
  /** 所属小说 ID */
  novelId: string
  /** 非草稿章节数 */
  chapterCount: number
  /** 非草稿章节的总字数 */
  totalWordCount: number
  /** 最近更新时间（ISO 字符串） */
  lastUpdatedAt: string | null
  /** 最近更新章节所属分卷序号 */
  lastUpdatedVolumeSequence: number | null
  /** 最近更新章节自身序号 */
  lastUpdatedChapterSequence: number | null
  /** 最近更新章节的标题 */
  lastUpdatedChapterTitle: string | null
}

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
  /** 统计信息，仅在 withStats=true 时有值 */
  stats?: NovelStats
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
  /** 是否包含统计信息（章节数、字数、最近更新等） */
  withStats?: boolean
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
  const withStats = filters?.withStats ?? false

  try {
    logger.debug('调用分页获取小说列表 API:', { page, pageSize, withTags, withStats })

    const result = await invoke<PaginatedResult<Novel>>('get_novels_with_pagination', {
      page,
      pageSize,
      withTags,
      withStats,
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
