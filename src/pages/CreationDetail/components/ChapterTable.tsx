import { Button, Popconfirm, Space, Table, message } from 'antd'
import { Ref, useCallback, useEffect, useImperativeHandle, useMemo, useState } from 'react'

import type { ColumnsType, TablePaginationConfig } from 'antd/es/table'
import type { FilterValue, SorterResult } from 'antd/es/table/interface'

import { useCreationState } from '@/hooks/useCreationState'
import {
  type Chapter,
  type ChapterQuery,
  deleteChapter,
  getChaptersWithPagination,
} from '@/services/chapterService'
import { formatDateTime } from '@/utils/date'
import { logger } from '@/utils/logger'

/**
 * ChapterTable 对外暴露的命令式方法
 *
 * 通过 `ref` 获取实例后调用，典型场景：父组件在新建 / 保存章节成功后手动刷新列表。
 */
export interface ChapterTableHandle {
  /** 按当前分页 / 排序条件重新拉取一次列表数据 */
  refresh: () => void
}

export interface ChapterTableProps {
  /** 分卷序号；草稿模式下忽略。默认 1 */
  volumeSequence?: number
  /** 是否草稿模式。默认 false */
  isDraft?: boolean
  /** 组件实例 ref，用于外部命令式调用 */
  ref?: Ref<ChapterTableHandle>
}

type SortField = NonNullable<ChapterQuery['sortField']>
type SortOrder = NonNullable<ChapterQuery['sortOrder']>

/** Antd Table 的排序方向映射到后端 sort_order */
const mapAntdOrder = (order: 'ascend' | 'descend' | null | undefined): SortOrder | undefined => {
  if (order === 'ascend') return 'asc'
  if (order === 'descend') return 'desc'
  return undefined
}

/** Antd Table 默认排序方向（仅用于列定义的 defaultSortOrder） */
type AntdOrder = 'ascend' | 'descend'

/** 从未知错误中提取可展示的错误消息 */
const resolveErrorMessage = (err: unknown, fallback: string): string => {
  if (typeof err === 'string' && err.trim()) return err
  if (err instanceof Error && err.message) return err.message
  return fallback
}

const PAGE_SIZE = 10

const ChapterTable: React.FC<ChapterTableProps> = ({
  volumeSequence = 1,
  isDraft = false,
  ref,
}) => {
  const { novelId } = useCreationState()
  const [messageApi, contextHolder] = message.useMessage()

  const [data, setData] = useState<Chapter[]>([])
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(1)
  const [loading, setLoading] = useState(false)
  // 默认排序：草稿按创建时间倒排，非草稿按序号倒排
  const defaultSortField: SortField = isDraft ? 'createdAt' : 'sequence'
  const [sortField, setSortField] = useState<SortField>(defaultSortField)
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc')

  /** 加载列表数据 */
  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const query: ChapterQuery = {
        isDraft,
        volumeSequence: isDraft ? undefined : volumeSequence,
        sortField,
        sortOrder,
      }
      const result = await getChaptersWithPagination(novelId, page, PAGE_SIZE, query)
      setData(result.data)
      setTotal(result.total)
    } catch (e) {
      logger.error('加载章节列表失败:', e)
      messageApi.error('加载章节列表失败')
    } finally {
      setLoading(false)
    }
  }, [novelId, page, sortField, sortOrder, isDraft, volumeSequence, messageApi])

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    loadData()
  }, [loadData])

  /** 对外暴露命令式方法：refresh */
  useImperativeHandle(ref, () => ({ refresh: loadData }), [loadData])

  /** 处理 Table 的分页 / 排序变化 */
  const handleTableChange = useCallback(
    (
      pagination: TablePaginationConfig,
      _filters: Record<string, FilterValue | null>,
      sorter: SorterResult<Chapter> | SorterResult<Chapter>[],
    ) => {
      const current = Array.isArray(sorter) ? sorter[0] : sorter
      const nextOrder = mapAntdOrder(current?.order)
      if (nextOrder && current?.field) {
        setSortField(current.field as SortField)
        setSortOrder(nextOrder)
      } else {
        // 未选排序时回退到默认
        setSortField(defaultSortField)
        setSortOrder('desc')
      }
      setPage(pagination.current ?? 1)
    },
    [defaultSortField],
  )

  /** 删除章节 */
  const handleDelete = useCallback(
    async (chapterId: string) => {
      try {
        await deleteChapter(novelId, chapterId)
        messageApi.success('删除成功')
        await loadData()
      } catch (e) {
        logger.error('删除章节失败:', e)
        messageApi.error(resolveErrorMessage(e, '删除失败，请重试'))
      }
    },
    [novelId, loadData, messageApi],
  )

  const columns = useMemo<ColumnsType<Chapter>>(() => {
    // 默认排序方向：defaultSortField 默认 desc
    const defaultOrder: AntdOrder = 'descend'

    const sequenceCol = {
      title: '章节序号',
      dataIndex: 'sequence',
      key: 'sequence',
      width: 120,
      sorter: true,
      defaultSortOrder: defaultOrder,
      render: (value: number) => `第${value}章`,
    }

    const baseCols: ColumnsType<Chapter> = [
      {
        title: '章节名称',
        dataIndex: 'title',
        key: 'title',
        sorter: true,
        ellipsis: true,
      },
      { title: '字数', dataIndex: 'wordCount', key: 'wordCount', width: 100 },
      {
        title: '创建时间',
        dataIndex: 'createdAt',
        key: 'createdAt',
        width: 180,
        sorter: true,
        defaultSortOrder: isDraft ? defaultOrder : undefined,
        render: (value: string) => formatDateTime(value),
      },
      {
        title: '更新时间',
        dataIndex: 'updatedAt',
        key: 'updatedAt',
        width: 180,
        sorter: true,
        render: (value: string) => formatDateTime(value),
      },
      {
        title: '操作',
        key: 'actions',
        width: 160,
        render: (_: unknown, record) => (
          <Space>
            <Button type="link" size="small">
              编辑
            </Button>
            {record.deletable && (
              <Popconfirm
                title="确认删除该章节？"
                description="删除后不可恢复，请谨慎操作。"
                okText="删除"
                okButtonProps={{ danger: true }}
                cancelText="取消"
                onConfirm={() => handleDelete(record.id)}
              >
                <Button type="link" size="small" danger>
                  删除
                </Button>
              </Popconfirm>
            )}
          </Space>
        ),
      },
    ]

    return isDraft ? baseCols : [sequenceCol, ...baseCols]
  }, [isDraft, handleDelete])

  return (
    <>
      {contextHolder}
      <Table<Chapter>
        rowKey="id"
        columns={columns}
        dataSource={data}
        loading={loading}
        onChange={handleTableChange}
        pagination={{
          current: page,
          pageSize: PAGE_SIZE,
          total,
          showSizeChanger: false,
          showTotal: (t) => `共 ${t} 条`,
        }}
      />
    </>
  )
}

export default ChapterTable
