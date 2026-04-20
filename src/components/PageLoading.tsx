import { Spin } from 'antd'

const PageLoading: React.FC = () => {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: '400px',
      }}
    >
      <Spin size="large" description="加载中..." />
    </div>
  )
}

export default PageLoading
