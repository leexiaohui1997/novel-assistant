import { lazy } from 'react'
import { Navigate } from 'react-router-dom'

import type { RouteObject } from 'react-router-dom'

import Layout from '@/components/layout/Layout'

/**
 * 路由 handle 元信息接口
 * 通过 React Router 的 handle 属性传递页面级配置
 */
export interface RouteHandle {
  /** 页面标题 */
  title: string
  /** 是否需要认证 */
  requiresAuth?: boolean
  /** 是否隐藏侧边栏 */
  hideSidebar?: boolean
}

/**
 * 创建懒加载路由元素
 * 避免在路由配置文件中声明顶层组件变量触发 react-refresh lint 规则
 */
const lazyElement = (loader: () => Promise<{ default: React.FC }>) => {
  const C = lazy(loader)
  return <C />
}

/**
 * 路由配置
 * Layout 作为父路由，通过 Outlet 渲染子路由
 * handle 传递页面级 meta 信息，Layout 通过 useMatches 读取
 */
export const ROUTES_CONFIG: RouteObject[] = [
  {
    element: <Layout />,
    children: [
      {
        index: true,
        element: <Navigate to="/novels" replace />,
      },
      {
        path: '/novels',
        element: lazyElement(() => import('@/pages/Novels')),
        handle: {
          title: '作品管理',
        } satisfies RouteHandle,
      },
      {
        path: '/creation/:novelId',
        element: lazyElement(() => import('@/pages/CreationDetail')),
        handle: {
          title: '创作详情',
          hideSidebar: true,
        } satisfies RouteHandle,
      },
    ],
  },
]
