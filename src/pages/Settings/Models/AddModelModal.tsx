import { message, Modal, Space } from 'antd'
import React, { useEffect, useImperativeHandle, useState } from 'react'

import { ProviderSelect } from '@/components/ProviderSelect'
import { fetchProviderModels, ModelInfo } from '@/services/modelService'
import { logger } from '@/utils/logger'

export interface AddModelModalHandle {
  show: () => void
}

export interface AddModelModalProps {
  ref?: React.RefObject<AddModelModalHandle>
}

const AddModelModal: React.FC<AddModelModalProps> = ({ ref }) => {
  const [messageApi, messageContext] = message.useMessage()
  const [open, setOpen] = useState(false)
  const [selectedProviderId, setSelectedProviderId] = useState<string>()
  const [fetchingModels, setFetchingModels] = useState(false)
  const [models, setModels] = useState<ModelInfo[]>([])

  const show = () => {
    setOpen(true)
  }

  const close = () => {
    setOpen(false)
  }

  useImperativeHandle(ref, () => {
    return {
      show,
    }
  })

  useEffect(() => {
    let cancelled = false

    const loadModels = async () => {
      if (!selectedProviderId) {
        setModels([])
        setFetchingModels(false)
        return
      }

      try {
        setFetchingModels(true)
        const result = await fetchProviderModels(selectedProviderId)
        if (!cancelled) {
          setModels(result)
        }
      } catch (error) {
        if (!cancelled) {
          logger.error('加载模型列表失败:', error)
          messageApi.error('加载模型列表失败')
        }
      } finally {
        if (!cancelled) {
          setFetchingModels(false)
        }
      }
    }

    void loadModels()

    return () => {
      cancelled = true
    }
  }, [selectedProviderId, messageApi])

  console.log({ fetchingModels, models })

  return (
    <>
      {messageContext}
      <Modal title="添加模型" open={open} onCancel={close} width={800}>
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <Space>
              <ProviderSelect
                className="w-50"
                value={selectedProviderId}
                onChange={setSelectedProviderId}
              />
            </Space>
          </div>
        </div>
      </Modal>
    </>
  )
}

export default AddModelModal
