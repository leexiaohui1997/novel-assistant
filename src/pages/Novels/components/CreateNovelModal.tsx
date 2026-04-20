import { Button, Col, Form, Input, Modal, Radio, Row, Space, Typography, message } from 'antd'

import TagSelector from './TagSelector'

import { createNovel } from '@/services/novelService'
import { type TargetAudience } from '@/services/tagService'
import { logger } from '@/utils/logger'

const { TextArea } = Input
const { Text } = Typography

interface CreateNovelModalProps {
  open: boolean
  onClose: () => void
  onSuccess?: () => void
}

interface NovelFormValues {
  title: string
  target_reader: Exclude<TargetAudience, 'both'>
  tag_ids: number[]
  description: string
}

const CreateNovelModal: React.FC<CreateNovelModalProps> = ({ open, onClose, onSuccess }) => {
  const [form] = Form.useForm<NovelFormValues>()

  const handleCancel = () => {
    onClose()
  }

  const handleOk = async () => {
    try {
      const values = await form.validateFields()
      logger.debug('创建作品表单数据:', values)

      // 调用后端 API
      await createNovel({
        title: values.title,
        target_reader: values.target_reader,
        tag_ids: values.tag_ids || [],
        description: values.description,
      })

      message.success('作品创建成功')
      onSuccess?.()
      onClose()
    } catch (error) {
      logger.error('创建作品失败:', error)
      message.error('创建作品失败，请重试')
    }
  }

  return (
    <Modal
      title="创建作品"
      open={open}
      onCancel={handleCancel}
      afterClose={() => form.resetFields()}
      width={800}
      footer={[
        <Button key="cancel" onClick={handleCancel}>
          取消
        </Button>,
        <Button key="submit" type="primary" onClick={handleOk}>
          立即创建
        </Button>,
      ]}
    >
      <Form
        form={form}
        layout="horizontal"
        labelCol={{ flex: '100px' }}
        wrapperCol={{ flex: 1 }}
        classNames={{ content: 'w-0' }}
      >
        <Row>
          <Col flex="120px" style={{ marginRight: '8px' }}>
            {/* 封面占位 */}
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
                <Button>选择封面</Button>
              </Space>
            </Form.Item>
          </Col>

          <Col flex={1} className="w-0">
            {/* 书本名称 */}
            <Form.Item
              label="书本名称"
              name="title"
              rules={[
                { required: true, message: '请输入作品名称' },
                { max: 15, message: '作品名称不能超过15个字' },
              ]}
            >
              <Input placeholder="请输入作品名称" maxLength={15} showCount />
            </Form.Item>

            {/* 目标读者 */}
            <Form.Item
              label="目标读者"
              name="target_reader"
              rules={[{ required: true, message: '请选择目标读者' }]}
            >
              <Radio.Group>
                <Space>
                  <Radio value="male">男频</Radio>
                  <Radio value="female">女频</Radio>
                </Space>
              </Radio.Group>
            </Form.Item>

            {/* 作品标签 */}
            <Form.Item
              noStyle
              shouldUpdate={(prevValues, currentValues) =>
                prevValues.target_reader !== currentValues.target_reader
              }
            >
              {() => (
                <Form.Item label="作品标签" name="tag_ids" shouldUpdate>
                  <TagSelector targetAudience={form.getFieldValue('target_reader')} />
                </Form.Item>
              )}
            </Form.Item>

            {/* 作品简介 */}
            <Form.Item
              label="作品简介"
              name="description"
              rules={[
                { required: true, message: '请输入作品简介' },
                { max: 500, message: '作品简介不能超过500字' },
              ]}
            >
              <TextArea
                placeholder="请输入50-500字以内的作品简介，不可出现低俗、暴力、血腥等不符合法律法规的内容"
                maxLength={500}
                rows={6}
                showCount
              />
            </Form.Item>
          </Col>
        </Row>
      </Form>
    </Modal>
  )
}

export default CreateNovelModal
