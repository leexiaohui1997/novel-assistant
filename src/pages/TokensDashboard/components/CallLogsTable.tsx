import { BarChartOutlined } from '@ant-design/icons'
import { App, Button, Card, Table } from 'antd'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import type { ColumnsType } from 'antd/es/table'

import { AiCallLogDrawer, AiCallLogDrawerHandle } from '@/components/AiCallLog/AiCallLogDrawer'
import {
  fetchAiCallLogs,
  type AiCallLogItem,
  type TokensRangeParams,
} from '@/services/tokensDashboardService'
import { formatDateTime } from '@/utils/date'
import { formatThousands } from '@/utils/number'

export interface CallLogsTableProps {
  range: TokensRangeParams
}

const DEFAULT_PAGE_SIZE = 50

/** 构造列定义 */
function buildColumns(onDetail: (record: AiCallLogItem) => void): ColumnsType<AiCallLogItem> {
  return [
    {
      title: '请求时间',
      dataIndex: 'callTime',
      key: 'callTime',
      width: 180,
      render: (value: string) => formatDateTime(value),
    },
    {
      title: '供应商',
      dataIndex: 'providerName',
      key: 'providerName',
      width: 140,
      ellipsis: true,
    },
    {
      title: '模型',
      dataIndex: 'modelName',
      key: 'modelName',
      width: 180,
      ellipsis: true,
    },
    {
      title: '提问',
      dataIndex: 'message',
      key: 'message',
      width: 300,
      render: (value: string) => <div className="text-cut w-75">{value}</div>,
    },
    {
      title: '总 Tokens',
      dataIndex: 'totalTokens',
      key: 'totalTokens',
      width: 120,
      align: 'right',
      render: (value: number) => formatThousands(value),
    },
    {
      title: '操作',
      key: 'actions',
      width: 100,
      align: 'center',
      fixed: 'right',
      render: (_, record) => (
        <Button
          color="primary"
          variant="link"
          icon={<BarChartOutlined />}
          onClick={() => onDetail(record)}
        >
          详情
        </Button>
      ),
    },
  ]
}

/**
 * 详细使用记录表：分页加载
 */
const CallLogsTable: React.FC<CallLogsTableProps> = ({ range }) => {
  const { message } = App.useApp()
  const [rows, setRows] = useState<AiCallLogItem[]>([])
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(1)
  const [loading, setLoading] = useState(false)
  const seqRef = useRef(0)
  const drawerRef = useRef<AiCallLogDrawerHandle>(null)

  // 时间范围变化时重置到第 1 页
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setPage(1)
  }, [range])

  useEffect(() => {
    const currentSeq = ++seqRef.current
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLoading(true)
    fetchAiCallLogs(range, page, DEFAULT_PAGE_SIZE)
      .then((result) => {
        if (currentSeq !== seqRef.current) return
        setRows(result.data)
        setTotal(result.total)
      })
      .catch(() => {
        if (currentSeq !== seqRef.current) return
        message.error('获取 AI 调用记录失败')
      })
      .finally(() => {
        if (currentSeq !== seqRef.current) return
        setLoading(false)
      })
  }, [range, page, message])

  // 详情按钮占位：待实现
  const handleDetail = useCallback((record: AiCallLogItem) => {
    drawerRef.current?.open(record)
  }, [])

  // eslint-disable-next-line react-hooks/refs
  const columns = useMemo(() => buildColumns(handleDetail), [handleDetail])

  return (
    <>
      <Card title="详细使用记录">
        <Table<AiCallLogItem>
          rowKey="id"
          columns={columns}
          dataSource={rows}
          loading={loading}
          pagination={{
            current: page,
            pageSize: DEFAULT_PAGE_SIZE,
            total,
            showSizeChanger: false,
            showTotal: (t) => `共 ${t} 条`,
            onChange: (nextPage) => setPage(nextPage),
          }}
          scroll={{ x: 'max-content' }}
          size="middle"
        />
      </Card>
      <AiCallLogDrawer ref={drawerRef} />
    </>
  )
}

export default CallLogsTable
