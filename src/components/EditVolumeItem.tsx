import { CheckOutlined, CloseOutlined, DeleteOutlined, EditOutlined } from '@ant-design/icons'
import { Button, Input, InputRef, Popconfirm, Space, Tooltip, message } from 'antd'
import { useCallback, useRef, useState } from 'react'

import { SimpleVolume } from '@/services/chapterService'
import { numToCn } from '@/utils/number'

/** 编辑分卷条目组件属性 */
interface EditVolumeItemProps {
  /** 分卷数据 */
  volume: SimpleVolume
  /** 是否为"新建且尚未命名"的条目：初始即进入编辑态；取消编辑时整条删除 */
  isNew?: boolean
  /** 是否为尚未持久化到后端的草稿条目：删除时免二次确认 */
  isDraft?: boolean
  /** 是否允许删除；false 时删除按钮将禁用 */
  deletable?: boolean
  /** 不可删除时的提示文案（仅在 deletable=false 时显示） */
  disabledReason?: string
  /** 重命名回调，参数为新名称 */
  onRename?: (newName: string) => unknown
  /** 删除回调 */
  onDelete?: () => unknown
  /** 取消新建回调：仅在 isNew=true 且用户放弃编辑时触发 */
  onCancelCreate?: () => void
}

/**
 * 编辑分卷条目组件
 *
 * 支持查看、重命名（行内编辑）和删除（二次确认/禁用提示）分卷。
 */
const EditVolumeItem: React.FC<EditVolumeItemProps> = ({
  volume,
  isNew = false,
  isDraft = false,
  deletable = true,
  disabledReason = '',
  onRename,
  onDelete,
  onCancelCreate,
}) => {
  const [messageApi, contextHolder] = message.useMessage()
  const inputRef = useRef<InputRef>(null)
  const [isEditing, setIsEditing] = useState(isNew)
  const [doingRename, setDoingRename] = useState(false)

  /** 退出编辑模式 */
  const finishEdit = useCallback(() => setIsEditing(false), [setIsEditing])

  /** 取消编辑：新建未命名条目整条删除，已命名条目仅退出编辑态（非受控 Input 会自动还原原名） */
  const handleCancel = useCallback(() => {
    if (isNew) {
      onCancelCreate?.()
      return
    }
    finishEdit()
  }, [isNew, onCancelCreate, finishEdit])

  /** 确认重命名：trim 后为空则提示并阻止提交；未变更则静默退出编辑 */
  const handleRename = useCallback(async () => {
    const newName = (inputRef.current?.input?.value ?? '').trim()
    if (!newName) {
      messageApi.warning('分卷名字至少输入1个字')
      return
    }
    if (onRename && newName !== volume.name) {
      try {
        setDoingRename(true)
        await Promise.resolve(onRename(newName))
      } finally {
        setDoingRename(false)
      }
    }
    finishEdit()
  }, [onRename, finishEdit, volume, messageApi])

  /** 渲染删除按钮（含禁用/二次确认两种分支） */
  const renderDeleteBtn = () => {
    if (!deletable) {
      return (
        <Tooltip title={disabledReason}>
          <Button icon={<DeleteOutlined />} color="danger" variant="link" disabled />
        </Tooltip>
      )
    }
    // 草稿条目（尚未提交到后端）直接删除，不做二次确认
    if (isDraft) {
      return <Button icon={<DeleteOutlined />} color="danger" variant="link" onClick={onDelete} />
    }
    return (
      <Popconfirm
        title="确认删除该分卷？"
        description="删除后不可恢复，请谨慎操作。"
        okText="删除"
        okButtonProps={{ danger: true }}
        cancelText="取消"
        onConfirm={onDelete}
      >
        <Button icon={<DeleteOutlined />} color="danger" variant="link" />
      </Popconfirm>
    )
  }

  const sequenceLabel = `第${numToCn(volume.sequence)}卷：`
  const inputMaxLength = 20 - sequenceLabel.length

  return (
    <div
      className={`flex items-center h-10 rounded-md cursor-pointer ${isEditing ? '' : 'hover:bg-gray-50'}`}
    >
      {contextHolder}
      <div
        className={`flex-1 w-0 flex items-center h-full rounded-md px-3.75!  ${isEditing ? 'bg-brand/10' : ''}`}
      >
        <div>{sequenceLabel}</div>
        <div className="flex-1 w-0">
          {isEditing ? (
            <Input
              ref={inputRef}
              defaultValue={volume.name}
              variant="borderless"
              placeholder="请输入分卷名字"
              count={{
                max: inputMaxLength,
                show: ({ count }) => `${count + sequenceLabel.length}/20`,
                exceedFormatter: (value, { max }) => value.slice(0, max),
              }}
              autoFocus
              onPressEnter={handleRename}
            />
          ) : (
            <span>{volume.name}</span>
          )}
        </div>
      </div>
      <Space size={0}>
        {isEditing ? (
          <>
            <Button
              icon={<CheckOutlined />}
              color="green"
              variant="link"
              loading={doingRename}
              onClick={handleRename}
            />
            <Button icon={<CloseOutlined />} color="danger" variant="link" onClick={handleCancel} />
          </>
        ) : (
          <>
            {renderDeleteBtn()}
            <Button
              icon={<EditOutlined />}
              color="default"
              variant="link"
              onClick={() => setIsEditing(true)}
            />
          </>
        )}
      </Space>
    </div>
  )
}

export default EditVolumeItem
