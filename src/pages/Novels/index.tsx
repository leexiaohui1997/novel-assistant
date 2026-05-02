import { PlusOutlined } from '@ant-design/icons'
import { Button, Typography } from 'antd'
import { useState, useRef } from 'react'

import CreateNovelModal from './components/CreateNovelModal'
import NovelItem from './components/NovelItem'

import List, { type ListRef } from '@/components/List'
import { getNovelsWithPagination, type Novel } from '@/services/novelService'

import './index.css'

const { Title } = Typography

const Novels: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const listRef = useRef<ListRef>(null)

  const handleCreate = () => setIsModalOpen(true)

  const handleModalClose = () => setIsModalOpen(false)

  const handleSuccess = () => {
    // 刷新列表数据
    listRef.current?.refresh()
    setIsModalOpen(false)
  }

  return (
    <div className="novels-page">
      <div className="novels-header">
        <Title level={4} style={{ margin: 0 }}>
          作品管理
        </Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
          创建作品
        </Button>
      </div>

      <div className="novels-content">
        <List<Novel>
          ref={listRef}
          fetchList={getNovelsWithPagination}
          renderItem={(novel) => <NovelItem novel={novel} />}
          pageSize={10}
          classNames={{
            list: 'flex flex-col gap-4',
          }}
        />
      </div>

      <CreateNovelModal open={isModalOpen} onClose={handleModalClose} onSuccess={handleSuccess} />
    </div>
  )
}

export default Novels
