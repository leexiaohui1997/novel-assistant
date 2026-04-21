import { Button, Divider, Space } from 'antd'
import { Fragment } from 'react'

import type { Novel } from '@/services/novelService'

interface NovelItemProps {
  novel: Novel
}

/**
 * 小说项目组件
 * 用于展示单个小说的基本信息
 */
const NovelItem: React.FC<NovelItemProps> = ({ novel }) => {
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
            <div className="text-gray-400">
              <Fragment>最近更新：</Fragment>
              <Fragment>暂未创建章节</Fragment>
            </div>
          </div>

          {/* 底部 */}
          <div className="flex items-center justify-between">
            <Space className="text-gray-400">
              <span>0 章</span>
              <Divider orientation="vertical" />
              <span>0 字</span>
            </Space>

            <Space>
              <Button type="primary" shape="round">
                开始创作
              </Button>
            </Space>
          </div>
        </div>
      </div>
    </div>
  )
}

export default NovelItem
