import { FundOutlined } from '@ant-design/icons'
import { App, Button } from 'antd'
import { useCallback, useState } from 'react'

import { testModel } from '@/services/aiService'
import { getErrorMsg } from '@/utils/error'

interface TestActionProps {
  modelId: string
  onStatusChanged?: () => void
}

export const TestAction = ({ modelId, onStatusChanged }: TestActionProps) => {
  const { message } = App.useApp()
  const [doingTest, setDoingTest] = useState(false)

  const handleTest = useCallback(async () => {
    if (!modelId) {
      message.warning('请先选择模型')
      return
    }

    try {
      setDoingTest(true)
      const result = await testModel(modelId)

      if (result.success) {
        message.success('测试成功')
        if (result.modelStatusChanged) {
          onStatusChanged?.()
        }
      } else {
        message.error('测试失败')
      }
    } catch (error) {
      message.error(`测试失败：${getErrorMsg(error, '未知错误')}`)
    } finally {
      setDoingTest(false)
    }
  }, [modelId, message, onStatusChanged])

  return (
    <>
      <Button
        size="small"
        color="primary"
        variant="link"
        icon={<FundOutlined />}
        loading={doingTest}
        onClick={handleTest}
      >
        测试
      </Button>
    </>
  )
}
