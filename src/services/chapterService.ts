import { invoke } from '@tauri-apps/api/core'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

/** 章节实体（后端返回结构，字段 camelCase） */
export interface Chapter {
  id: string
  novelId: string
  title: string
  content: string
  wordCount: number
  sequence: number
  createdAt: string
  updatedAt: string
}

/** 分卷实体 */
export interface Volume {
  id: number
  novelId: string
  name: string
  sequence: number
  createdAt: string
  updatedAt: string
}

/** 章节分页查询参数 */
export interface ChapterQuery {
  /** 分卷 ID；草稿模式下会被后端忽略 */
  volumeId?: number
  /** 是否草稿模式 */
  isDraft?: boolean
  /** 排序字段：sequence / title / createdAt / updatedAt */
  sortField?: 'sequence' | 'title' | 'createdAt' | 'updatedAt'
  /** 排序顺序 */
  sortOrder?: 'asc' | 'desc'
}

/** 批量编辑分卷请求项 */
export interface VolumeUpsert {
  /** 分卷 ID；不传表示新增 */
  id?: number
  name: string
  sequence: number
}

/**
 * 分页查询章节
 */
export async function getChaptersWithPagination(
  novelId: string,
  page: number,
  pageSize: number,
  query: ChapterQuery = {},
): Promise<PaginatedResult<Chapter>> {
  try {
    logger.debug('调用分页查询章节 API:', { novelId, page, pageSize, query })

    const result = await invoke<PaginatedResult<Chapter>>('get_chapters_with_pagination', {
      novelId,
      page,
      pageSize,
      query,
    })

    logger.debug('获取到章节列表:', result.data.length, '条，总数:', result.total)
    return result
  } catch (error) {
    logger.error('分页查询章节失败:', error)
    throw error
  }
}

/**
 * 删除章节
 */
export async function deleteChapter(novelId: string, chapterId: string): Promise<void> {
  try {
    logger.debug('调用删除章节 API:', { novelId, chapterId })

    await invoke<void>('delete_chapter', { novelId, chapterId })

    logger.debug('章节删除成功')
  } catch (error) {
    logger.error('删除章节失败:', error)
    throw error
  }
}

/**
 * 查询小说下全部分卷（按 sequence 升序）
 */
export async function getVolumes(novelId: string): Promise<Volume[]> {
  try {
    logger.debug('调用查询分卷列表 API:', novelId)

    const result = await invoke<Volume[]>('get_volumes', { novelId })

    logger.debug('获取到分卷列表:', result.length, '条')
    return result
  } catch (error) {
    logger.error('查询分卷列表失败:', error)
    throw error
  }
}

/**
 * 批量编辑分卷（事务写入）
 *
 * @param novelId - 小说 ID
 * @param payload - 完整分卷列表；含 id 视为更新，无 id 视为新增，已有但不在列表中的视为删除
 */
export async function batchUpdateVolumes(
  novelId: string,
  payload: VolumeUpsert[],
): Promise<Volume[]> {
  try {
    logger.debug('调用批量编辑分卷 API:', { novelId, count: payload.length })

    const result = await invoke<Volume[]>('batch_update_volumes', { novelId, payload })

    logger.debug('批量编辑分卷成功，返回分卷数:', result.length)
    return result
  } catch (error) {
    logger.error('批量编辑分卷失败:', error)
    throw error
  }
}

/** 默认分卷序号（虚拟默认分卷使用） */
export const DEFAULT_VOLUME_SEQUENCE = 1

/** 默认分卷名称 */
export const DEFAULT_VOLUME_NAME = '默认'

/**
 * 构造虚拟默认分卷
 *
 * 当小说下无任何分卷时使用，用于前端下拉选择器等场景。
 * 虚拟分卷的 id 为 0（真实分卷不会占用 0），用于区分真实分卷。
 */
function buildDefaultVolume(novelId: string): Volume {
  return {
    id: 0,
    novelId,
    name: DEFAULT_VOLUME_NAME,
    sequence: DEFAULT_VOLUME_SEQUENCE,
    createdAt: '',
    updatedAt: '',
  }
}

/**
 * 分卷列表规整：为空时虚拟化一个默认分卷
 *
 * @param list - 后端返回的分卷列表
 * @param novelId - 所属小说 ID
 * @returns 至少包含一项的分卷列表
 *
 * @example
 * const volumes = await getVolumes(novelId)
 * const displayList = resolveVolumesWithDefault(volumes, novelId)
 */
export function resolveVolumesWithDefault(list: Volume[], novelId: string): Volume[] {
  if (list.length === 0) {
    return [buildDefaultVolume(novelId)]
  }
  return list
}
