import { Button, Form, Modal, message } from 'antd'

import NovelBasicForm, { type NovelBasicFormValues } from '@/components/NovelBasicForm'
import { createNovel } from '@/services/novelService'
import { logger } from '@/utils/logger'

interface CreateNovelModalProps {
  open: boolean
  onClose: () => void
  onSuccess?: () => void
}

const CreateNovelModal: React.FC<CreateNovelModalProps> = ({ open, onClose, onSuccess }) => {
  const [form] = Form.useForm<NovelBasicFormValues>()
  const [messageApi, contextHolder] = message.useMessage()

  const handleCancel = () => {
    onClose()
  }

  const handleOk = async () => {
    try {
      const values = await form.validateFields()
      logger.debug('创建作品表单数据:', values)

      try {
        await createNovel({
          title: values.title,
          targetReader: values.targetReader,
          tagIds: values.tagIds || [],
          description: values.description,
        })

        messageApi.success('作品创建成功')
        onSuccess?.()
        onClose()
      } catch (apiError) {
        logger.error('创建作品失败:', apiError)
        messageApi.error('创建作品失败，请重试')
      }
    } catch (validationError) {
      logger.debug('表单校验失败:', validationError)
    }
  }

  return (
    <>
      {contextHolder}
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
        <NovelBasicForm form={form} mode="edit" />
      </Modal>
    </>
  )
}

export default CreateNovelModal
