import { Select } from 'antd'
import { DefaultOptionType } from 'antd/es/select'
import { useCallback, useEffect, useImperativeHandle, useMemo, useState } from 'react'

import TagSelectorModal from './TagSelectorModal'

import { getTagsByAudience, type Tag, type TargetAudience } from '@/services/tagService'
import { logger } from '@/utils/logger'

export interface TagSelectorHandle {
  handleSelectChange: (ids: number[]) => void
}

interface TagSelectorProps {
  value?: number[]
  onChange?: (value: number[]) => void
  targetAudience?: Exclude<TargetAudience, 'both'>
  placeholder?: string
  ref?: React.Ref<TagSelectorHandle>
}

/**
 * 标签选择器组件
 */
const TagSelector: React.FC<TagSelectorProps> = ({
  value,
  onChange,
  targetAudience,
  placeholder = '请选择作品标签',
  ref,
}) => {
  const [open, setOpen] = useState(false)
  const [tags, setTags] = useState<Tag[]>([])
  const [loading, setLoading] = useState(false)
  const [innerValue, setInnerValue] = useState<number[]>(value || [])

  // 外部 value 变化时同步到内部
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setInnerValue(value || [])
  }, [value])

  /**
   * Select 变更（清除 / 手动删标签）
   */
  const handleSelectChange = useCallback(
    (ids: number[]) => {
      console.log('TagSelector handleSelectChange:', ids)
      setInnerValue(ids)
      onChange?.(ids)
    },
    [onChange],
  )

  const tagOptions = useMemo<DefaultOptionType[]>(
    () => tags.map((tag) => ({ label: tag.name, value: tag.id })),
    [tags],
  )

  useImperativeHandle(ref, () => ({
    handleSelectChange,
  }))

  // 加载标签数据
  useEffect(() => {
    if (!targetAudience) return

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
  }, [targetAudience])

  /**
   * 目标受众（男频/女频）切换时，自动剔除不兼容的已选标签。
   *
   * 场景：用户先选了「女频」下的若干标签，随后把作品受众切到「男频」，
   *      原来的女频标签就不再合法，需要从选中值里过滤掉。
   *
   * 规则：只保留 targetAudience 为当前受众或 'both'（通用）的标签。
   * 性能：仅在过滤前后长度不一致时才触发更新，避免无意义的 re-render。
   */
  useEffect(() => {
    // tags 未加载完 / targetAudience 缺失时跳过，避免误清空
    if (!tags.length) return
    if (!targetAudience) return

    const allowIds = innerValue.filter((id) =>
      ['both', targetAudience].includes(tags.find((t) => t.id === id)?.targetAudience || ''),
    )

    if (allowIds.length !== innerValue.length) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      handleSelectChange(allowIds)
    }
  }, [innerValue, targetAudience, tags, handleSelectChange])

  /**
   * 确认选择
   */
  const handleConfirm = (ids: number[]) => {
    setInnerValue(ids)
    onChange?.(ids)
    setOpen(false)
  }

  return (
    <>
      <Select
        open={false}
        mode="multiple"
        value={innerValue}
        options={tagOptions}
        disabled={!targetAudience}
        placeholder={placeholder}
        onClick={() => targetAudience && setOpen(true)}
        onChange={handleSelectChange}
        allowClear={true}
      ></Select>

      <TagSelectorModal
        open={open}
        value={innerValue}
        tags={tags}
        loading={loading}
        onOk={handleConfirm}
        onCancel={() => setOpen(false)}
      />
    </>
  )
}

export default TagSelector
