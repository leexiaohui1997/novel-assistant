import { ArrowLeftOutlined } from '@ant-design/icons'
import { Button } from 'antd'
import { useNavigate, useParams } from 'react-router-dom'

import { useCreationState } from '@/hooks/useCreationState'
import { CreationStateProvider } from '@/providers/CreationStateProvider'

interface CreationDetailProps {
  novelId: string
}

const CreationDetailContent: React.FC = () => {
  const navigate = useNavigate()
  const context = useCreationState()

  console.log(context)

  return (
    <div className="h-full flex">
      {/* Tab栏 */}
      <div className="w-10 flex flex-col justify-between bg-neutral-50">
        <div className="py-4! flex items-center justify-center">
          <Button icon={<ArrowLeftOutlined />} type="text" onClick={() => navigate(-1)}></Button>
        </div>
        <div className="pt-8! bg-white rounded-tr-3xl border border-gray-200 flex-1 h-0"></div>
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
