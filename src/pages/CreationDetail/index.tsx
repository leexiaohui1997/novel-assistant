import { ArrowLeftOutlined } from '@ant-design/icons'
import { Button } from 'antd'
import { useMemo } from 'react'
import { Outlet, useNavigate, useParams } from 'react-router-dom'

import { CREATION_TABS } from './config'

import { useCreationState } from '@/hooks/useCreationState'
import { CreationStateProvider } from '@/providers/CreationStateProvider'

interface CreationDetailProps {
  novelId: string
}

const CreationDetailContent: React.FC = () => {
  const navigate = useNavigate()
  const context = useCreationState()

  const renderedTabs = useMemo(
    () =>
      CREATION_TABS.map((tabConfig) => {
        const isActive = tabConfig.checkActive?.(context) ?? false
        const isDisabled = tabConfig.checkDisabled?.(context) ?? false
        const onClick = () => {
          if (!isDisabled && !isActive) {
            tabConfig.clickHandler?.(navigate, context)
          }
        }

        return (
          <div
            className={`flex flex-col items-center justify-center py-2! gap-1 transition-colors cursor-pointer ${
              isActive ? 'bg-brand text-white' : isDisabled ? '' : 'hover:bg-brand/20'
            } ${isDisabled ? 'cursor-not-allowed opacity-70' : ''}`}
            key={tabConfig.id}
            onClick={onClick}
          >
            <div>{tabConfig.icon}</div>
            <div className="text-xs">{tabConfig.title}</div>
          </div>
        )
      }),
    [context, navigate],
  )

  return (
    <div className="ml-10!">
      {/* Tab栏 */}
      <div className="w-10 h-[calc(100vh-56px)] flex flex-col justify-between bg-neutral-50 fixed left-0">
        <div className="py-4! flex items-center justify-center">
          <Button icon={<ArrowLeftOutlined />} type="text" onClick={() => navigate(-1)}></Button>
        </div>
        <div className="pt-8! bg-white rounded-tr-3xl border border-gray-200 flex-1 h-0">
          <div className="flex flex-col gap-2">{renderedTabs}</div>
        </div>
      </div>

      {/* 内容区域 */}
      <div>
        <Outlet />
      </div>
    </div>
  )
}

const CreationDetail: React.FC<CreationDetailProps> = ({ novelId }) => {
  return (
    <CreationStateProvider novelId={novelId}>
      <CreationDetailContent />
    </CreationStateProvider>
  )
}

const CreationDetailPage: React.FC = () => {
  const { novelId } = useParams()
  return novelId ? <CreationDetail novelId={novelId} key={novelId} /> : null
}

export default CreationDetailPage
