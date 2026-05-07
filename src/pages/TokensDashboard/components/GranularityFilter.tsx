import { DatePicker, Segmented, Space } from 'antd'
import dayjs, { type Dayjs } from 'dayjs'
import { useMemo } from 'react'

import type { TimeGranularity } from '@/utils/timeRange'

/** 粒度选项配置 */
const GRANULARITY_OPTIONS: Array<{ label: string; value: TimeGranularity }> = [
  { label: '今天', value: 'today' },
  { label: '昨天', value: 'yesterday' },
  { label: '本周', value: 'week' },
  { label: '本月', value: 'month' },
  { label: '30 天', value: 'last30' },
  { label: '自定义', value: 'custom' },
]

export interface GranularityFilterProps {
  granularity: TimeGranularity
  customRange: [Dayjs, Dayjs] | null
  onChange: (granularity: TimeGranularity, customRange: [Dayjs, Dayjs] | null) => void
}

/**
 * 时间粒度筛选器
 */
const GranularityFilter: React.FC<GranularityFilterProps> = ({
  granularity,
  customRange,
  onChange,
}) => {
  const handleGranularityChange = (value: TimeGranularity) => {
    // 切换非自定义粒度时清空 RangePicker 值
    if (value !== 'custom') {
      onChange(value, null)
      return
    }
    onChange(value, customRange)
  }

  const handleRangeChange = (values: unknown) => {
    const tuple = values as [Dayjs | null, Dayjs | null] | null
    if (!tuple?.[0] || !tuple?.[1]) {
      onChange('custom', null)
      return
    }
    onChange('custom', [tuple[0], tuple[1]])
  }

  const rangeValue = useMemo<[Dayjs, Dayjs] | undefined>(() => {
    return customRange ?? undefined
  }, [customRange])

  return (
    <Space size="middle" wrap>
      <Segmented<TimeGranularity>
        options={GRANULARITY_OPTIONS}
        value={granularity}
        onChange={handleGranularityChange}
      />
      {granularity === 'custom' && (
        <DatePicker.RangePicker
          value={rangeValue}
          onChange={handleRangeChange}
          maxDate={dayjs().endOf('day')}
          allowClear
        />
      )}
    </Space>
  )
}

export default GranularityFilter
