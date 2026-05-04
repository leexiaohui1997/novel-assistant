import { FileOutlined, IdcardOutlined, ProjectOutlined } from '@ant-design/icons'
import React from 'react'
import { NavigateFunction } from 'react-router-dom'

import { CreationStateContextType } from '@/providers/CreationStateContext'

/**
 * 创作详情页 Tab 配置项
 */
export type CreationTab = {
  /** Tab 唯一标识 */
  id: string
  /** Tab 标题 */
  title: string
  /** Tab 图标 */
  icon: React.ReactNode
  /** 判断该 Tab 是否处于激活状态 */
  checkActive?: (ctx: CreationStateContextType) => boolean
  /** 判断该 Tab 是否被禁用 */
  checkDisabled?: (ctx: CreationStateContextType) => boolean
  /** Tab 点击回调，可执行导航等操作 */
  clickHandler?: (navigate: NavigateFunction, ctx: CreationStateContextType) => unknown
}

/**
 * 创作详情页 Tab 列表配置
 */
export const CREATION_TABS: CreationTab[] = [
  {
    id: 'basic',
    title: '概览',
    icon: <ProjectOutlined />,
    checkActive: (ctx) => ctx.currentMatch.id === 'creation-basic',
    clickHandler: (navigate) => navigate('./basic', { replace: true }),
  },
  {
    id: 'character',
    title: '角色',
    icon: <IdcardOutlined />,
    checkActive: (ctx) => ctx.currentMatch.id === 'creation-character',
    clickHandler: (navigate) => navigate('./character', { replace: true }),
  },
  {
    id: 'article',
    title: '文章',
    icon: <FileOutlined />,
    checkActive: (ctx) => ctx.currentMatch.id === 'creation-article',
    clickHandler: (navigate) => navigate('./article', { replace: true }),
  },
]
