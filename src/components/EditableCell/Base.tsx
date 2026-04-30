import { CheckOutlined, CloseOutlined, EditOutlined } from '@ant-design/icons'
import { Button, message } from 'antd'
import { SizeType } from 'antd/es/config-provider/SizeContext'
import { MessageInstance } from 'antd/es/message/interface'
import React, { useCallback, useMemo, useState } from 'react'

import { getErrorMsg } from '@/utils/error'

/** 默认只读态渲染：直接展示值 */
const DefaultReadCell = (value: unknown) => <>{value}</>

/** 默认编辑态渲染：空占位 */
const DefaultEditCell = () => <></>

/** 默认变更回调：空操作 */
const DefaultChangeHandle = () => {}

/** 编辑单元格上下文，提供提交编辑的方法 */
interface EditCellContext {
  /** 尺寸 */
  size: SizeType
  /** 是否处于错误状态 */
  isError: boolean
  /** 提交当前编辑 */
  commitEdit: () => Promise<void>
}

/** Hook 上下文，提供 antd message 实例等 */
interface HookContext {
  /** antd message API，用于弹出提示 */
  messageApi: MessageInstance
}

/**
 * 可编辑单元格基础组件的属性
 *
 * @template T - 编辑值的类型
 */
export type BaseEditableCellProps<T> = {
  /** 当前展示的值 */
  value: T
  /** 尺寸 */
  size?: SizeType
  /** 提交前的校验函数，抛出异常则阻止提交 */
  validate?: (value: T) => unknown
  /** 只读态的自定义渲染函数 */
  readCell?: (value: T) => React.ReactNode
  /** 编辑态的自定义渲染函数，接收当前值、变更函数和编辑上下文 */
  editCell?: (
    value: T,
    onChange: React.Dispatch<React.SetStateAction<T>>,
    ctx: EditCellContext,
  ) => React.ReactNode
  /** 提交成功的回调（值变更持久化） */
  onChange?: (value: T) => unknown
  /** 编辑过程出错的回调 */
  onEditError?: (error: unknown, ctx: HookContext) => unknown
  /** 编辑成功的回调 */
  onEditSuccess?: (ctx: HookContext) => unknown
}

/**
 * 可编辑单元格基础组件
 *
 * 提供只读/编辑双态切换，内置校验、提交、取消、错误提示等逻辑。
 * 适用于表格行内编辑等场景。
 *
 * @template T - 编辑值的类型
 *
 * @example
 * ```tsx
 * <BaseEditableCell
 *   value={name}
 *   onChange={handleNameChange}
 *   validate={(v) => { if (!v) throw new Error('名称不能为空') }}
 *   editCell={(val, setVal) => <Input value={val} onChange={(e) => setVal(e.target.value)} />}
 * />
 * ```
 */
export function BaseEditableCell<T>({
  value,
  size = 'medium',
  validate,
  onEditError,
  onEditSuccess,
  onChange = DefaultChangeHandle,
  readCell = DefaultReadCell,
  editCell = DefaultEditCell,
}: BaseEditableCellProps<T>) {
  const [messageApi, messageContext] = message.useMessage()

  /** 是否处于编辑态 */
  const [isEdit, setIsEdit] = useState(false)
  /** 编辑态下的临时值 */
  const [inputValue, setInputValue] = useState(value)
  /** 是否正在提交中 */
  const [doingCommit, setDoingCommit] = useState(false)
  const [isError, setIsError] = useState(false)

  const hookContext = useMemo(
    () => ({
      messageApi,
    }),
    [messageApi],
  )

  /** 进入编辑态，用当前值初始化临时值 */
  const startEdit = useCallback(() => {
    setInputValue(value)
    setIsEdit(true)
  }, [value])

  /** 取消编辑，回到只读态 */
  const cancelEdit = () => {
    setIsEdit(false)
    setIsError(false)
  }

  /** 提交编辑：校验 → 持久化 → 成功回调；失败则提示错误 */
  const commitEdit = useCallback(async () => {
    try {
      await validate?.(inputValue)
      setDoingCommit(true)
      await onChange(inputValue)
      setIsError(false)
      setIsEdit(false)
      onEditSuccess?.(hookContext)
    } catch (error) {
      setIsError(true)
      onEditError?.(error, hookContext)
      messageApi.error(getErrorMsg(error))
    } finally {
      setDoingCommit(false)
    }
  }, [onChange, validate, hookContext, messageApi, inputValue, onEditError, onEditSuccess])

  return (
    <>
      {messageContext}
      <div className="flex items-center gap-2">
        <div className="flex-1 w-0">
          {isEdit
            ? editCell(inputValue, setInputValue, { size, isError, commitEdit })
            : readCell(value)}
        </div>
        <div className="flex items-center gap-1">
          {isEdit ? (
            <>
              <Button
                icon={<CheckOutlined />}
                size={size}
                loading={doingCommit}
                color="green"
                variant="text"
                onClick={commitEdit}
              />
              <Button
                icon={<CloseOutlined />}
                size={size}
                color="danger"
                variant="text"
                onClick={cancelEdit}
              />
            </>
          ) : (
            <>
              <Button
                icon={<EditOutlined />}
                size={size}
                color="primary"
                variant="text"
                onClick={startEdit}
              />
            </>
          )}
        </div>
      </div>
    </>
  )
}
