import { Button, Card, Form, Space, message } from 'antd'
import { useMemo, useState } from 'react'

import NovelBasicForm, { type NovelBasicFormValues } from '@/components/NovelBasicForm'
import { useCreationState } from '@/hooks/useCreationState'
import { updateNovel } from '@/services/novelService'
import { type TargetAudience } from '@/services/tagService'
import { logger } from '@/utils/logger'

type Mode = 'edit' | 'view'

/**
 * 基础信息卡片：支持查看 / 编辑两种模式切换
 */
const BasicInfoCard: React.FC = () => {
  const { novelId, novelInfo, refreshNovelInfo } = useCreationState()
  const [form] = Form.useForm<NovelBasicFormValues>()
  const [mode, setMode] = useState<Mode>('view')
  const [saving, setSaving] = useState(false)
  const [messageApi, contextHolder] = message.useMessage()

  const initialValues = useMemo<Partial<NovelBasicFormValues>>(
    () => ({
      title: novelInfo.title,
      targetReader: novelInfo.targetReader as Exclude<TargetAudience, 'both'>,
      tagIds: (novelInfo.tags || []).map((t) => t.id),
      description: novelInfo.description,
    }),
    [novelInfo],
  )

  const handleEdit = () => {
    form.setFieldsValue(initialValues)
    setMode('edit')
  }

  const handleCancel = () => {
    form.resetFields()
    setMode('view')
  }

  const handleSave = async () => {
    try {
      const values = await form.validateFields()
      setSaving(true)
      try {
        await updateNovel(novelId, values)
        await refreshNovelInfo()
        messageApi.success('保存成功')
        setMode('view')
      } catch (e) {
        logger.error('保存基础信息失败:', e)
        messageApi.error('保存失败，请重试')
      } finally {
        setSaving(false)
      }
    } catch (e) {
      logger.debug('表单校验失败:', e)
    }
  }

  const extra =
    mode === 'view' ? (
      <Button type="primary" onClick={handleEdit}>
        修改
      </Button>
    ) : (
      <Space>
        <Button onClick={handleCancel}>取消</Button>
        <Button type="primary" onClick={handleSave} loading={saving}>
          保存
        </Button>
      </Space>
    )

  return (
    <>
      {contextHolder}
      <Card title="基础信息" extra={extra}>
        <NovelBasicForm
          form={form}
          mode={mode}
          initialValues={initialValues}
          tags={novelInfo.tags}
        />
      </Card>
    </>
  )
}

export default BasicInfoCard
