import { Spin, Pagination, Form, Button, Empty } from 'antd'
import React, { useState, useCallback, useEffect, useRef, useImperativeHandle } from 'react'

import type { FormInstance } from 'antd'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

interface ListProps<T> {
  /**
   * 获取列表数据的函数
   * @param page - 当前页码
   * @param pageSize - 每页大小
   * @param filters - 筛选条件
   * @returns Promise<{ data: T[], total: number }>
   */
  fetchList: (
    page: number,
    pageSize: number,
    filters?: Record<string, unknown>,
  ) => Promise<PaginatedResult<T>>

  /**
   * 渲染每一项的函数
   * @param item - 列表项数据
   * @param index - 索引
   * @returns ReactNode
   */
  renderItem: (item: T, index: number, order: number) => React.ReactNode

  /**
   * 渲染筛选表单的函数
   * @param formRef - Ant Design Form 的 ref
   * @returns ReactNode
   */
  renderFilters?: (formRef: React.RefObject<FormInstance>) => React.ReactNode

  /**
   * 每页显示数量，默认 10
   */
  pageSize?: number

  /**
   * 初始页码，默认 1
   */
  initialPage?: number

  classNames?: {
    list?: string
    item?: string
  }

  /**
   * 空状态的描述文本
   */
  emptyDescription?: string
}

/**
 * List 组件暴露的命令式方法
 */
export interface ListRef {
  /** 刷新列表数据（保持当前页码和筛选条件） */
  refresh: () => void
}

/**
 * 通用列表组件
 * 支持分页、加载状态、筛选等基础功能
 */
function List<T>(props: ListProps<T>, ref: React.ForwardedRef<ListRef>) {
  const {
    fetchList,
    renderItem,
    renderFilters,
    pageSize = 10,
    initialPage = 1,
    classNames,
    emptyDescription = '暂无数据',
  } = props
  const [data, setData] = useState<T[]>([])
  const [total, setTotal] = useState(0)
  const [currentPage, setCurrentPage] = useState(initialPage)
  const [loading, setLoading] = useState(false)
  const [filters, setFilters] = useState<Record<string, unknown>>({})
  const formRef = useRef<FormInstance>(null)

  /**
   * 执行数据加载（带 isMounted 检查的安全版本）
   */
  const executeFetch = useCallback(
    async (
      page: number,
      currentFilters?: Record<string, unknown>,
      isMountedCheck?: () => boolean,
    ) => {
      setLoading(true)
      try {
        const result = await fetchList(page, pageSize, currentFilters || filters)
        if (!isMountedCheck || isMountedCheck()) {
          setData(result.data)
          setTotal(result.total)
          setCurrentPage(page)
        }
      } catch (error) {
        logger.error('加载列表数据失败:', error)
      } finally {
        if (!isMountedCheck || isMountedCheck()) {
          setLoading(false)
        }
      }
    },
    [fetchList, pageSize, filters],
  )

  /**
   * 加载列表数据（用户主动触发，无需 isMounted 检查）
   */
  const loadData = useCallback(
    async (page: number, currentFilters?: Record<string, unknown>) => {
      await executeFetch(page, currentFilters)
    },
    [executeFetch],
  )

  /**
   * 对外暴露命令式方法：refresh
   */
  useImperativeHandle(
    ref,
    () => ({
      refresh: () => {
        void loadData(currentPage, filters)
      },
    }),
    [loadData, currentPage, filters],
  )

  // 初始化加载数据（需要 isMounted 检查）
  useEffect(() => {
    let isMounted = true

    const init = async () => {
      await executeFetch(initialPage, undefined, () => isMounted)
    }

    void init()

    return () => {
      isMounted = false
    }
  }, [executeFetch, initialPage])

  /**
   * 处理页码变化
   */
  const handlePageChange = (page: number) => {
    void loadData(page)
  }

  /**
   * 处理筛选条件变化
   */
  const handleFilterChange = () => {
    const values = formRef.current?.getFieldsValue() || {}
    setFilters(values)
    // 筛选时重置到第一页
    void loadData(1, values)
  }

  /**
   * 处理表单提交（点击搜索按钮时）
   */
  const handleFilterSubmit = () => {
    handleFilterChange()
  }

  /**
   * 处理表单重置
   */
  const handleFilterReset = () => {
    formRef.current?.resetFields()
    const values = formRef.current?.getFieldsValue() || {}
    setFilters(values)
    void loadData(1, values)
  }

  return (
    <div className="generic-list">
      {renderFilters && (
        <Form ref={formRef} onFinish={handleFilterSubmit} style={{ marginBottom: 16 }}>
          {renderFilters(formRef)}
          <Form.Item>
            <div style={{ display: 'flex', gap: 8 }}>
              <Button type="primary" htmlType="submit">
                搜索
              </Button>
              <Button onClick={handleFilterReset}>重置</Button>
            </div>
          </Form.Item>
        </Form>
      )}

      <Spin spinning={loading}>
        {data.length === 0 ? (
          <div className="p-6!">
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={emptyDescription} />
          </div>
        ) : (
          <div className={classNames?.list}>
            {data.map((item, index) => (
              <div key={index} className={classNames?.item}>
                {renderItem(item, index, (currentPage - 1) * pageSize + index + 1)}
              </div>
            ))}
          </div>
        )}
      </Spin>

      {total > pageSize && (
        <div style={{ marginTop: 16, textAlign: 'center' }}>
          <Pagination
            current={currentPage}
            pageSize={pageSize}
            total={total}
            onChange={handlePageChange}
            showSizeChanger={false}
          />
        </div>
      )}
    </div>
  )
}

const ForwardedList = React.forwardRef(List)
ForwardedList.displayName = 'List'

// 添加泛型支持
const ListWithGenerics = ForwardedList as <T>(
  props: ListProps<T> & { ref?: React.ForwardedRef<ListRef> },
) => ReturnType<typeof List>

export default ListWithGenerics
