import { DeleteOutlined, ProjectOutlined } from '@ant-design/icons'
import { Button, Divider, Space } from 'antd'
import { Fragment } from 'react'
import { useNavigate } from 'react-router-dom'

import type { Novel, NovelStats } from '@/services/novelService'

import { formatRelativeTime } from '@/utils/date'

interface NovelItemProps {
  novel: Novel
  onDelete?: (novel: Novel) => unknown
}

/**
 * 渲染"最近更新"内容（相对时间 | 第 n 卷第 m 章 标题）
 *
 * 无统计信息或无最近更新时返回"暂未创建章节"
 */
const renderLastUpdatedText = (stats?: NovelStats): React.ReactNode => {
  if (!stats || !stats.lastUpdatedAt) {
    return '暂未创建章节'
  }

  const relative = formatRelativeTime(stats.lastUpdatedAt)
  const volumeSeq = stats.lastUpdatedVolumeSequence ?? 1
  const chapterSeq = stats.lastUpdatedChapterSequence ?? 0
  const title = stats.lastUpdatedChapterTitle ?? ''

  return `${relative} | 第 ${volumeSeq} 卷第 ${chapterSeq} 章 ${title}`
}

/**
 * 小说项目组件
 * 用于展示单个小说的基本信息（含章节数、总字数、最近更新等统计信息）
 */
const NovelItem: React.FC<NovelItemProps> = ({ novel, onDelete }) => {
  const navigate = useNavigate()

  const chapterCount = novel.stats?.chapterCount ?? 0
  const totalWordCount = novel.stats?.totalWordCount ?? 0

  return (
    <div className="p-4! bg-gray-50 rounded-md">
      <div className="flex gap-4">
        {/* 左边封面 */}
        <div className="w-20 aspect-3/4 bg-gray-200 rounded-md"></div>
        {/* 右边信息 */}
        <div className="flex-1 w-0 flex flex-col gap-1">
          {/* 标题和更新信息 */}
          <div className="flex-1 h-0 flex flex-col gap-1">
            <div className="text-lg font-semibold">{novel.title}</div>
            <div className="text-gray-400 truncate">
              <Fragment>最近更新：</Fragment>
              <Fragment>{renderLastUpdatedText(novel.stats)}</Fragment>
            </div>
          </div>

          {/* 底部 */}
          <div className="flex items-center justify-between">
            <Space className="text-gray-400">
              <span>{chapterCount} 章</span>
              <Divider orientation="vertical" />
              <span>{totalWordCount} 字</span>
            </Space>

            <Space>
              <Button
                type="primary"
                shape="round"
                onClick={() => navigate(`/creation/${novel.id}`)}
                icon={<ProjectOutlined />}
              >
                作品详情
              </Button>

              <Button
                variant="solid"
                color="danger"
                shape="round"
                icon={<DeleteOutlined />}
                onClick={() => onDelete?.(novel)}
              >
                删除
              </Button>
            </Space>
          </div>
        </div>
      </div>
    </div>
  )
}

export default NovelItem
