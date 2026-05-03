import { App, Button, Col, Form, Input, Radio, Row, Space, Tag as AntdTag, Typography } from 'antd'
import { FormInstance } from 'antd/es/form'
import { useRef } from 'react'

import { WithAiAction } from '../WithAiAction'

import TagSelector, { TagSelectorHandle } from '@/pages/Novels/components/TagSelector'
import { type Tag, type TargetAudience } from '@/services/tagService'

const { TextArea } = Input
const { Text } = Typography

/**
 * 表单字段值类型
 */
export interface NovelBasicFormValues {
  title: string
  targetReader: Exclude<TargetAudience, 'both'>
  tagIds: number[]
  description: string
}

/**
 * 组件 Props
 */
interface NovelBasicFormProps {
  /** 外部表单实例 */
  form: FormInstance<NovelBasicFormValues>
  /** 模式：edit 可编辑 / view 查看（纯文本展示） */
  mode?: 'edit' | 'view'
  /** 初始值 */
  initialValues?: Partial<NovelBasicFormValues>
  /** 查看态下用于展示标签名称（完整 Tag 对象列表） */
  tags?: Tag[]
}

/**
 * 作品基础信息表单（可复用）
 *
 * - edit 模式：标准表单控件
 * - view 模式：以纯文本展示当前值，保留标签结构
 */
