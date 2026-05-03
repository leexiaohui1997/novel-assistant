import { ThunderboltOutlined } from '@ant-design/icons'
import { Button, Divider, Form, FormInstance, Input, message, Popover } from 'antd'
import { TooltipPlacement } from 'antd/es/tooltip'
import React, { useCallback, useRef, useState } from 'react'

import { ModelSelect } from './ModelSelect'

import { useAiAction, UseAiActionProps } from '@/hooks/useAiAction'
import { getErrorMsg } from '@/utils/error'

/**
 * AI Action 触发器组件属性
 */
export type WithAiActionProps<T = unknown> = {
  /** 操作提示文本（Tooltip 显示内容） */
  tip?: string
  /** 是否禁用按钮 */
  disabled?: boolean
  /** 子组件（通常是输入框或表单字段） */
  children?: React.ReactNode
  /** AI 操作回调函数，由父组件实现具体的业务逻辑 */
  onResult?: (result: T) => unknown
  /** AI 操作的配置 */
  aiAction: UseAiActionProps
  /** 提示 placement */
  placement?: TooltipPlacement
  /** 是否提供意见输入框 */
  showFeedback?: boolean
  /** 样式类名 */
  classNames?: {
    root?: string
    left?: string
    right?: string
  }
}

/**
 * AI Action 触发器组件
 *
 * 提供一个带闪电图标的按钮，用于触发 AI 相关的操作。
 * 自动管理 loading 状态和错误提示，保持 UI 交互的一致性。
 * 内置模型选择器和运行按钮，支持配置 AI Action。
 */
export function WithAiAction<T = unknown>({
  tip,
  children,
  disabled = false,
  aiAction,
  onResult,
  classNames,
  placement = 'leftTop',
  showFeedback = false,
  ...props
}: WithAiActionProps<T>) {
  // Ant Design Message 实例
  const [messageApi, contextHolder] = message.useMessage()
  // 按钮加载状态
  const [loading, setLoading] = useState(false)
  const { execute } = useAiAction<T>(aiAction)
  const formRef = useRef<FormInstance>(null)

  /**
   * 点击处理函数
   * 执行 AI 操作并自动处理 loading 状态和错误提示
   */
  const onClick = useCallback(async () => {
    try {
      setLoading(true)
      const result = await execute({
        modelId: formRef.current?.getFieldValue('modelId'),
        userFeedback: formRef.current?.getFieldValue('userFeedback'),
      })
      await onResult?.(result)
    } catch (error) {
      // 统一错误提示
      messageApi.error(getErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }, [messageApi, onResult, execute])

  return (
    <>
      {contextHolder}
      {/* 布局容器：左侧子组件 + 右侧 AI 按钮 */}
      <div className={`flex items-start gap-2 ${classNames?.root}`}>
        {/* 子组件区域：占据剩余空间 */}
        <div className={`${classNames?.left ?? 'flex-1 w-0'}`}>
          {React.isValidElement(children) ? React.cloneElement(children, props) : children}
        </div>

        {/* 按钮区域：固定宽度 */}
        <div className={`flex items-center gap-1 ${classNames?.right}`}>
          <Popover
            trigger="click"
            placement={placement}
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
                  ref={formRef}
                  disabled={loading}
                  classNames={{ content: 'flex justify-end' }}
                  initialValues={{ modelId: '' }}
                  styles={{}}
                >
                  <Form.Item label="模型" name="modelId">
                    <ModelSelect withAuto className="w-full" allowClear={false} />
                  </Form.Item>

                  {showFeedback && (
                    <Form.Item label="意见" name="userFeedback">
                      <Input.TextArea placeholder="请输入意见" rows={3} />
                    </Form.Item>
                  )}
                </Form>
              </div>
            }
            classNames={{ content: 'min-w-75!' }}
          >
            <Button
              icon={<ThunderboltOutlined />}
              variant="filled"
              color="primary"
              disabled={disabled}
            />
          </Popover>
        </div>
      </div>
    </>
  )
}
