import { Card } from 'antd'
import React from 'react'

import BasicInfoCard from '../components/BasicInfoCard'
import ChapterManagement from '../components/ChapterManagement'

const CreationDetailBasic: React.FC = () => {
  return (
    <div className="p-6! flex flex-col gap-4">
      <BasicInfoCard />
      <Card title="章节管理">
        <ChapterManagement />
      </Card>
    </div>
  )
}

export default CreationDetailBasic
