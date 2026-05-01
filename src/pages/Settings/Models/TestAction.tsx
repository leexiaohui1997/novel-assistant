import { FundOutlined } from '@ant-design/icons'
import { Button, message } from 'antd'
import { useCallback, useState } from 'react'

import { testModel } from '@/services/aiService'
import { getErrorMsg } from '@/utils/error'

interface TestActionProps {
  modelId: string
}

export const TestAction = ({ modelId }: TestActionProps) => {
  const [messageApi, contextHolder] = message.useMessage()
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
        messageApi.success('测试成功')
      } else {
        messageApi.error('测试失败')
      }
    } catch (error) {
      messageApi.error(`测试失败：${getErrorMsg(error, '未知错误')}`)
    } finally {
      setDoingTest(false)
    }
  }, [modelId, messageApi])

  return (
    <>
      {contextHolder}
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
