import React from 'react'

import BasicInfoCard from '../components/BasicInfoCard'
import ChapterCard from '../components/ChapterCard'

const CreationDetailBasic: React.FC = () => {
  return (
    <div className="p-6! flex flex-col gap-4">
      <BasicInfoCard />
      <ChapterCard />
    </div>
  )
}

export default CreationDetailBasic
