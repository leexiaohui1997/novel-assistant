import { ColumnFilterItem } from 'antd/es/table/interface'
import { useCallback, useMemo } from 'react'

import { ModelInfo } from '@/services/modelService'

export const useModelIdFilters = (models: ModelInfo[]) => {
  const modelIdFilters = useMemo(() => {
    const data: ColumnFilterItem[] = []
    const ensureId = (id: string): ColumnFilterItem | null => {
      const chunks = id.split('/').filter(Boolean)
      if (!chunks.length) return null
      const parent = chunks.length === 1 ? data : ensureId(chunks.slice(0, -1).join('/'))?.children
      if (parent) {
        let target = parent.find((item) => item.value === id)
        if (!target) {
          target = { text: chunks[chunks.length - 1], value: id, children: [] }
          parent.push(target)
        }
        return target
      }
      return null
    }
    models.forEach((model) => ensureId(model.modelId))
    return data
  }, [models])

  const modelIdFilterSearch = useCallback((input: string, record: ColumnFilterItem) => {
    return (record.value as string).startsWith(input) || input.startsWith(record.value as string)
  }, [])

  return {
    modelIdFilters,
    modelIdFilterSearch,
  }
}
