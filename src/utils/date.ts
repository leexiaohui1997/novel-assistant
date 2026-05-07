import dayjs from 'dayjs'

/** 默认日期时间格式（精确到秒） */
export const DEFAULT_DATETIME_FORMAT = 'YYYY-MM-DD HH:mm:ss'

/** 默认日期格式（仅日期） */
export const DEFAULT_DATE_FORMAT = 'YYYY-MM-DD'

/**
 * 格式化日期时间
 *
 * 对 `null` / `undefined` / 空串 / 非法值 直接返回 `fallback`（默认空串）。
 *
 * @param value - 可被 dayjs 解析的日期（ISO 字符串、Date、时间戳等）
 * @param format - 输出格式，默认 `YYYY-MM-DD HH:mm:ss`
 * @param fallback - 无效值兜底，默认 `''`
 *
 * @example
 * formatDateTime('2026-04-27T08:48:08.845Z') // '2026-04-27 16:48:08'
 * formatDateTime(null) // ''
 */
export function formatDateTime(
  value: string | number | Date | null | undefined,
  format: string = DEFAULT_DATETIME_FORMAT,
  fallback: string = '',
): string {
  if (value === null || value === undefined || value === '') return fallback
  const d = dayjs(value)
  return d.isValid() ? d.format(format) : fallback
}

/**
 * 格式化为日期（不含时间）
 *
 * @param value - 可被 dayjs 解析的日期
 * @param fallback - 无效值兜底，默认 `''`
 *
 * @example
 * formatDate('2026-04-27T08:48:08.845Z') // '2026-04-27'
 */
export function formatDate(
  value: string | number | Date | null | undefined,
  fallback: string = '',
): string {
  return formatDateTime(value, DEFAULT_DATE_FORMAT, fallback)
}

/** 相对时间阈值常量（毫秒） */
const MS_PER_MINUTE = 60 * 1000
const MS_PER_HOUR = 60 * MS_PER_MINUTE
const MS_PER_DAY = 24 * MS_PER_HOUR
const MAX_RELATIVE_DAYS = 30

/**
 * 将毫秒差转换为中文相对时间描述（< 30 天范围内）
 * 超过 30 天返回 null，由上层回退到绝对日期
 */
function diffToRelativeText(diffMs: number): string | null {
  if (diffMs < MS_PER_MINUTE) return '刚刚'
  if (diffMs < MS_PER_HOUR) return `${Math.floor(diffMs / MS_PER_MINUTE)} 分钟前`
  if (diffMs < MS_PER_DAY) return `${Math.floor(diffMs / MS_PER_HOUR)} 小时前`
  const days = Math.floor(diffMs / MS_PER_DAY)
  if (days < MAX_RELATIVE_DAYS) return `${days} 天前`
  return null
}

/**
 * 格式化为中文相对时间
 *
 * 规则：
 * - 1 分钟内 → "刚刚"
 * - 1 小时内 → "x 分钟前"
 * - 24 小时内 → "x 小时前"
 * - 30 天内 → "x 天前"
 * - 超过 30 天 → 绝对日期（默认 `YYYY-MM-DD`）
 *
 * 未使用 dayjs relativeTime 插件，因其默认文案为英文且边界条件需自定义。
 *
 * @param value - 可被 dayjs 解析的日期（ISO 字符串、Date、时间戳等）
 * @param fallback - 无效值兜底，默认 `''`
 *
 * @example
 * formatRelativeTime('2026-05-07T10:20:00+08:00') // '3 小时前'
 * formatRelativeTime(null) // ''
 */
export function formatRelativeTime(
  value: string | number | Date | null | undefined,
  fallback: string = '',
): string {
  if (value === null || value === undefined || value === '') return fallback
  const target = dayjs(value)
  if (!target.isValid()) return fallback

  const diffMs = dayjs().diff(target)
  if (diffMs < 0) return '刚刚'

  const relative = diffToRelativeText(diffMs)
  return relative ?? target.format(DEFAULT_DATE_FORMAT)
}
