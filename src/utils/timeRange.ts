import dayjs, { type Dayjs } from 'dayjs'
import isoWeek from 'dayjs/plugin/isoWeek'

dayjs.extend(isoWeek)

/** 时间粒度枚举 */
export type TimeGranularity = 'today' | 'yesterday' | 'week' | 'month' | 'last30' | 'custom'

/** 时间范围（毫秒时间戳） */
export interface TimeRangeMs {
  startMs: number
  endMs: number
}

/** 自定义粒度下的用户输入（开始/结束日期） */
export interface CustomRangeInput {
  start: Dayjs
  end: Dayjs
}

/**
 * 计算「今日」范围：[今日 00:00:00, 今日 23:59:59.999]
 */
function rangeToday(): TimeRangeMs {
  const now = dayjs()
  return {
    startMs: now.startOf('day').valueOf(),
    endMs: now.endOf('day').valueOf(),
  }
}

/**
 * 计算「昨天」范围：[昨日 00:00:00, 昨日 23:59:59.999]
 */
function rangeYesterday(): TimeRangeMs {
  const yesterday = dayjs().subtract(1, 'day')
  return {
    startMs: yesterday.startOf('day').valueOf(),
    endMs: yesterday.endOf('day').valueOf(),
  }
}

/**
 * 计算「本周」范围（周一为起点）
 */
function rangeWeek(): TimeRangeMs {
  const now = dayjs()
  return {
    startMs: now.startOf('isoWeek').valueOf(),
    endMs: now.endOf('isoWeek').valueOf(),
  }
}

/**
 * 计算「本月」范围
 */
function rangeMonth(): TimeRangeMs {
  const now = dayjs()
  return {
    startMs: now.startOf('month').valueOf(),
    endMs: now.endOf('month').valueOf(),
  }
}

/**
 * 计算「滚动 30 天」范围：[当前 − 30 天, 当前]
 */
function rangeLast30(): TimeRangeMs {
  const now = dayjs()
  return {
    startMs: now.subtract(30, 'day').valueOf(),
    endMs: now.valueOf(),
  }
}

/**
 * 将自定义日期区间转换为毫秒区间（扩展到当日 00:00 ~ 23:59:59.999）
 */
function rangeCustom(input: CustomRangeInput): TimeRangeMs {
  return {
    startMs: input.start.startOf('day').valueOf(),
    endMs: input.end.endOf('day').valueOf(),
  }
}

/**
 * 根据时间粒度计算时间范围（毫秒时间戳）
 *
 * @param granularity - 粒度：today/yesterday/week/month/last30/custom
 * @param custom - 自定义粒度下必传；其他粒度下忽略
 * @returns 时间范围毫秒时间戳；非法输入返回 `null`
 *
 * @example
 * getRangeByGranularity('today')
 * getRangeByGranularity('custom', { start: dayjs('2026-01-01'), end: dayjs('2026-01-31') })
 */
export function getRangeByGranularity(
  granularity: TimeGranularity,
  custom?: CustomRangeInput,
): TimeRangeMs | null {
  if (granularity !== 'custom') {
    const builders: Record<Exclude<TimeGranularity, 'custom'>, () => TimeRangeMs> = {
      today: rangeToday,
      yesterday: rangeYesterday,
      week: rangeWeek,
      month: rangeMonth,
      last30: rangeLast30,
    }
    return builders[granularity]()
  }

  if (!custom?.start?.isValid() || !custom?.end?.isValid()) return null
  return rangeCustom(custom)
}
