import { Typography } from 'antd'
import { useState } from 'react'

import CallLogsTable from './components/CallLogsTable'
import GranularityFilter from './components/GranularityFilter'
import ModelUsageTable from './components/ModelUsageTable'
import SummaryCards from './components/SummaryCards'

import type { Dayjs } from 'dayjs'

import { getRangeByGranularity, type TimeGranularity, type TimeRangeMs } from '@/utils/timeRange'

const { Title } = Typography

/** 获取页面初始时间范围（默认 30 天） */
function getInitialRange(): TimeRangeMs {
  const range = getRangeByGranularity('last30')
  return range ?? { startMs: 0, endMs: 0 }
}

/**
 * Tokens 看板页面
 */
const TokensDashboard: React.FC = () => {
  const [granularity, setGranularity] = useState<TimeGranularity>('last30')
  const [customRange, setCustomRange] = useState<[Dayjs, Dayjs] | null>(null)
  const [range, setRange] = useState<TimeRangeMs>(getInitialRange)

  const handleFilterChange = (next: TimeGranularity, nextCustom: [Dayjs, Dayjs] | null) => {
    setGranularity(next)
    setCustomRange(nextCustom)

    const computed =
      next === 'custom'
        ? nextCustom
          ? getRangeByGranularity('custom', { start: nextCustom[0], end: nextCustom[1] })
          : null
        : getRangeByGranularity(next)

    // 自定义未选完整区间时保留上一次 range，避免空白
    if (computed) setRange(computed)
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <Title level={4} style={{ margin: 0 }}>
          Tokens 看板
        </Title>
        <GranularityFilter
          granularity={granularity}
          customRange={customRange}
          onChange={handleFilterChange}
        />
      </div>

      <SummaryCards range={range} />
      <ModelUsageTable range={range} />
      <CallLogsTable range={range} />
    </div>
  )
}

export default TokensDashboard
