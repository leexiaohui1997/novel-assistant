import { invoke } from '@tauri-apps/api/core'

export type ChapterOutline = {
  id: number
  novelId: string
  chapterId?: string
  positioning: string
  createdAt: string
  updatedAt: string
}

/**
 * 编辑章节大纲
 * @param novelId 小说ID
 * @param chapterId 章节ID（可选，为空时表示整本小说的通用大纲）
 * @param positioning 大纲内容
 */
export async function editChapterOutline(
  novelId: string,
  chapterId: string | undefined,
  positioning: string,
): Promise<ChapterOutline> {
  return invoke<ChapterOutline>('edit_chapter_outline', {
    input: {
      novel_id: novelId,
      chapter_id: chapterId,
      positioning,
    },
  })
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
  return invoke<ChapterOutline | null>('get_chapter_outline', {
    input: {
      novel_id: novelId,
      chapter_id: chapterId,
    },
  })
}
