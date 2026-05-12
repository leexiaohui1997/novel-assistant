import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

/**
 * 小说名词类型枚举
 *
 * 字面量值（snake_case）与后端 `TermType` 序列化形式严格一致，
 * 顺序与后端 `ALL_TERM_TYPES` 对齐，不可随意调整。
 */
export enum TermType {
  Character = 'character',
  Location = 'location',
  Faction = 'faction',
  Item = 'item',
  Skill = 'skill',
  Race = 'race',
  Era = 'era',
  Culture = 'culture',
  Emotion = 'emotion',
}

/**
 * 名词类型 → 中文展示标签
 *
 * 文案与后端 `TermType::label()` 完全一致。
 */
export const TERM_TYPE_LABELS: Record<TermType, string> = {
  [TermType.Character]: '人物',
  [TermType.Location]: '地域 & 场景',
  [TermType.Faction]: '势力 & 组织',
  [TermType.Item]: '物品 & 道具',
  [TermType.Skill]: '功法 & 技能 & 术法',
  [TermType.Race]: '生灵 & 种族',
  [TermType.Era]: '时间 & 纪元 & 设定',
  [TermType.Culture]: '文化 & 典籍 & 规则',
  [TermType.Emotion]: '情感 & 专属代称',
}

/**
 * 所有名词类型，按声明顺序导出，便于 UI 遍历。
 */
export const ALL_TERM_TYPES: readonly TermType[] = [
  TermType.Character,
  TermType.Location,
  TermType.Faction,
  TermType.Item,
  TermType.Skill,
  TermType.Race,
  TermType.Era,
  TermType.Culture,
  TermType.Emotion,
] as const

/**
 * 小说名词主表实体
 *
 * 注意：`description` 在前端始终为字符串（空字符串表示无描述）。
 */
export type Term = {
  id: string
  novelId: string
  termType: TermType
  name: string
  description: string
  createdAt: string
  updatedAt: string
}

/**
 * 章节-名词关联（含名词详情）
 *
 * `id` 取自后端 `relationId`，便于前端 React key / 选中态使用。
 */
export type ChapterTerm = {
  id: string
  novelId: string
  chapterId?: string
  description: string
  createdAt: string
  updatedAt: string
  term: Term
}

/**
 * AI 生成的名词
 */
export type AiTerm = {
  id?: string
  name: string
  term_type: TermType
  description: string
}

/**
 * 后端名词主表的原始结构（仅本文件内部使用）
 */
interface BackendNovelTerm {
  id: string
  novelId: string
  termType: string
  name: string
  description: string | null
  createdAt: string
  updatedAt: string
}

/**
 * 后端 `ChapterTermRelationWithTerm` 的原始结构（仅本文件内部使用）
 */
interface BackendChapterTermRelationWithTerm {
  relationId: string
  chapterId?: string | null
  description: string | null
  createdAt: string
  updatedAt: string
  term: BackendNovelTerm
}

/**
 * 文本归一化：null/undefined/纯空白 一律返回空字符串。
 */
function normalizeText(value: string | null | undefined): string {
  if (value === null || value === undefined) {
    return ''
  }
  return value.trim() === '' ? '' : value
}

/**
 * 解析后端字符串为 TermType；命中已知枚举值则直接返回，
 * 否则告警并以 `as TermType` 透传，兼容后端未来扩展。
 */
function parseTermType(raw: string): TermType {
  const matched = ALL_TERM_TYPES.find((t) => t === raw)
  if (matched) {
    return matched
  }
  logger.warn('未知的 TermType 值，已透传:', raw)
  return raw as TermType
}

/**
 * 适配后端 `ChapterTermRelationWithTerm` → 前端 `ChapterTerm`。
 */
function mapRelationToChapterTerm(
  relation: BackendChapterTermRelationWithTerm,
  novelId: string,
): ChapterTerm {
  const backendTerm = relation.term
  const term: Term = {
    id: backendTerm.id,
    novelId: backendTerm.novelId,
    termType: parseTermType(backendTerm.termType),
    name: backendTerm.name,
    description: normalizeText(backendTerm.description),
    createdAt: backendTerm.createdAt,
    updatedAt: backendTerm.updatedAt,
  }

  return {
    id: relation.relationId,
    novelId,
    chapterId: relation.chapterId ?? undefined,
    description: normalizeText(relation.description),
    createdAt: relation.createdAt,
    updatedAt: relation.updatedAt,
    term,
  }
}

/**
 * 查询小说（可选指定章节）下的名词关联列表。
 *
 * @param params.novelId  - 小说 ID（必填）
 * @param params.chapterId - 章节 ID（可选；未传则返回该小说下全部章节关联）
 * @returns 适配后的 ChapterTerm 列表，保持后端原始顺序
 */
export async function getChapterTerms(params: {
  novelId: string
  chapterId?: string
}): Promise<ChapterTerm[]> {
  const { novelId, chapterId } = params

  if (!novelId) {
    logger.warn('getChapterTerms 调用被忽略：novelId 不能为空')
    return []
  }

  try {
    const result = await invoke<BackendChapterTermRelationWithTerm[]>('get_chapter_terms', {
      novelId,
      chapterId,
    })
    return result.map((relation) => mapRelationToChapterTerm(relation, novelId))
  } catch (error) {
    logger.error('查询章节名词列表失败:', error)
    throw error
  }
}

/**
 * 后端 `update_chapter_terms` 命令的单条名词入参（仅本文件内部使用）
 *
 * 字段命名遵循后端 `TermInput` 结构（snake_case）。
 */
interface BackendTermInput {
  id: string | null
  name: string
  term_type: string
  description: string | null
}

/**
 * 把前端 `AiTerm` 适配为后端 `TermInput` 形态。
 *
 * - `id` 为空串/undefined 透传为 `null`（后端识别为新名词分支）
 * - `description` 为空串/undefined 透传为 `null`（与后端 `Option<String>` 对齐）
 */
function toBackendTermInput(item: AiTerm): BackendTermInput {
  const id = item.id && item.id.trim() !== '' ? item.id : null
  const description = item.description && item.description.trim() !== '' ? item.description : null
  return {
    id,
    name: item.name,
    term_type: item.term_type,
    description,
  }
}

/**
 * 全量更新指定章节的名词关联列表（事务化覆盖语义）。
 *
 * 后端会执行：新名词 INSERT → 旧关联 DELETE → 新关联 INSERT，
 * 保存范围与 `terms` 完全一致——传什么列表，章节就只剩这个列表的关联。
 *
 * @param params.novelId - 小说 ID（必填）
 * @param params.chapterId - 章节 ID（可选；未传时关联落 `chapter_id IS NULL`）
 * @param params.terms - 章节最终保留的名词列表（含 AI 新增 + 已有名词）
 * @returns 调用成功时 resolve 为 void；本函数不消费后端返回数据，
 *          调用方应通过 `useRefresh.refresh()` 重拉最新数据
 */
export async function updateChapterTerms(params: {
  novelId: string
  chapterId?: string
  terms: AiTerm[]
}): Promise<void> {
  const { novelId, chapterId, terms } = params

  try {
    await invoke('update_chapter_terms', {
      input: {
        novel_id: novelId,
        chapter_id: chapterId ?? null,
        terms: terms.map(toBackendTermInput),
      },
    })
  } catch (error) {
    logger.error('保存章节名词列表失败:', error)
    throw error
  }
}