const NovelBasicForm: React.FC<NovelBasicFormProps> = ({
  form,
  mode = 'edit',
  initialValues,
  tags,
}) => {
  const isView = mode === 'view'
  const { message } = App.useApp()
  const tagSelectorRef = useRef<TagSelectorHandle>(null)

  return (
    <Form
      form={form}
      layout="horizontal"
      labelCol={{ flex: '100px' }}
      wrapperCol={{ flex: 1 }}
      classNames={{ content: 'w-0' }}
      initialValues={initialValues}
    >
      <Row>
        <Col flex="120px" style={{ marginRight: '8px' }}>
          <Form.Item label={null} colon={false}>
            <Space orientation="vertical" align="center">
              <div
                style={{
                  width: 120,
                  height: 160,
                  backgroundColor: '#f0f0f0',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  borderRadius: 4,
                }}
              >
                <Text type="secondary">封面</Text>
              </div>
              {!isView && <Button>选择封面</Button>}
            </Space>
          </Form.Item>
        </Col>

        <Col flex={1} className="w-0">
          <Form.Item
            label="书本名称"
            name="title"
            rules={[
              { required: true, message: '请输入作品名称' },
              { max: 15, message: '作品名称不能超过15个字' },
            ]}
          >
            {isView ? (
              <ViewText />
            ) : (
              <WithAiAction
                tip="AI 生成书名"
                aiAction={{
                  actionName: 'generate_title',
                  getParams: () => ({
                    channel: form.getFieldValue('targetReader'),
                    tag_ids: form.getFieldValue('tagIds'),
                    introduction: form.getFieldValue('description'),
                  }),
                }}
                onAction={async (result: { title: string }) => {
                  if (result?.title) {
                    form.setFieldValue('title', result.title)
                    message.success('已生成书名')
                  } else {
                    message.info('未生成书名')
                  }
                }}
              >
                <Input placeholder="请输入作品名称" maxLength={15} showCount />
              </WithAiAction>
            )}
          </Form.Item>

          <Form.Item
            label="目标读者"
            name="targetReader"
            rules={[{ required: true, message: '请选择目标读者' }]}
          >
            {isView ? (
              <ViewTargetReader />
            ) : (
              <Radio.Group>
                <Space>
                  <Radio value="male">男频</Radio>
                  <Radio value="female">女频</Radio>
                </Space>
              </Radio.Group>
            )}
          </Form.Item>

          <Form.Item
            noStyle
            shouldUpdate={(prev, curr) =>
              prev.targetReader !== curr.targetReader || prev.tagIds !== curr.tagIds
            }
          >
            {() => (
              <Form.Item
                label="作品标签"
                name="tagIds"
                rules={[{ required: true, message: '请选择作品标签' }]}
              >
                {isView ? (
                  <ViewTagIds tags={tags} />
                ) : (
                  <WithAiAction
                    tip="AI 推荐标签"
                    aiAction={{
                      actionName: 'recommend_tags',
                      getParams: () => ({
                        title: form.getFieldValue('title'),
                        channel: form.getFieldValue('targetReader'),
                        introduction: form.getFieldValue('description'),
                        tag_ids: form.getFieldValue('tagIds'),
                      }),
                    }}
                    onAction={async (result: { tags: number[] }) => {
                      if (result?.tags && result.tags.length > 0) {
                        // 将推荐的标签 ID 填入表单
                        const originTags = form.getFieldValue('tagIds') || []
                        const newTags = [...new Set([...originTags, ...result.tags])]
                        form.setFieldValue('tagIds', newTags)
                        message.success(`已推荐 ${result.tags.length} 个标签`)
                      } else {
                        message.info('未找到合适的标签推荐')
                      }
                    }}
                    disabled={!form.getFieldValue('targetReader')}
                  >
                    <TagSelector
                      ref={tagSelectorRef}
                      targetAudience={form.getFieldValue('targetReader')}
                    />
                  </WithAiAction>
                )}
              </Form.Item>
            )}
          </Form.Item>

          <Form.Item
            label="作品简介"
            name="description"
            rules={[
              { required: true, message: '请输入作品简介' },
              { max: 500, message: '作品简介不能超过500字' },
            ]}
          >
            {isView ? (
              <ViewText multiline />
            ) : (
              <WithAiAction
                tip="AI 生成作品简介"
                aiAction={{
                  actionName: 'generate_introduction',
                  getParams: () => ({
                    title: form.getFieldValue('title'),
                    channel: form.getFieldValue('targetReader'),
                    tag_ids: form.getFieldValue('tagIds'),
                  }),
                }}
                onAction={async (result: { introduction: string }) => {
                  if (result?.introduction) {
                    form.setFieldValue('description', result.introduction)
                    message.success('已生成作品简介')
                  } else {
                    message.info('未生成简介内容')
                  }
                }}
              >
                <TextArea
                  placeholder="请输入50-500字以内的作品简介，不可出现低俗、暴力、血腥等不符合法律法规的内容"
                  maxLength={500}
                  rows={6}
                  showCount
                />
              </WithAiAction>
            )}
          </Form.Item>
        </Col>
      </Row>
    </Form>
  )
}

/**
 * 查看态通用文本展示（受控组件，读取 value）
 */
const ViewText: React.FC<{ value?: string; multiline?: boolean }> = ({ value, multiline }) => {
  if (!value) {
    return <Text type="secondary">-</Text>
  }
  return (
    <div style={{ whiteSpace: multiline ? 'pre-wrap' : 'normal', lineHeight: 1.8 }}>{value}</div>
  )
}

/**
 * 查看态目标读者展示
 */
const ViewTargetReader: React.FC<{ value?: string }> = ({ value }) => {
  const map: Record<string, string> = { male: '男频', female: '女频' }
  return <div>{value ? map[value] || '-' : '-'}</div>
}

/**
 * 查看态标签展示：依据 value (tagIds) 与外部传入的 tags 列表匹配标签名
 */
const ViewTagIds: React.FC<{ value?: number[]; tags?: Tag[] }> = ({ value, tags }) => {
  if (!value || value.length === 0) {
    return <Text type="secondary">-</Text>
  }
  const nameMap = new Map((tags || []).map((t) => [t.id, t.name]))
  return (
    <Space size={[4, 4]} wrap>
      {value.map((id) => (
        <AntdTag key={id}>{nameMap.get(id) || id}</AntdTag>
      ))}
    </Space>
  )
}

export default NovelBasicForm
