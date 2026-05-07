import { Descriptions, Typography } from 'antd'

import { AiCallLogItem } from '@/services/tokensDashboardService'
import { formatDateTime } from '@/utils/date'

const { Text } = Typography

export type AiCallLogBasicInfoProps = {
  logInfo: AiCallLogItem
}

export function AiCallLogBasicInfo({ logInfo }: AiCallLogBasicInfoProps) {
  return (
    <Descriptions
      title="基本信息"
      column={1}
      items={[
        {
          label: '日志ID',
          children: <Text>{logInfo.id}</Text>,
        },
        {
          label: '供应商',
          children: <Text>{logInfo.providerName}</Text>,
        },
        {
          label: '模型',
          children: <Text>{logInfo.modelName}</Text>,
        },
        {
          label: '请求时间',
          children: <Text>{formatDateTime(logInfo.callTime)}</Text>,
        },
        {
          label: '结束时间',
          children: <Text>{formatDateTime(logInfo.createdAt)}</Text>,
        },
        {
          label: '耗时',
          children: <Text>{logInfo.durationMs} ms</Text>,
        },
        {
          label: '总 tokens',
          children: <Text>{logInfo.totalTokens}</Text>,
        },
      ]}
      size="small"
      bordered
    />
  )
}
