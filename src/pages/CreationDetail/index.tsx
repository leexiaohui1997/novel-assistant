import { ArrowLeftOutlined } from '@ant-design/icons'
import { Button } from 'antd'
import { useNavigate, useParams } from 'react-router-dom'

import { logger } from '@/utils/logger'

const CreationDetail: React.FC = () => {
  const { novelId } = useParams()
  const navigate = useNavigate()

  logger.debug('CreationDetail novelId:', novelId)

  return (
    <div className="h-full flex">
      {/* Tab栏 */}
      <div className="w-10 flex flex-col justify-between">
        <div className="py-4! flex items-center justify-center">
          <Button icon={<ArrowLeftOutlined />} type="text" onClick={() => navigate(-1)}></Button>
        </div>
        <div className="pt-8! bg-white rounded-tr-3xl border border-gray-200 flex-1 h-0"></div>
      </div>
    </div>
  )
}

export default CreationDetail
