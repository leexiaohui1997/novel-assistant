import { useMemo } from 'react'
import { useMatches } from 'react-router-dom'

import { RouteHandle } from '@/config'

/**
 * 路由元信息 Hook
 *
 * 合并当前匹配的所有路由层级中的 handle 信息，
 * 返回聚合后的 RouteHandle 对象，方便子路由覆盖或扩展父路由的元信息。
 */
export const useRouteMeta = () => {
  /** 获取当前匹配的所有路由（从父到子依次排列） */
  const matches = useMatches()

  /** 将各层路由的 handle 逐层合并，后匹配的子路由会覆盖同名字段 */
  const handle = useMemo(
    () =>
      matches.reduce((acc, match) => {
        return { ...acc, ...(match.handle || {}) }
      }, {} as RouteHandle),
    [matches],
  )

  return handle
}
