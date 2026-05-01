import { Empty, message, Modal, Space, Table } from 'antd'
import { ColumnsType } from 'antd/es/table'
import React, { useCallback, useEffect, useImperativeHandle, useState } from 'react'

import { useModelIdFilters } from './hooks/useModelIdFilters'

import { InputEditableCell } from '@/components/EditableCell/Input'
import { ProviderSelect } from '@/components/ProviderSelect'
import { addModels, fetchProviderModels, ModelInfo } from '@/services/modelService'
import { logger } from '@/utils/logger'

export interface AddModelModalHandle {
  show: () => void
}

export interface AddModelModalProps {
  ref?: React.RefObject<AddModelModalHandle>
  /** 添加成功后回调，用于外部刷新列表 */
  onSuccess?: () => void
}

const AddModelModal: React.FC<AddModelModalProps> = ({ ref, onSuccess }) => {
  const [messageApi, messageContext] = message.useMessage()
  const [open, setOpen] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [selectedProviderId, setSelectedProviderId] = useState<string>()
  const [fetchingModels, setFetchingModels] = useState(false)
  const [models, setModels] = useState<ModelInfo[]>([])
  const [selectedModelIds, setSelectedModelIds] = useState<React.Key[]>([])

  const { modelIdFilters, modelIdFilterSearch } = useModelIdFilters(models)

  const show = () => {
    setOpen(true)
  }

  const resetState = () => {
    setSelectedProviderId(undefined)
    setModels([])
    setSelectedModelIds([])
  }

  const close = () => {
    setOpen(false)
  }

  const setModelName = useCallback(({ modelId }: { modelId: string }, modelName: string) => {
    setModels((models) =>
      models.map((model) => {
        if (model.modelId === modelId) {
          return {
            ...model,
            modelName,
          }
        }
        return model
      }),
    )
  }, [])

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
        setSelectedModelIds([])
        setFetchingModels(false)
        return
      }

      try {
        setFetchingModels(true)
        const result = await fetchProviderModels(selectedProviderId)
        if (!cancelled) {
          setModels(result)
          setSelectedModelIds([])
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

  /** 根据已勾选的行构造提交载荷 */
  const buildPayload = () => {
    const selectedSet = new Set(selectedModelIds)
    return models
      .filter((m) => selectedSet.has(m.modelId))
      .map((m) => ({ modelId: m.modelId, alias: m.modelName }))
  }

  const handleOk = async () => {
    if (!selectedProviderId) {
      messageApi.warning('请先选择供应商')
      return
    }
    const items = buildPayload()
    if (items.length === 0) {
      messageApi.warning('请至少勾选一个模型')
      return
    }

    try {
      setSubmitting(true)
      await addModels({ providerId: selectedProviderId, models: items })
      messageApi.success(`成功添加 ${items.length} 个模型`)
      onSuccess?.()
      close()
    } catch (error) {
      logger.error('添加模型失败:', error)
      messageApi.error('添加模型失败')
    } finally {
      setSubmitting(false)
    }
  }

  const columns: ColumnsType<ModelInfo> = [
    {
      title: '模型ID',
      dataIndex: 'modelId',
      key: 'modelId',
      // 动态生成筛选项
      filters: modelIdFilters,
      filterMode: 'tree',
      filterSearch: modelIdFilterSearch,
      // 本地过滤函数
      onFilter: (value, record) => record.modelId === value,
      // 支持多选
      filterMultiple: true,
      filterDropdownProps: {
        classNames: {
          root: 'min-w-90!',
        },
      },
      // 设置列宽以适应筛选图标
      minWidth: 200,
    },
    {
      title: '模型名称',
      dataIndex: 'modelName',
      key: 'modelName',
      render: (value, record) => (
        <InputEditableCell
          size="small"
          value={value}
          validate={(val) => {
            if (!val) {
              throw new Error('模型名称不能为空')
            }
          }}
          onChange={(modelName) => setModelName(record, modelName)}
        />
      ),
    },
  ]

  const emptyText = <Empty description={selectedProviderId ? '无可用模型' : '请选择供应商'} />

  return (
    <>
      {messageContext}
      <Modal
        title="添加模型"
        open={open}
        onCancel={close}
        onOk={handleOk}
        okText="确定"
        cancelText="取消"
        confirmLoading={submitting}
        okButtonProps={{ disabled: selectedModelIds.length === 0 }}
        afterClose={resetState}
        destroyOnHidden
        width={800}
      >
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

          <Table<ModelInfo>
            rowKey="modelId"
            columns={columns}
            virtual={true}
            loading={fetchingModels}
            dataSource={models}
            scroll={{ y: 400 }}
            pagination={false}
            locale={{ emptyText }}
            rowSelection={{
              columnWidth: 40,
              selectedRowKeys: selectedModelIds,
              onChange: setSelectedModelIds,
            }}
          ></Table>
        </div>
      </Modal>
    </>
  )
}

export default AddModelModal
