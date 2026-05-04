import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

export type ChapterOutline = {
  id: number
  novelId: string
  chapterId?: string
  positioning?: string
  plot?: string
  createdAt: string
  updatedAt: string
}

/**
 * 编辑章节大纲
 * @param novelId 小说ID
 * @param chapterId 章节ID（可选，为空时表示整本小说的通用大纲）
 * @param positioning 定位内容
 * @param plot 剧情内容（可选）
 */
export async function editChapterOutline(
  novelId: string,
  chapterId: string | undefined,
  positioning: string,
  plot?: string,
): Promise<ChapterOutline> {
  try {
    logger.debug('调用编辑章节大纲 API:', { novelId, chapterId, positioning, plot })

    const result = await invoke<ChapterOutline>('edit_chapter_outline', {
      input: {
        novel_id: novelId,
        chapter_id: chapterId,
        positioning,
        plot,
      },
    })

    logger.debug('章节大纲编辑成功:', result)
    return result
  } catch (error) {
    logger.error('编辑章节大纲失败:', error)
    throw error
  }
}

/**
 * 获取章节大纲
 * @param novelId 小说ID
 * @param chapterId 章节ID（可选，为空时表示获取整本小说的通用大纲）
 * @returns 大纲信息，如果不存在则返回 null
 */
export async function getChapterOutline(
  novelId: string,
  chapterId: string | undefined,
): Promise<ChapterOutline | null> {
  try {
    logger.debug('调用获取章节大纲 API:', { novelId, chapterId })

    const result = await invoke<ChapterOutline | null>('get_chapter_outline', {
      input: {
        novel_id: novelId,
        chapter_id: chapterId,
      },
    })

    logger.debug('获取章节大纲成功:', result)
    return result
  } catch (error) {
    logger.error('获取章节大纲失败:', error)
    throw error
  }
}
