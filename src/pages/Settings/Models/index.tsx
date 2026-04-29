import { PlusOutlined } from '@ant-design/icons'
import { Button, Card } from 'antd'
import React, { useRef } from 'react'

import AddModelModal, { AddModelModalHandle } from './AddModelModal'

const ModelManage: React.FC = () => {
  const addModalRef = useRef<AddModelModalHandle>(null)

  const renderedExtra = (
    <Button type="primary" icon={<PlusOutlined />} onClick={() => addModalRef.current?.show()}>
      添加模型
    </Button>
  )

  return (
    <>
      <Card title="模型管理" extra={renderedExtra}></Card>
      <AddModelModal ref={addModalRef} />
    </>
  )
}

export default ModelManage
