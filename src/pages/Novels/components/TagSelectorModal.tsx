import { Empty, Modal, Tag as AntdTag, Tooltip } from 'antd'
import { useState, Fragment } from 'react'

import { TAG_TYPE_LABELS, type Tag, type TagType } from '@/services/tagService'

interface TagSelectorModalProps {
  open: boolean
  value?: number[]
  tags: Tag[]
  loading?: boolean
  onOk: (ids: number[]) => void
  onCancel: () => void
}

const TAG_TYPE_ORDER: TagType[] = ['main_category', 'theme', 'character', 'plot']

/**
 * 统计指定类型已选数量
 */
const countByType = (selectedIds: number[], tags: Tag[], type: TagType): number => {
  return selectedIds.filter((id) => tags.find((t) => t.id === id)?.tagType === type).length
}

/**
 * 切换主分类（单选）：已选则清空该类型，未选则替换已有的主分类
 */
const toggleMainCategory = (prev: number[], tagId: number, tags: Tag[]): number[] => {
  const withoutMain = prev.filter(
    (id) => tags.find((t) => t.id === id)?.tagType !== 'main_category',
  )
  return prev.includes(tagId) ? withoutMain : [...withoutMain, tagId]
}

/**
 * 切换多选标签（同类型最多 2 个）
 */
const toggleMultiTag = (prev: number[], tagId: number, tagType: TagType, tags: Tag[]): number[] => {
  if (prev.includes(tagId)) {
    return prev.filter((id) => id !== tagId)
  }
  if (countByType(prev, tags, tagType) < 2) {
    return [...prev, tagId]
  }
  return prev
}

/**
 * 标签选择弹窗
 */
const TagSelectorModal: React.FC<TagSelectorModalProps> = ({
  open,
  value = [],
  tags,
  loading = false,
  onOk,
  onCancel,
}) => {
  const [activeTab, setActiveTab] = useState<TagType>('main_category')
  const [selectedIds, setSelectedIds] = useState<number[]>([...value])

  /**
   * 处理弹窗关闭
   */
  const afterClose = () => {
    setSelectedIds([...value])
    setActiveTab('main_category')
  }

  /**
   * 处理标签选择
   */
  const handleTagSelect = (tagId: number, tagType: TagType) => {
    setSelectedIds((prev) =>
      tagType === 'main_category'
        ? toggleMainCategory(prev, tagId, tags)
        : toggleMultiTag(prev, tagId, tagType, tags),
    )
  }

  const currentTabTags = tags.filter((tag) => tag.tagType === activeTab)

  return (
    <Modal
      title="作品标签"
      open={open}
      onCancel={onCancel}
      onOk={() => onOk(selectedIds)}
      width={800}
      okText="确认"
      cancelText="取消"
      footer={(originNode) => (
        <div className="flex justify-between items-center">
          <div className="text-xs text-gray-400">
            主分类必选且只能选一个，主题、角色、情节最多可选两个
          </div>
          <div className="flex items-center gap-2">{originNode}</div>
        </div>
      )}
      afterClose={afterClose}
    >
      <div className="flex flex-1 h-0 p-0!">
        {/* 左侧分类导航 */}
        <div className="w-25 py-4! flex flex-col gap-2">
          {TAG_TYPE_ORDER.map((type) => (
            <div
              key={type}
              onClick={() => setActiveTab(type)}
              className={`relative py-2! px-3! ps-6! cursor-pointer text-sm transition-all border-l-4 ${
                activeTab === type
                  ? 'text-brand bg-brand/10 font-semibold border-brand'
                  : 'text-gray-400 font-normal border-transparent hover:text-brand'
              }`}
            >
              {type === 'main_category' && (
                <span className="leading-none absolute top-1/2 left-4 -translate-y-1/2">*</span>
              )}
              <Fragment>{TAG_TYPE_LABELS[type]}</Fragment>
            </div>
          ))}
        </div>

        {/* 右侧标签列表 */}
        <div className="flex-1 flex flex-col w-0 pt-4!">
          {selectedIds.length > 0 && (
            <div className="pb-4! px-4! flex flex-wrap gap-2">
              {selectedIds.map((tagId) => {
                const tag = tags.find((t) => t.id === tagId)
                return tag ? (
                  <AntdTag
                    onClose={() => handleTagSelect(tag.id, tag.tagType)}
                    closable
                    key={tag.id}
                  >
                    {tag.name}
                  </AntdTag>
                ) : null
              })}
            </div>
          )}
          <div className="flex-1 h-0 overflow-auto">
            <div className="p-4! pt-0!">
              {loading ? (
                <div className="flex items-center justify-center h-48">加载中...</div>
              ) : currentTabTags.length === 0 ? (
                <Empty description="暂无标签" />
              ) : (
                <div className="grid grid-cols-2 gap-4">
                  {currentTabTags.map((tag) => {
                    const isSelected = selectedIds.includes(tag.id)
                    const isMainCategory = tag.tagType === 'main_category'
                    const sameTypeCount = countByType(selectedIds, tags, tag.tagType)
                    const isMaxReached = !isMainCategory && sameTypeCount >= 2 && !isSelected

                    return (
                      <Tooltip
                        key={tag.id}
                        title={tag.description}
                        styles={{ root: { maxWidth: 400 } }}
                        placement="bottom"
                        arrow={false}
                        mouseEnterDelay={1}
                      >
                        <div
                          className={`border border-gray-200 rounded-md py-3! px-4! cursor-pointer flex flex-col gap-1 justify-center h-18 ${
                            isSelected
                              ? 'bg-brand/10 border-brand!'
                              : isMaxReached
                                ? 'cursor-not-allowed! opacity-70'
                                : 'hover:bg-gray-100'
                          }`}
                          onClick={() => {
                            if (!isMaxReached) {
                              handleTagSelect(tag.id, tag.tagType)
                            }
                          }}
                        >
                          <div className="font-bold text-cut">{tag.name}</div>
                          {tag.description && (
                            <div className="text-gray-400 h-4 leading-4 text-cut text-xs">
                              {tag.description}
                            </div>
                          )}
                        </div>
                      </Tooltip>
                    )
                  })}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </Modal>
  )
}

export default TagSelectorModal
