import { App, Card, Table } from 'antd'
import { useEffect, useMemo, useRef, useState } from 'react'

import type { ColumnsType } from 'antd/es/table'

import {
  fetchTokensModelUsage,
  type ModelUsageRow,
  type TokensRangeParams,
} from '@/services/tokensDashboardService'
import { formatThousands } from '@/utils/number'

export interface ModelUsageTableProps {
  range: TokensRangeParams
}

/** 构造列定义（数值列右对齐 + 千分位 + 排序） */
function buildColumns(): ColumnsType<ModelUsageRow> {
  return [
    {
      title: '供应商',
      dataIndex: 'providerName',
      key: 'providerName',
      width: 160,
      ellipsis: true,
    },
    {
      title: '模型',
      dataIndex: 'modelName',
      key: 'modelName',
      ellipsis: true,
    },
    {
      title: '请求次数',
      dataIndex: 'requestCount',
      key: 'requestCount',
      width: 120,
      align: 'right',
      sorter: (a, b) => a.requestCount - b.requestCount,
      render: (value: number) => formatThousands(value),
    },
    {
      title: '输入 Tokens',
      dataIndex: 'inputTokens',
      key: 'inputTokens',
      width: 140,
      align: 'right',
      sorter: (a, b) => a.inputTokens - b.inputTokens,
      render: (value: number) => formatThousands(value),
    },
    {
      title: '输出 Tokens',
      dataIndex: 'outputTokens',
      key: 'outputTokens',
      width: 140,
      align: 'right',
      sorter: (a, b) => a.outputTokens - b.outputTokens,
      render: (value: number) => formatThousands(value),
    },
    {
      title: '总 Tokens',
      dataIndex: 'totalTokens',
      key: 'totalTokens',
      width: 140,
      align: 'right',
      defaultSortOrder: 'descend',
      sorter: (a, b) => a.totalTokens - b.totalTokens,
      render: (value: number) => formatThousands(value),
    },
  ]
}

/**
 * 模型使用汇总表：一次加载 + 虚拟滚动
 */
const ModelUsageTable: React.FC<ModelUsageTableProps> = ({ range }) => {
  const { message } = App.useApp()
  const [rows, setRows] = useState<ModelUsageRow[]>([])
  const [loading, setLoading] = useState(false)
  const seqRef = useRef(0)

  useEffect(() => {
    const currentSeq = ++seqRef.current
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLoading(true)
    fetchTokensModelUsage(range)
      .then((data) => {
        if (currentSeq !== seqRef.current) return
        setRows(data)
      })
      .catch(() => {
        if (currentSeq !== seqRef.current) return
        message.error('获取模型使用汇总失败')
      })
      .finally(() => {
        if (currentSeq !== seqRef.current) return
        setLoading(false)
      })
  }, [range, message])

  const columns = useMemo(() => buildColumns(), [])

  return (
    <Card title="模型使用汇总">
      <Table<ModelUsageRow>
        rowKey={(r) => `${r.providerId}-${r.modelId}`}
        columns={columns}
        dataSource={rows}
        loading={loading}
        pagination={false}
        virtual
        scroll={{ y: 400 }}
        size="middle"
      />
    </Card>
  )
}

export default ModelUsageTable
