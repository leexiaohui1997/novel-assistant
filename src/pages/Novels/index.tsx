import { PlusOutlined } from '@ant-design/icons'
import { Button, Typography } from 'antd'
import { useState, useEffect, useCallback } from 'react'

import CreateNovelModal from './components/CreateNovelModal'

import { getNovels, type Novel } from '@/services/novelService'
import { logger } from '@/utils/logger'

import './index.css'

const { Title } = Typography

const Novels: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [novels, setNovels] = useState<Novel[]>([])

  /**
   * 加载小说列表数据
   * 使用 requestAnimationFrame 优化状态更新时机，避免阻塞首屏渲染
   */
  const loadNovels = useCallback(async () => {
    try {
      const data = await getNovels()
      requestAnimationFrame(() => setNovels(data))
    } catch (error) {
      logger.error('加载小说列表失败:', error)
    }
  }, [])

  // 组件挂载时初始化数据
  useEffect(() => {
    void loadNovels()
  }, [loadNovels])

  const handleCreate = () => setIsModalOpen(true)

  const handleModalClose = () => setIsModalOpen(false)

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
        <div style={{ marginTop: 16 }}>
          <Typography.Text>当前共有 {novels.length} 部作品</Typography.Text>
        </div>
      </div>

      <CreateNovelModal open={isModalOpen} onClose={handleModalClose} onSuccess={loadNovels} />
    </div>
  )
}

export default Novels
