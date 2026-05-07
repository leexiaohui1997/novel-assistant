import { Drawer } from 'antd'
import React, { useCallback, useImperativeHandle, useState } from 'react'

import { AiCallLogBasicInfo } from './AiCallLogBasicInfo'
import { AICallLogMessages } from './AICallLogMessages'

import { AiCallLogItem } from '@/services/tokensDashboardService'

export type AiCallLogDrawerHandle = {
  open: (data: AiCallLogItem) => void
}

export type AiCallLogDrawerProps = {
  ref?: React.Ref<AiCallLogDrawerHandle>
}

export function AiCallLogDrawer({ ref }: AiCallLogDrawerProps) {
  const [logInfo, setLogInfo] = useState<AiCallLogItem | null>(null)
  const [drawerVisible, setDrawerVisible] = useState(false)

  const open = useCallback((data: AiCallLogItem) => {
    setLogInfo(data)
    setDrawerVisible(true)
  }, [])

  const close = useCallback(() => {
    setDrawerVisible(false)
  }, [])

  const afterOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setLogInfo(null)
    }
  }, [])

  useImperativeHandle(ref, () => ({
    open,
  }))

  return (
    <Drawer
      open={drawerVisible}
      size={600}
      title="AI调用日志"
      onClose={close}
      afterOpenChange={afterOpenChange}
      destroyOnHidden
    >
      {logInfo && (
        <div className="flex flex-col gap-4">
          <AiCallLogBasicInfo logInfo={logInfo} />
          <AICallLogMessages logInfo={logInfo} />
        </div>
      )}
    </Drawer>
  )
}
