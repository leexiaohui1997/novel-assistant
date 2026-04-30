import { DeleteOutlined, PlusOutlined } from '@ant-design/icons'
import { Button, Card, message, Modal, Switch, Table, Tag } from 'antd'
import React, { useEffect, useRef, useState } from 'react'

import AddModelModal, { AddModelModalHandle } from './AddModelModal'

import type { ColumnsType } from 'antd/es/table'

import { InputEditableCell } from '@/components/EditableCell/Input'
import {
  deleteModel,
  getModelsWithPagination,
  Model,
  toggleModelEnabled,
  updateModelAlias,
} from '@/services/modelService'
import { formatDateTime } from '@/utils/date'
import { logger } from '@/utils/logger'

const ModelManage: React.FC = () => {
  const addModalRef = useRef<AddModelModalHandle>(null)
  const [messageApi, messageContext] = message.useMessage()
  const [modalApi, modalContext] = Modal.useModal()

  const [models, setModels] = useState<Model[]>([])
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [loading, setLoading] = useState(false)
  const [refreshCounter, setRefreshCounter] = useState(0)

  /** 手动刷新列表（增删改后调用） */
  const refreshList = () => setRefreshCounter((c) => c + 1)

  useEffect(() => {
    let cancelled = false

    const loadData = async () => {
      setLoading(true)
      try {
        const result = await getModelsWithPagination(page, pageSize)
        if (!cancelled) {
          setModels(result.data)
          setTotal(result.total)
        }
      } catch (error) {
        if (!cancelled) {
          logger.error('获取模型列表失败:', error)
          messageApi.error('获取模型列表失败')
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    void loadData()
    return () => {
      cancelled = true
    }
  }, [page, pageSize, refreshCounter, messageApi])

  const handleToggleEnabled = async (record: Model, checked: boolean) => {
    try {
      await toggleModelEnabled(record.id, checked)
      messageApi.success(checked ? '已启用' : '已禁用')
      refreshList()
    } catch (error) {
      logger.error('切换启用状态失败:', error)
      messageApi.error('切换启用状态失败')
    }
  }

  const handleUpdateAlias = async (record: Model, alias: string) => {
    await updateModelAlias(record.id, alias)
    refreshList()
  }

  const handleDelete = (record: Model) => {
    modalApi.confirm({
      title: '确认删除',
      content: `确定要删除模型「${record.alias}」吗？删除后不可恢复。`,
      okText: '确认',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        try {
          await deleteModel(record.id)
          messageApi.success('模型删除成功')
          refreshList()
        } catch (error) {
          logger.error('删除模型失败:', error)
          messageApi.error('删除模型失败')
        }
      },
    })
  }

  const columns: ColumnsType<Model> = [
    {
      title: '供应商',
      dataIndex: 'providerName',
      key: 'providerName',
      width: 160,
      render: (val: string) => <Tag color="blue">{val}</Tag>,
    },
    {
      title: '模型ID',
      dataIndex: 'modelId',
      key: 'modelId',
      ellipsis: true,
      width: 300,
    },
    {
      title: '名称',
      dataIndex: 'alias',
      key: 'alias',
      width: 300,
      render: (value: string, record) => (
        <InputEditableCell
          size="small"
          value={value}
          validate={(val) => {
            if (!val) {
              throw new Error('名称不能为空')
            }
          }}
          onChange={(alias) => handleUpdateAlias(record, alias)}
        />
      ),
    },
    {
      title: '是否启用',
      dataIndex: 'isEnabled',
      key: 'isEnabled',
      width: 110,
      render: (enabled: boolean, record) => (
        <Switch
          checked={enabled}
          checkedChildren="启用"
          unCheckedChildren="禁用"
          onChange={(checked) => handleToggleEnabled(record, checked)}
        />
      ),
    },
    {
      title: '添加时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      width: 180,
      render: (val: string) => formatDateTime(val),
    },
    {
      title: '操作',
      key: 'action',
      width: 100,
      fixed: 'right',
      render: (_, record) => (
        <Button
          type="link"
          size="small"
          danger
          icon={<DeleteOutlined />}
          onClick={() => handleDelete(record)}
        >
          删除
        </Button>
      ),
    },
  ]

  const renderedExtra = (
    <Button type="primary" icon={<PlusOutlined />} onClick={() => addModalRef.current?.show()}>
      添加模型
    </Button>
  )

  return (
    <>
      {messageContext}
      {modalContext}
      <Card title="模型管理" extra={renderedExtra}>
        <Table<Model>
          rowKey="id"
          columns={columns}
          dataSource={models}
          loading={loading}
          scroll={{ x: 1030 }}
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
      <AddModelModal ref={addModalRef} onSuccess={refreshList} />
    </>
  )
}

export default ModelManage
