import { PlusOutlined } from '@ant-design/icons'
import { Button, Typography } from 'antd'
import { useState } from 'react'

import CreateNovelModal from './components/CreateNovelModal'
import NovelItem from './components/NovelItem'

import List from '@/components/List'
import { getNovelsWithPagination, type Novel } from '@/services/novelService'

import './index.css'

const { Title } = Typography

const Novels: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)

  /**
   * 获取小说列表数据（适配 List 组件接口）
   */
  const fetchNovelList = async (page: number, pageSize: number) => {
    try {
      // 调用后端分页接口
      return getNovelsWithPagination(page, pageSize)
    } catch {
      return {
        data: [],
        total: 0,
      }
    }
  }

  const handleCreate = () => setIsModalOpen(true)

  const handleModalClose = () => setIsModalOpen(false)

  const handleSuccess = () => {
    // List 组件会自动重新获取数据
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
          fetchList={fetchNovelList}
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
