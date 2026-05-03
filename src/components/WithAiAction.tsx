import { CaretDownFilled, ThunderboltOutlined } from '@ant-design/icons'
import { Button, Divider, Form, message, Popover, Space } from 'antd'
import React, { useCallback, useState } from 'react'

import { ModelSelect } from './ModelSelect'

import { getErrorMsg } from '@/utils/error'

/**
 * AI Action 触发器组件属性
 */
export type WithAiActionProps = {
  /** 操作提示文本（Tooltip 显示内容） */
  tip?: string
  /** 是否禁用按钮 */
  disabled?: boolean
  /** 子组件（通常是输入框或表单字段） */
  children?: React.ReactNode
  /** AI 操作回调函数，由父组件实现具体的业务逻辑 */
  onAction?: () => unknown
}

/**
 * AI Action 触发器组件
 *
 * 提供一个带闪电图标的按钮，用于触发 AI 相关的操作。
 * 自动管理 loading 状态和错误提示，保持 UI 交互的一致性。
 *
 * @example
 * ```tsx
 * <WithAiAction
 *   tip="使用 AI 推荐标签"
 *   onAction={async () => {
 *     // 调用后端 AI Action
 *     const result = await invoke('execute_action', {
 *       actionName: 'recommend_tags',
 *       actionParams: { channel: 'male' }
 *     })
 *   }}
 * >
 *   <Input placeholder="输入标签" />
 * </WithAiAction>
 * ```
 */
export const WithAiAction: React.FC<WithAiActionProps> = ({
  tip,
  children,
  disabled = false,
  onAction,
  ...props
}) => {
  // Ant Design Message 实例
  const [messageApi, contextHolder] = message.useMessage()
  // 按钮加载状态
  const [loading, setLoading] = useState(false)

  /**
   * 点击处理函数
   * 执行 AI 操作并自动处理 loading 状态和错误提示
   */
  const onClick = useCallback(async () => {
    try {
      setLoading(true)
      await onAction?.()
    } catch (error) {
      // 统一错误提示
      messageApi.error(getErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }, [messageApi, onAction])

  return (
    <>
      {contextHolder}
      {/* 布局容器：左侧子组件 + 右侧 AI 按钮 */}
      <div className="flex items-start gap-2">
        {/* 子组件区域：占据剩余空间 */}
        <div className="flex-1 w-0">
          {React.isValidElement(children) ? React.cloneElement(children, props) : children}
        </div>

        {/* 按钮区域：固定宽度 */}
        <div className="flex items-center gap-1">
          <Space.Compact>
            <Button
              icon={<ThunderboltOutlined />}
              variant="filled"
              color="primary"
              loading={loading}
              disabled={disabled}
              onClick={onClick}
            />

            <Popover
              trigger="click"
              placement="bottomRight"
              content={
                <div>
                  <div className="flex items-center justify-between">
                    <div>{tip}</div>

                    <div className="flex items-center justify-center">
                      <Button
                        size="small"
                        type="primary"
                        className="min-w-20"
                        icon={<ThunderboltOutlined />}
                        loading={loading}
                        disabled={disabled}
                        onClick={onClick}
                      >
                        运行
                      </Button>
                    </div>
                  </div>
                  <Divider size="small"></Divider>
                  <Form
                    classNames={{ content: 'flex justify-end' }}
                    initialValues={{ modelId: '' }}
                  >
                    <Form.Item label="模型" name="modelId">
                      <ModelSelect withAuto className="w-full" allowClear={false} />
                    </Form.Item>
                  </Form>
                </div>
              }
              classNames={{ content: 'min-w-75!' }}
            >
              <Button
                icon={<CaretDownFilled />}
                variant="filled"
                color="primary"
                disabled={disabled}
              />
            </Popover>
          </Space.Compact>
        </div>
      </div>
    </>
  )
}
