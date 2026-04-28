import { Button, Drawer, Empty, Spin, message } from 'antd'
import { useCallback, useEffect, useState } from 'react'

import { getChapterVersions, type ChapterVersion } from '@/services/chapterService'
import { formatDateTime } from '@/utils/date'
import { logger } from '@/utils/logger'

/** 版本列表项点击"应用"的回调 */
export type ApplyVersionHandler = (version: ChapterVersion) => void

/** VersionDrawer 组件属性 */
interface VersionDrawerProps {
  /** 是否显示 */
  open: boolean
  /** 关闭事件 */
  onClose: () => void
  /** 当前编辑的章节 ID（未传时不加载数据） */
  chapterId?: string
  /** "应用" 按钮点击回调 */
  onApply: ApplyVersionHandler
}

/** 单条历史版本卡片属性 */
interface VersionItemProps {
  index: number
  version: ChapterVersion
  onApply: ApplyVersionHandler
}

/**
 * 单条历史版本卡片
 *
 * 展示：序号 + 标题 + 正文（两行省略） + 保存时间 + 应用按钮
 */
const VersionItem: React.FC<VersionItemProps> = ({ index, version, onApply }) => {
  const handleApply = useCallback(() => onApply(version), [version, onApply])
  return (
    <div className="p-3! border border-gray-200 rounded-lg bg-white">
      <div className="flex items-center mb-2! gap-2">
        <span className="text-sm text-gray-400">#{index}</span>
        <span className="flex-1 font-medium text-base truncate" title={version.title || '(未命名)'}>
          {version.title || '(未命名)'}
        </span>
      </div>
      <div
        className="text-sm text-gray-600 mb-3! overflow-hidden"
        style={{
          display: '-webkit-box',
          WebkitLineClamp: 2,
          WebkitBoxOrient: 'vertical',
          wordBreak: 'break-word',
        }}
      >
        {version.content || '(空)'}
      </div>
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-400">{formatDateTime(version.savedAt)}</span>
        <Button size="small" onClick={handleApply}>
          应用
        </Button>
      </div>
    </div>
  )
}

/** 版本列表加载状态 */
interface VersionListState {
  loading: boolean
  versions: ChapterVersion[]
}

/**
 * 历史版本加载 Hook：按需拉取版本列表
 *
 * - `open = false` 或 `chapterId` 为空时不加载
 * - 每次打开时重新拉取最新数据
 */
const useVersionList = (open: boolean, chapterId?: string): VersionListState => {
  const [state, setState] = useState<VersionListState>({ loading: false, versions: [] })

  useEffect(() => {
    if (!open || !chapterId) return
    let cancelled = false
    const load = async () => {
      setState({ loading: true, versions: [] })
      try {
        const list = await getChapterVersions(chapterId)
        if (!cancelled) setState({ loading: false, versions: list })
      } catch (e) {
        logger.error('加载历史版本失败:', e)
        if (!cancelled) {
          setState({ loading: false, versions: [] })
          message.error('加载历史版本失败')
        }
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [open, chapterId])

  return state
}

/** Drawer Body 内容：根据加载状态展示骨架 / 空态 / 列表 */
interface VersionDrawerBodyProps {
  loading: boolean
  versions: ChapterVersion[]
  onApply: ApplyVersionHandler
}

const VersionDrawerBody: React.FC<VersionDrawerBodyProps> = ({ loading, versions, onApply }) => {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-10">
        <Spin />
      </div>
    )
  }
  if (versions.length === 0) {
    return <Empty description="暂无历史版本" className="mt-10" />
  }
  return (
    <div className="flex flex-col gap-4">
      {versions.map((version, idx) => (
        <VersionItem key={version.id} index={idx + 1} version={version} onApply={onApply} />
      ))}
    </div>
  )
}

/**
 * 历史版本 Drawer 弹窗
 *
 * 从编辑器顶部栏"历史记录"按钮触发，从右侧弹出。
 */
const VersionDrawer: React.FC<VersionDrawerProps> = ({ open, onClose, chapterId, onApply }) => {
  const { loading, versions } = useVersionList(open, chapterId)
  return (
    <Drawer
      title="历史记录"
      placement="right"
      open={open}
      onClose={onClose}
      size={420}
      classNames={{ body: 'bg-gray-50!' }}
    >
      <VersionDrawerBody loading={loading} versions={versions} onApply={onApply} />
    </Drawer>
  )
}

export default VersionDrawer
