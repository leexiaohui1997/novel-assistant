import { Select } from 'antd'
import { useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react'

import { Provider, getProvidersWithPagination } from '@/services/providerService'
import { logger } from '@/utils/logger'

export interface ProviderSelectHandle {
  getInfo: (id: string) => Provider | undefined
}

export interface ProviderSelectProps {
  ref?: React.RefObject<ProviderSelectHandle>
  value?: string
  onChange?: (value: string) => void
  placeholder?: string
  className?: string
}

export const ProviderSelect: React.FC<ProviderSelectProps> = ({
  ref,
  value,
  onChange,
  className,
  placeholder = '请选择供应商',
}) => {
  const [providers, setProviders] = useState<Provider[]>([])
  const providersRef = useRef(providers)

  /** 加载已启用的供应商列表 */
  useEffect(() => {
    let cancelled = false

    const loadData = async () => {
      try {
        const result = await getProvidersWithPagination(1, 100)
        if (!cancelled) {
          const enabled = result.data.filter((p) => p.isEnabled)
          setProviders(enabled)
        }
      } catch (error) {
        if (!cancelled) {
          logger.error('加载供应商列表失败:', error)
        }
      }
    }

    void loadData()
    return () => {
      cancelled = true
    }
  }, [])

  /** 保持 ref 与 state 同步 */
  useEffect(() => {
    providersRef.current = providers
  }, [providers])

  /** 暴露 getInfo 方法 */
  useImperativeHandle(ref, () => ({
    getInfo: (id: string) => providersRef.current.find((p) => p.id === id),
  }))

  const options = useMemo(() => providers.map((p) => ({ label: p.name, value: p.id })), [providers])

  return (
    <Select
      value={value}
      className={className}
      onChange={onChange}
      options={options}
      placeholder={placeholder}
      allowClear
      showSearch={{
        optionFilterProp: 'label',
      }}
    />
  )
}
