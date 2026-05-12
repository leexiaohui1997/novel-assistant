import { Card, Table, Tooltip } from 'antd'
import { useCallback } from 'react'

import TermChapterDesc from '../components/TermChapterDesc'

import { useCreationState } from '@/hooks/useCreationState'
import { useRefresh } from '@/hooks/useRefresh'
import {
  ALL_TERM_TYPES,
  getTermsByNovel,
  TERM_TYPE_LABELS,
  TermType,
} from '@/services/termsService'
import { formatDateTime } from '@/utils/date'

export default function CreationDetailTerms() {
  const { novelId } = useCreationState()

  const refreshFn = useCallback(() => getTermsByNovel({ novelId }), [novelId])
  const { data, loading } = useRefresh({
    refreshFn,
    initialData: [],
  })

  return (
    <div className="p-6">
      <Card title="名词管理">
        <Table
          rowKey="id"
          loading={loading}
          dataSource={data}
          expandable={{
            expandedRowRender: ({ id }) => <TermChapterDesc termId={id} />,
          }}
          pagination={{
            showSizeChanger: true,
            defaultPageSize: 8,
            pageSizeOptions: [8, 10, 20, 50],
          }}
          columns={[
            {
              title: '序号',
              key: 'index',
              width: 60,
              align: 'center',
              fixed: 'left',
              render: (_, record) => data.indexOf(record) + 1,
            },
            Table.EXPAND_COLUMN,
            {
              title: '名词',
              dataIndex: 'name',
              width: 200,
            },
            {
              title: '类型',
              dataIndex: 'termType',
              width: 160,
              filters: ALL_TERM_TYPES.map((item) => ({
                value: item,
                text: TERM_TYPE_LABELS[item],
              })),
              onFilter: (value, record) => record.termType === value,
              render: (val: TermType) => TERM_TYPE_LABELS[val],
            },
            {
              title: '描述',
              dataIndex: 'description',
              ellipsis: { showTitle: false },
              render: (val: string) => (
                <Tooltip placement="topLeft" title={val}>
                  {val}
                </Tooltip>
              ),
            },
            {
              title: '创建时间',
              dataIndex: 'createdAt',
              width: 180,
              render: (val: string) => formatDateTime(val),
              sorter: (a, b) => a.createdAt.localeCompare(b.createdAt),
              showSorterTooltip: { target: 'sorter-icon' },
            },
            {
              title: '更新时间',
              dataIndex: 'updatedAt',
              width: 180,
              render: (val: string) => formatDateTime(val),
              sorter: (a, b) => a.createdAt.localeCompare(b.createdAt),
            },
          ]}
        />
      </Card>
    </div>
  )
}
