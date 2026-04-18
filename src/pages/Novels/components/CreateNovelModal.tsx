import { Button, Col, Form, Input, Modal, Radio, Row, Select, Space, Typography } from 'antd'

import { logger } from '@/utils/logger'

const { TextArea } = Input
const { Text } = Typography

interface CreateNovelModalProps {
  open: boolean
  onClose: () => void
}

interface NovelFormValues {
  bookName: string
  targetReader: 'male' | 'female'
  tags: string[]
  description: string
}

const CreateNovelModal: React.FC<CreateNovelModalProps> = ({ open, onClose }) => {
  const [form] = Form.useForm<NovelFormValues>()

  const handleCancel = () => {
    form.resetFields()
    onClose()
  }

  const handleOk = async () => {
    try {
      const values = await form.validateFields()
      logger.debug('创建作品表单数据:', values)
      handleCancel()
    } catch (error) {
      logger.error('表单验证失败:', error)
    }
  }

  return (
    <Modal
      title="创建作品"
      open={open}
      onCancel={handleCancel}
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
      <Form form={form} layout="horizontal" labelCol={{ flex: '100px' }} wrapperCol={{ flex: 1 }}>
        <Row>
          <Col flex="120px" style={{ marginRight: '8px' }}>
            {/* 封面占位 */}
            <Form.Item label={null} colon={false}>
              <Space direction="vertical" align="center">
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

          <Col flex={1}>
            {/* 书本名称 */}
            <Form.Item
              label="书本名称"
              name="bookName"
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
              name="targetReader"
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
            <Form.Item label="作品标签" name="tags">
              <Select mode="multiple" placeholder="请选择作品标签" />
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
