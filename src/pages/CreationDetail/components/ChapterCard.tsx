import { Card, Tabs } from 'antd'
import React, { useCallback, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'

import ChapterManagement from './ChapterManagement'
import DraftBox from './DraftBox'

const ChapterCard: React.FC = () => {
  const [searchParams, setSearchParams] = useSearchParams()

  const activeKey = useMemo(() => {
    const key = searchParams.get('tab') || 'chapters'
    return ['chapters', 'drafts'].includes(key) ? key : 'chapters'
  }, [searchParams])

  const setActiveKey = useCallback(
    (key: string) => {
      setSearchParams({ tab: key })
    },
    [setSearchParams],
  )

  const renderedTitle = useMemo(
    () => (
      <Tabs
        activeKey={activeKey}
        onChange={setActiveKey}
        items={[
          {
            key: 'chapters',
            label: '章节管理',
          },
          {
            key: 'drafts',
            label: '草稿箱',
          },
        ]}
      ></Tabs>
    ),
    [activeKey, setActiveKey],
  )

  return (
    <Card
      title={renderedTitle}
      classNames={{
        header: 'border-b-0!',
      }}
    >
      {activeKey === 'chapters' ? <ChapterManagement /> : <DraftBox />}
    </Card>
  )
}

export default ChapterCard
