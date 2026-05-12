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
 * 名词在某章节中的引用记录（含章节展示信息）
 *
 * 由后端 `ChapterTermRelationWithChapter` 适配而来，仅出现在
 * 「按名词 ID 反查章节关联列表」场景，因此 `chapterId` 必非空。
 *
 * `id` 取自后端 `relationId`，便于前端做 React key / 列表选中态。
 */
export type TermChapterRelation = {
  id: string
  novelId: string
  chapterId: string
  description: string
  createdAt: string
  updatedAt: string
  chapterTitle: string
  chapterSequence: number
  volumeSequence: number
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
 * 后端 `ChapterTermRelationWithChapter` 的原始结构（仅本文件内部使用）
 *
 * 与 Rust 侧 `serde(rename_all = "camelCase")` 输出严格对齐。
 */
interface BackendChapterTermRelationWithChapter {
  relationId: string
  novelId: string
  chapterId: string
  description: string | null
  createdAt: string
  updatedAt: string
  chapterTitle: string
  chapterSequence: number
  volumeSequence: number
}

/**
 * 后端 `PaginatedResult<T>` 的原始结构（仅本文件内部使用）
 *
 * 与 Rust 侧 `utils::pagination::PaginatedResult` 对齐：`data` 是当前页数据，
 * `total` 为命中总数；`page` / `pageSize` 由后端回显，前端通常无需关心。
 */
interface BackendPaginatedResult<T> {
  data: T[]
  total: number
  page: number
  pageSize: number
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
 * 适配后端 `NovelTerm` → 前端 `Term`。
 *
 * 统一处理：`description` 走 `normalizeText`、`termType` 走 `parseTermType`。
 */
function mapBackendTerm(backend: BackendNovelTerm): Term {
  return {
    id: backend.id,
    novelId: backend.novelId,
    termType: parseTermType(backend.termType),
    name: backend.name,
    description: normalizeText(backend.description),
    createdAt: backend.createdAt,
    updatedAt: backend.updatedAt,
  }
}

/**
 * 适配后端 `ChapterTermRelationWithTerm` → 前端 `ChapterTerm`。
 */
function mapRelationToChapterTerm(
  relation: BackendChapterTermRelationWithTerm,
  novelId: string,
): ChapterTerm {
  const term = mapBackendTerm(relation.term)

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
 * 适配后端 `ChapterTermRelationWithChapter` → 前端 `TermChapterRelation`。
 *
 * - `id` 取自 `relationId`
 * - `description` 走 `normalizeText` 归一化
 * - 其余字段直透
 */
function mapBackendTermChapterRelation(
  backend: BackendChapterTermRelationWithChapter,
): TermChapterRelation {
  return {
    id: backend.relationId,
    novelId: backend.novelId,
    chapterId: backend.chapterId,
    description: normalizeText(backend.description),
    createdAt: backend.createdAt,
    updatedAt: backend.updatedAt,
    chapterTitle: backend.chapterTitle,
    chapterSequence: backend.chapterSequence,
    volumeSequence: backend.volumeSequence,
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
 * 按名词 ID 反查其在各章节中的引用列表。
 *
 * 后端命令 `get_chapter_relations_by_term` 已保证：
 * - 仅返回 `chapter_id` 非空的关联记录
 * - 已级联补齐 `chapterTitle / chapterSequence / volumeSequence`
 * - 排序为「分卷序号 ASC + 章节序号 ASC」
 *
 * 调用方拿到的是开箱即用的渲染数据，无需再做二次查询。
 *
 * @param params.termId - 名词 ID（必填，falsy 时记录 warn 并返回空数组）
 * @returns 适配后的 `TermChapterRelation[]`，保留后端排序
 */
export async function getChapterRelationsByTerm(params: {
  termId: string
}): Promise<TermChapterRelation[]> {
  const { termId } = params

  if (!termId) {
    logger.warn('getChapterRelationsByTerm 调用被忽略：termId 不能为空')
    return []
  }

  try {
    const result = await invoke<BackendChapterTermRelationWithChapter[]>(
      'get_chapter_relations_by_term',
      { termId },
    )
    return result.map(mapBackendTermChapterRelation)
  } catch (error) {
    logger.error('查询名词关联章节列表失败:', error)
    throw error
  }
}

/**
 * 按小说 ID 一次性获取该小说下的全部名词列表（封装"全量获取"语义）。
 *
 * 内部固定走后端 `get_novel_terms` 命令并传 `page: 1`、`pageSize: 0`
 * （后端约定 `pageSize=0` 表示返回全部、不分页）；调用方无需关心
 * 分页或后端契约细节，拿到的就是干净的 `Term[]`。
 *
 * 适用场景：名词总览页、AI 上下文构建、重复名词校验等。
 *
 * @param params.novelId - 小说 ID（必填，falsy 时记录 warn 并返回空数组）
 * @param params.termType - 可选，按名词类型精确过滤
 * @param params.name - 可选，按名词名称精确过滤
 * @param params.descriptionKeyword - 可选，按描述模糊匹配（后端使用 LIKE）
 * @returns 适配后的 `Term[]`；不暴露 `total` 等分页元数据
 *
 * @example
 * // 全量
 * const terms = await getTermsByNovel({ novelId })
 *
 * @example
 * // 仅取某类名词 + 描述模糊匹配
 * const items = await getTermsByNovel({
 *   novelId,
 *   termType: TermType.Item,
 *   descriptionKeyword: '玄铁',
 * })
 */
export async function getTermsByNovel(params: {
  novelId: string
  termType?: TermType
  name?: string
  descriptionKeyword?: string
}): Promise<Term[]> {
  const { novelId, termType, name, descriptionKeyword } = params

  if (!novelId) {
    logger.warn('getTermsByNovel 调用被忽略：novelId 不能为空')
    return []
  }

  try {
    const result = await invoke<BackendPaginatedResult<BackendNovelTerm>>('get_novel_terms', {
      novelId,
      termType,
      name,
      descriptionKeyword,
      page: 1,
      pageSize: 0,
    })
    return result.data.map(mapBackendTerm)
  } catch (error) {
    logger.error('查询小说名词列表失败:', error)
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
