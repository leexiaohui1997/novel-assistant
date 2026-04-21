import { useParams } from 'react-router-dom'

import { logger } from '@/utils/logger'

const CreationDetail: React.FC = () => {
  const { novelId } = useParams()
  logger.debug('CreationDetail novelId:', novelId)

  return <div>创作详情页</div>
}

export default CreationDetail
