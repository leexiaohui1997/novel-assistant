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
