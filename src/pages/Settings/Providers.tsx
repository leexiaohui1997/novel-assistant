import { PlusOutlined } from '@ant-design/icons'
import {
  Button,
  Card,
  Form,
  FormInstance,
  Input,
  message,
  Modal,
  Select,
  Space,
  Switch,
  Table,
  Tag,
} from 'antd'
import React, { useEffect, useRef, useState } from 'react'

import type { ColumnsType } from 'antd/es/table'

import {
  Provider,
  ProviderTypeInfo,
  createProvider,
  deleteProvider,
  getProviderTypes,
  getProvidersWithPagination,
  updateProvider,
} from '@/services/providerService'
import { formatDateTime } from '@/utils/date'
import { logger } from '@/utils/logger'

/** 供应商表单字段值 */
export interface ProviderFormValues {
  name: string
  baseUrl: string
  apiKey: string
  modelFetchType: string
  isEnabled: boolean
}

/** API Key 脱敏显示 */
const maskApiKey = (key?: string): string => {
  if (!key) return '-'
  if (key.length <= 6) return 'sk-****'
  return `sk-****${key.slice(-4)}`
}

/** 获取供应商拉取类型列表 */
const useProviderTypes = () => {
  const [providerTypes, setProviderTypes] = useState<ProviderTypeInfo[]>([])
  // 加载供应商拉取类型列表
  useEffect(() => {
    let cancelled = false
    const loadTypes = async () => {
      try {
        const types = await getProviderTypes()
        if (!cancelled) setProviderTypes(types)
      } catch (error) {
        logger.error('获取供应商拉取类型失败:', error)
      }
    }
    loadTypes()
    return () => {
      cancelled = true
    }
  }, [])
  return providerTypes
}

