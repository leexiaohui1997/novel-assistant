import { Card } from 'antd'
import React from 'react'

import ChapterManagement from '../components/ChapterManagement'

const CreationDetailArticle: React.FC = () => {
  return (
    <div className="p-6! flex flex-col gap-4">
      <Card title="章节管理">
        <ChapterManagement />
      </Card>
    </div>
  )
}

export default CreationDetailArticle
