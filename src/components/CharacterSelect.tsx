import { Select } from 'antd'
import { useEffect, useMemo, useState } from 'react'

import { getCharactersByNovel } from '@/services/characterService'
import { logger } from '@/utils/logger'

export interface CharacterSelectProps {
  /** 小说 ID，用于加载该小说下的角色列表 */
  novelId: string
  /** 当前选中的角色 ID 列表 */
  value?: string[]
  /** 选中值变化回调 */
  onChange?: (value: string[]) => void
  /** 是否多选模式，默认 true */
  multiple?: boolean
  /** 占位文字 */
  placeholder?: string
  className?: string
}

/**
 * 角色选择器组件
 *
 * 根据 novelId 自动加载角色列表，支持单选/多选模式。
 * 遵循 Ant Design Form.Item 受控组件规范（value + onChange）。
 *
 * @example
 * ```tsx
 * <Form.Item label="出场角色" name="characterIds">
 *   <CharacterSelect novelId={novelId} />
 * </Form.Item>
 * ```
 */
export const CharacterSelect: React.FC<CharacterSelectProps> = ({
  novelId,
  value,
  onChange,
  multiple = true,
  placeholder = '请选择角色',
  className,
}) => {
  const [characters, setCharacters] = useState<{ label: string; value: string }[]>([])

  useEffect(() => {
    let cancelled = false

    const loadCharacters = async () => {
      try {
        const list = await getCharactersByNovel(novelId)
        if (!cancelled) {
          setCharacters(list.map((c) => ({ label: c.name, value: c.id })))
        }
      } catch (e) {
        if (!cancelled) {
          logger.error('加载角色列表失败:', e)
        }
      }
    }

    void loadCharacters()
    return () => {
      cancelled = true
    }
  }, [novelId])

  const options = useMemo(() => characters, [characters])

  return (
    <Select
      mode={multiple ? 'multiple' : undefined}
      value={value}
      onChange={onChange}
      className={className}
      options={options}
      placeholder={placeholder}
      allowClear
      showSearch={{ optionFilterProp: 'label' }}
    />
  )
}
