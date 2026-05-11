import { Select } from 'antd'
import React, { useCallback, useEffect, useImperativeHandle, useMemo, useState } from 'react'

import { getCharactersByNovel } from '@/services/characterService'
import { Character } from '@/types/character'
import { logger } from '@/utils/logger'

export interface CharacterSelectHandle {
  refresh: () => Promise<Character[]>
  addNewCharacter: (character: Character) => void
}

export interface CharacterSelectProps {
  /** 小说 ID，用于加载该小说下的角色列表 */
  novelId: string
  /** 当前选中的角色 ID 列表 */
  value?: string[]
  /** 选中值变化回调 */
  onChange?: (value: string[]) => void
  /** 占位文字 */
  placeholder?: string
  className?: string
  ref?: React.RefObject<CharacterSelectHandle>
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
  placeholder = '请选择角色',
  className,
  ref,
}) => {
  const [loading, setLoading] = useState(true)
  const [characters, setCharacters] = useState<{ label: string; value: string }[]>([])
  const [refreshPromise, setRefreshPromise] = useState<{
    resolve: (value: Character[]) => void
    reject: (reason?: unknown) => void
  }>()

  const refresh = useCallback((): Promise<Character[]> => {
    return new Promise((resolve, reject) => {
      setRefreshPromise({ resolve, reject })
    })
  }, [])

  const addNewCharacter = useCallback(
    async ({ id }: Character) => {
      const list = await refresh()
      if (list.some((c) => c.id === id)) {
        onChange?.([...(value || []), id])
      }
    },
    [refresh, value, onChange],
  )

  useImperativeHandle(ref, () => ({ refresh, addNewCharacter }))

  useEffect(() => {
    let cancelled = false

    const loadCharacters = async () => {
      try {
        setLoading(true)
        const list = await getCharactersByNovel(novelId)
        if (!cancelled) {
          setCharacters(list.map((c) => ({ label: c.name, value: c.id })))
          refreshPromise?.resolve(list)
        }
      } catch (e) {
        if (!cancelled) {
          logger.error('加载角色列表失败:', e)
          refreshPromise?.reject(e)
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    void loadCharacters()
    return () => {
      cancelled = true
    }
  }, [novelId, refreshPromise])

  const options = useMemo(() => characters, [characters])

  return (
    <Select
      mode="multiple"
      loading={loading}
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
