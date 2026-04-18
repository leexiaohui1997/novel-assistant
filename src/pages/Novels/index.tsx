import { PlusOutlined } from '@ant-design/icons'
import { Button, Typography } from 'antd'
import { useState } from 'react'

import CreateNovelModal from './components/CreateNovelModal'

import './index.css'

const { Title } = Typography

const Novels: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)

  const handleCreate = () => {
    setIsModalOpen(true)
  }

  const handleModalClose = () => {
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

      <div className="novels-content">{/* 内容区域暂不实现 */}</div>

      <CreateNovelModal open={isModalOpen} onClose={handleModalClose} />
    </div>
  )
}

export default Novels
