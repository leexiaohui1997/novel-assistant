import { Select, Tag, type SelectProps } from 'antd'
import { DefaultOptionType } from 'antd/es/select'
import React, { useCallback, useEffect, useMemo, useState } from 'react'

import { getAllModels, type Model } from '@/services/modelService'
import { logger } from '@/utils/logger'

/**
 * 模型选择器组件属性
 */
export interface ModelSelectProps extends Omit<SelectProps, 'options'> {
  /** 选中的模型 ID */
  value?: string
  /** 选中变化回调 */
  onChange?: (value: string) => void
  /** 是否只显示启用的模型 */
  onlyEnabled?: boolean
  /** 是否显示自动选择的模型 */
  withAuto?: boolean
}

/**
 * 模型选择器组件
 *
 * 提供从数据库加载已配置模型的列表，支持搜索和筛选。
 * 默认只显示启用的模型，可通过 onlyEnabled 属性控制。
 *
 * @example
 * ```tsx
 * <ModelSelect
 *   value={selectedModelId}
 *   onChange={setSelectedModelId}
 *   placeholder="请选择模型"
 * />
 * ```
 */
export const ModelSelect: React.FC<ModelSelectProps> = ({
  value,
  onChange,
  withAuto = false,
  onlyEnabled = true,
  ...selectProps
}) => {
  const [models, setModels] = useState<Model[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    let cancelled = false

    const loadModels = async () => {
      try {
        setLoading(true)
        // 直接调用后端 API，支持启用状态筛选
        const result = await getAllModels(onlyEnabled ? true : undefined)
        if (!cancelled) {
          setModels(result)
        }
      } catch (error) {
        if (!cancelled) {
          logger.error('加载模型列表失败:', error)
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    void loadModels()

    return () => {
      cancelled = true
    }
  }, [onlyEnabled])

  // 将模型数据转换为 Select 选项格式
  const options = useMemo(() => {
    const result = models.map((model) => ({
      label: `${model.alias}`,
      value: model.id,
    }))

    if (withAuto) {
      result.unshift({
        label: '自动选择',
        value: '',
      })
    }

    return result
  }, [models, withAuto])

  const optionRender: SelectProps['optionRender'] = useCallback(
    (oriOption: DefaultOptionType) => {
      const model = models.find((model) => model.id === oriOption.value)

      if (!model) return <div className="text-xs">{oriOption.label}</div>

      return (
        <div className="flex items-center gap-1">
          <div className="text-xs">{model.alias}</div>
          {model.providerName && <Tag color="orange">{model.providerName}</Tag>}
        </div>
      )
    },
    [models],
  )

  return (
    <Select
      placeholder="请选择模型"
      allowClear
      showSearch={{
        filterOption: (input, option) => {
          const searchText = input.toLowerCase()
          const label = (option?.label as string)?.toLowerCase() || ''
          const value = (option?.value as string)?.toLowerCase() || ''
          return label.includes(searchText) || value.includes(searchText)
        },
      }}
      optionRender={optionRender}
      options={options}
      loading={loading}
      value={value}
      onChange={onChange}
      {...selectProps}
    />
  )
}