/** 添加/编辑供应商弹窗 */
const AddProviderModal: React.FC<{
  open: boolean
  editingProvider?: Provider | null
  onClose: () => void
  onSuccess: () => void
}> = ({ open, editingProvider, onClose, onSuccess }) => {
  const [messageApi, contextHolder] = message.useMessage()
  const [submitting, setSubmitting] = useState(false)
  const formRef = useRef<FormInstance<ProviderFormValues>>(null)
  const providerTypes = useProviderTypes()

  const isEdit = !!editingProvider

  const handleOk = async () => {
    if (!formRef.current) return
    try {
      const values = await formRef.current.validateFields()
      setSubmitting(true)

      const params = {
        name: values.name,
        baseUrl: values.baseUrl,
        apiKey: values.apiKey || undefined,
        modelFetchType: values.modelFetchType || 'default',
        isEnabled: values.isEnabled,
      }

      if (isEdit && editingProvider) {
        await updateProvider(editingProvider.id, params)
        messageApi.success('供应商更新成功')
      } else {
        await createProvider(params)
        messageApi.success('供应商创建成功')
      }

      onSuccess()
      onClose()
    } catch (error) {
      if (error instanceof Error) {
        logger.error('供应商保存失败:', error)
        messageApi.error('供应商保存失败')
      }
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <>
      {contextHolder}
      <Modal
        title={isEdit ? '编辑供应商' : '添加供应商'}
        open={open}
        onCancel={onClose}
        afterClose={() => formRef.current?.resetFields()}
        destroyOnHidden
        confirmLoading={submitting}
        footer={[
          <Button key="cancel" onClick={onClose} disabled={submitting}>
            取消
          </Button>,
          <Button key="submit" type="primary" loading={submitting} onClick={handleOk}>
            确认
          </Button>,
        ]}
      >
        <Form
          ref={formRef}
          layout="vertical"
          initialValues={
            editingProvider
              ? {
                  name: editingProvider.name,
                  baseUrl: editingProvider.baseUrl,
                  apiKey: editingProvider.apiKey || '',
                  modelFetchType: editingProvider.modelFetchType || 'default',
                  isEnabled: editingProvider.isEnabled,
                }
              : { isEnabled: true, modelFetchType: 'default' }
          }
        >
          <Form.Item
            label="供应商名称"
            name="name"
            rules={[{ required: true, message: '请输入供应商名称' }]}
          >
            <Input placeholder="如 OpenAI、DeepSeek、Ollama" />
          </Form.Item>

          <Form.Item
            label="API 地址"
            name="baseUrl"
            rules={[{ required: true, message: '请输入 API 地址' }]}
          >
            <Input placeholder="如 https://api.openai.com/v1" />
          </Form.Item>

          <Form.Item label="API Key" name="apiKey">
            <Input.Password placeholder="请输入 API Key" />
          </Form.Item>

          <Form.Item
            label="模型拉取策略"
            name="modelFetchType"
            rules={[{ required: true, message: '请选择模型拉取策略' }]}
          >
            <Select
              placeholder="请选择模型拉取策略"
              options={providerTypes.map((t) => ({
                key: t.id,
                value: t.id,
                label: t.name,
              }))}
            />
          </Form.Item>

          <Form.Item label="启用状态" name="isEnabled" valuePropName="checked">
            <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          </Form.Item>
        </Form>
      </Modal>
    </>
  )
}

/** 供应商管理页面 */
const ProviderManage: React.FC = () => {
  const [messageApi, contextHolder] = message.useMessage()
  const [modalApi, contextHolderModal] = Modal.useModal()
  const [providers, setProviders] = useState<Provider[]>([])
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [loading, setLoading] = useState(false)
  const [refreshCounter, setRefreshCounter] = useState(0)
  const [modalOpen, setModalOpen] = useState(false)
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null)
  const providerTypes = useProviderTypes()

  /** 手动刷新列表（增删改后调用） */
  const refreshList = () => setRefreshCounter((c) => c + 1)

  useEffect(() => {
    let cancelled = false

    const loadData = async () => {
      setLoading(true)
      try {
        const result = await getProvidersWithPagination(page, pageSize)
        if (!cancelled) {
          setProviders(result.data)
          setTotal(result.total)
        }
      } catch (error) {
        if (!cancelled) {
          logger.error('获取供应商列表失败:', error)
          messageApi.error('获取供应商列表失败')
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    loadData()
    return () => {
      cancelled = true
    }
  }, [page, pageSize, refreshCounter, messageApi])

  const handleAdd = () => {
    setEditingProvider(null)
    setModalOpen(true)
  }

  const handleEdit = (provider: Provider) => {
    setEditingProvider(provider)
    setModalOpen(true)
  }

  const handleDelete = (provider: Provider) => {
    modalApi.confirm({
      title: '确认删除',
      content: `确定要删除供应商「${provider.name}」吗？该操作将同时删除其下所有模型。`,
      okText: '确认',
      cancelText: '取消',
      onOk: async () => {
        try {
          await deleteProvider(provider.id)
          messageApi.success('供应商删除成功')
          refreshList()
        } catch (error) {
          logger.error('删除供应商失败:', error)
          messageApi.error('删除供应商失败')
        }
      },
    })
  }

  const handleModelManage = () => {
    messageApi.info('暂未实现')
  }

  const handleModalClose = () => {
    setModalOpen(false)
    setEditingProvider(null)
  }

  const columns: ColumnsType<Provider> = [
    {
      title: '供应商名称',
      dataIndex: 'name',
      key: 'name',
      width: 140,
    },
    {
      title: 'API 地址',
      dataIndex: 'baseUrl',
      key: 'baseUrl',
      ellipsis: true,
      width: 220,
    },
    {
      title: 'API Key',
      dataIndex: 'apiKey',
      key: 'apiKey',
      width: 140,
      render: (key?: string) => maskApiKey(key),
    },
    {
      title: '拉取策略',
      dataIndex: 'modelFetchType',
      key: 'modelFetchType',
      width: 140,
      render: (val: string) => {
        const typeName = providerTypes.find((t) => t.id === val)?.name || '-'
        return <Tag color="blue">{typeName}</Tag>
      },
    },
    {
      title: '启用状态',
      dataIndex: 'isEnabled',
      key: 'isEnabled',
      width: 90,
      render: (enabled: boolean) =>
        enabled ? <Tag color="green">启用</Tag> : <Tag color="default">禁用</Tag>,
    },
    {
      title: '创建时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      width: 180,
      render: (val: string) => formatDateTime(val),
    },
    {
      title: '操作',
      key: 'action',
      width: 200,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          <Button type="link" size="small" onClick={handleModelManage}>
            模型管理
          </Button>
          <Button type="link" size="small" onClick={() => handleEdit(record)}>
            编辑
          </Button>
          <Button type="link" size="small" danger onClick={() => handleDelete(record)}>
            删除
          </Button>
        </Space>
      ),
    },
  ]

  return (
    <>
      {contextHolder}
      {contextHolderModal}
      <Card
        title="供应商管理"
        extra={
          <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
            添加供应商
          </Button>
        }
      >
        <Table<Provider>
          rowKey="id"
          columns={columns}
          dataSource={providers}
          loading={loading}
          scroll={{ x: 990 }}
          pagination={{
            current: page,
            pageSize,
            total,
            showSizeChanger: true,
            onChange: (p, ps) => {
              setPage(p)
              setPageSize(ps)
            },
          }}
        />
      </Card>

      <AddProviderModal
        open={modalOpen}
        editingProvider={editingProvider}
        onClose={handleModalClose}
        onSuccess={refreshList}
      />
    </>
  )
}

export default ProviderManage
