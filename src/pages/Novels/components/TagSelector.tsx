import { Empty, Modal, Select, Tag as AntdTag, Tooltip } from 'antd'
import { DefaultOptionType } from 'antd/es/select'
import { useEffect, useMemo, useState, Fragment } from 'react'

import {
  TAG_TYPE_LABELS,
  getTagsByAudience,
  type Tag,
  type TagType,
  type TargetAudience,
} from '@/services/tagService'
import { logger } from '@/utils/logger'

interface TagSelectorProps {
  value?: number[]
  onChange?: (value: number[]) => void
  targetAudience?: Exclude<TargetAudience, 'both'>
  placeholder?: string
}

/**
 * 标签选择器组件
 */
const TagSelector: React.FC<TagSelectorProps> = ({
  value,
  onChange,
  targetAudience,
  placeholder = '请选择作品标签',
}) => {
  const [open, setOpen] = useState(false)
  const [tags, setTags] = useState<Tag[]>([])
  const [loading, setLoading] = useState(false)
  const [activeTab, setActiveTab] = useState<TagType>('main_category')
  const [selectedIds, setSelectedIds] = useState<number[]>(value || [])

  const tagOptions = useMemo<DefaultOptionType[]>(
    () => tags.map((tag) => ({ label: tag.name, value: tag.id })),
    [tags],
  )

  // 加载标签数据
  useEffect(() => {
    if (!targetAudience || !open) return

    const loadTags = async () => {
      setLoading(true)
      try {
        const data = await getTagsByAudience(targetAudience)
        setTags(data)
        logger.debug(`加载${targetAudience === 'male' ? '男频' : '女频'}标签成功:`, data.length)
      } catch (error) {
        logger.error('加载标签失败:', error)
        setTags([])
      } finally {
        setLoading(false)
      }
    }

    void loadTags()
  }, [targetAudience, open])

  // 同步外部 value 变化
  useEffect(() => {
    if (value) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSelectedIds(value)
    }
  }, [value])

  /**
   * 处理标签选择
   */
  const handleTagSelect = (tagId: number, tagType: TagType) => {
    setSelectedIds((prev) => {
      const isSelected = prev.includes(tagId)

      // 主分类：只能选一个
      if (tagType === 'main_category') {
        if (isSelected) {
          return prev.filter((id) => {
            const tag = tags.find((t) => t.id === id)
            return tag?.tag_type !== 'main_category'
          })
        } else {
          // 移除其他主分类，添加当前主分类
          const withoutMainCategory = prev.filter((id) => {
            const tag = tags.find((t) => t.id === id)
            return tag?.tag_type !== 'main_category'
          })
          return [...withoutMainCategory, tagId]
        }
      }

      // 其他类型：最多选两个
      const sameTypeCount = prev.filter((id) => {
        const tag = tags.find((t) => t.id === id)
        return tag?.tag_type === tagType
      }).length

      if (isSelected) {
        return prev.filter((id) => id !== tagId)
      } else if (sameTypeCount < 2) {
        return [...prev, tagId]
      }

      return prev
    })
  }

  /**
   * 确认选择
   */
  const handleConfirm = () => {
    onChange?.(selectedIds)
    setOpen(false)
  }

  /**
   * 取消选择
   */
  const handleCancel = () => {
    setSelectedIds(value || [])
    setOpen(false)
  }

  /**
   * 获取当前 Tab 的标签列表
   */
  const getCurrentTabTags = () => {
    return tags.filter((tag) => tag.tag_type === activeTab)
  }

  return (
    <>
      <Select
        open={false}
        mode="multiple"
        value={selectedIds}
        options={tagOptions}
        disabled={!targetAudience}
        placeholder={placeholder}
        onClick={() => targetAudience && setOpen(true)}
        onChange={setSelectedIds}
        allowClear={true}
      ></Select>

      <Modal
        title="作品标签"
        open={open}
        onCancel={handleCancel}
        onOk={handleConfirm}
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
      >
        <div className="flex flex-1 h-0 p-0!">
          {/* 左侧分类导航 */}
          <div className="w-25 py-4! flex flex-col gap-2">
            {(['main_category', 'theme', 'character', 'plot'] as TagType[]).map((type) => (
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
                  const tag = tags.find((tag) => tag.id === tagId)
                  return tag ? (
                    <AntdTag
                      onClose={() => handleTagSelect(tag.id, tag.tag_type)}
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
                ) : getCurrentTabTags().length === 0 ? (
                  <Empty description="暂无标签" />
                ) : (
                  <div className="grid grid-cols-2 gap-4">
                    {getCurrentTabTags().map((tag) => {
                      const isSelected = selectedIds.includes(tag.id)
                      const isMainCategory = tag.tag_type === 'main_category'
                      const sameTypeCount = selectedIds.filter((id) => {
                        const t = tags.find((t) => t.id === id)
                        return t?.tag_type === tag.tag_type
                      }).length
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
                                handleTagSelect(tag.id, tag.tag_type)
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
    </>
  )
}

export default TagSelector
