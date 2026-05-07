import { PlusOutlined, MinusOutlined } from '@ant-design/icons'
import { Collapse, CollapseProps, Typography } from 'antd'
import { useMemo } from 'react'

import { JsonContent } from '../Markdown/JsonContent'
import { MarkdownContent } from '../Markdown/MarkdownContent'

import { AiCallLogItem } from '@/services/tokensDashboardService'
import { isJson } from '@/utils/check'

const { Title } = Typography

export type AICallLogMessagesProps = {
  logInfo: AiCallLogItem
}

export function AICallLogMessages({ logInfo }: AICallLogMessagesProps) {
  const items = useMemo(() => {
    const messages: Exclude<CollapseProps['items'], undefined> = [
      {
        key: 'input',
        label: '输入',
        extra: <span className="text-gray-400 text-xs">{logInfo.inputTokens} tokens</span>,
        children: <MarkdownContent content={logInfo.message} />,
      },
      {
        key: 'output',
        label: '输出',
        extra: <span className="text-gray-400 text-xs">{logInfo.outputTokens} tokens</span>,
        children: logInfo.response ? (
          <>
            {isJson(logInfo.response) ? (
              <JsonContent content={logInfo.response} />
            ) : (
              <MarkdownContent content={logInfo.response} />
            )}
          </>
        ) : (
          <></>
        ),
      },
    ]

    if (logInfo.thinkingContent) {
      messages.push({
        key: 'thinking',
        label: '思考',
        children: <MarkdownContent content={logInfo.thinkingContent} />,
      })
    }

    return messages
  }, [logInfo])

  return (
    <div>
      <Title level={5}>对话详情</Title>
      <Collapse
        size="small"
        items={items}
        expandIcon={({ isActive }) => (isActive ? <MinusOutlined /> : <PlusOutlined />)}
      />
    </div>
  )
}
