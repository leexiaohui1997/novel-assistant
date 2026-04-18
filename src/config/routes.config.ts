import { lazy } from 'react'

/**
 * 路由配置接口
 */
export interface RouteConfig {
  /** 路由路径 */
  path: string
  /** 懒加载的组件 */
  component: React.LazyExoticComponent<React.FC>
  /** 路由元信息 */
  meta?: {
    /** 页面标题 */
    title: string
    /** 是否需要认证 */
    requiresAuth?: boolean
  }
  /** 子路由 */
  children?: RouteConfig[]
}

/**
 * 路由配置列表
 * 使用 React.lazy 实现组件懒加载
 */
export const ROUTES_CONFIG: RouteConfig[] = [
  {
    path: '/novels',
    component: lazy(() => import('@/pages/Novels')),
    meta: {
      title: '作品管理',
    },
  },
]
