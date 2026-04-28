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
  /** 是否可删除（仅分卷下最后一个非草稿章节为 true） */
  deletable: boolean
}

/** 简化分卷实体 */
export interface SimpleVolume {
  id: number
  name: string
  sequence: number
}

/** 分卷实体 */
export interface Volume extends SimpleVolume {
  novelId: string
  createdAt: string
  updatedAt: string
}

/** 章节分页查询参数 */
export interface ChapterQuery {
  /** 分卷业务序号（非主键 id）；草稿模式下会被后端忽略 */
  volumeSequence?: number
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

/** 创建章节请求 payload */
export interface NewChapterPayload {
  title: string
  content: string
  /** 分卷 ID（可选，未传表示无分卷关联） */
  volumeId?: number
  /** 章节序号（可选）：未传时后端落库为草稿（sequence=-1），传入则作为正式章节序号 */
  sequence?: number
}

/** 更新章节请求 payload */
export interface UpdateChapterPayload {
  title: string
  content: string
  /** 章节序号（可选） */
  sequence?: number
  /** 分卷 ID（可选） */
  volumeId?: number
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
 * 创建章节
 *
 * @param novelId - 所属小说 ID
 * @param payload - 章节数据（标题、正文、所属分卷 ID）
 * @returns 创建成功的章节
 */
export async function createChapter(novelId: string, payload: NewChapterPayload): Promise<Chapter> {
  try {
    logger.debug('调用创建章节 API:', { novelId, payload })

    const result = await invoke<Chapter>('create_chapter', { novelId, payload })

    logger.debug('章节创建成功:', result.id)
    return result
  } catch (error) {
    logger.error('创建章节失败:', error)
    throw error
  }
}

/**
 * 更新章节
 *
 * @param chapterId - 章节 ID
 * @param payload - 更新数据（标题、正文、序号、分卷 ID）
 * @returns 更新后的章节
 */
export async function updateChapter(
  chapterId: string,
  payload: UpdateChapterPayload,
): Promise<Chapter> {
  try {
    logger.debug('调用更新章节 API:', { chapterId, payload })

    const result = await invoke<Chapter>('update_chapter', { chapterId, payload })

    logger.debug('章节更新成功:', result.id)
    return result
  } catch (error) {
    logger.error('更新章节失败:', error)
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
export function buildDefaultVolume(novelId: string): Volume {
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
export function resolveVolumesWithDefault<T extends SimpleVolume | Volume>(
  list: T[],
  novelId: string,
): T[] {
  if (list.length === 0) {
    return [buildDefaultVolume(novelId) as T]
  }
  return list
}

/** 章节历史版本实体（后端返回结构，字段 camelCase） */
export interface ChapterVersion {
  id: string
  chapterId: string
  title: string
  content: string
  wordCount: number
  /** 该版本文章的更新时间（即快照时刻章节的 updatedAt） */
  savedAt: string
  /** 快照记录的插入时间 */
  createdAt: string
}

/**
 * 查询指定章节的全部历史版本（按 savedAt 倒序）
 *
 * @param chapterId - 章节 ID
 * @returns 历史版本列表
 */
export async function getChapterVersions(chapterId: string): Promise<ChapterVersion[]> {
  try {
    logger.debug('调用查询章节历史版本 API:', chapterId)

    const result = await invoke<ChapterVersion[]>('get_chapter_versions', { chapterId })

    logger.debug('获取到章节历史版本:', result.length, '条')
    return result
  } catch (error) {
    logger.error('查询章节历史版本失败:', error)
    throw error
  }
}
