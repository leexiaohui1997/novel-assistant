import { ExportOutlined, ImportOutlined, ThunderboltOutlined } from '@ant-design/icons'
import { App, Card } from 'antd'
import { useEffect, useRef, useState } from 'react'

import {
  fetchTokensSummary,
  type TokensRangeParams,
  type TokensSummary,
} from '@/services/tokensDashboardService'
import { formatThousands } from '@/utils/number'

export interface SummaryCardsProps {
  range: TokensRangeParams
}

interface CardMeta {
  key: keyof TokensSummary
  label: string
  icon: React.ReactNode
}

const CARD_META: CardMeta[] = [
  { key: 'totalRequests', label: '总请求数', icon: <ThunderboltOutlined /> },
  { key: 'inputTokens', label: '输入 Tokens', icon: <ImportOutlined /> },
  { key: 'outputTokens', label: '输出 Tokens', icon: <ExportOutlined /> },
]

const EMPTY_SUMMARY: TokensSummary = {
  totalRequests: 0,
  inputTokens: 0,
  outputTokens: 0,
}

/**
 * 汇总卡片区：总请求数 / 输入 tokens / 输出 tokens
 */
const SummaryCards: React.FC<SummaryCardsProps> = ({ range }) => {
  const { message } = App.useApp()
  const [summary, setSummary] = useState<TokensSummary>(EMPTY_SUMMARY)
  const [loading, setLoading] = useState(false)
  // 请求序列号，避免竞态；保留旧数据以防错误时空白
  const seqRef = useRef(0)

  useEffect(() => {
    const currentSeq = ++seqRef.current
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLoading(true)
    fetchTokensSummary(range)
      .then((data) => {
        if (currentSeq !== seqRef.current) return
        setSummary(data)
      })
      .catch(() => {
        if (currentSeq !== seqRef.current) return
        message.error('获取 Tokens 汇总失败')
      })
      .finally(() => {
        if (currentSeq !== seqRef.current) return
        setLoading(false)
      })
  }, [range, message])

  return (
    <div className="grid grid-cols-3 gap-4">
      {CARD_META.map((meta) => (
        <Card key={meta.key} loading={loading} styles={{ body: { padding: 20 } }}>
          <div className="flex items-center gap-4">
            <div className="text-3xl text-brand">{meta.icon}</div>
            <div className="flex flex-col">
              <span className="text-sm text-gray-400">{meta.label}</span>
              <span className="text-2xl font-semibold">{formatThousands(summary[meta.key])}</span>
            </div>
          </div>
        </Card>
      ))}
    </div>
  )
}

export default SummaryCards
