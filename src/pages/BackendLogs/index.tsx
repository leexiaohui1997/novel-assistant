import { Card, Splitter } from 'antd'
import { useState } from 'react'

import { LogFileContent } from './LogFileContent'
import { LogFileTree } from './LogFileTree'

export default function BackendLogs() {
  const [selectedFileName, setSelectedFileName] = useState<string>()

  return (
    <Card
      title="后端日志"
      classNames={{
        root: 'flex-1 flex flex-col gap-px',
        body: 'flex-1 h-0 overflow-hidden flex p-0!',
      }}
    >
      <Splitter>
        {/* 目录树 */}
        <Splitter.Panel defaultSize={200} min={200}>
          <LogFileTree value={selectedFileName} onChange={setSelectedFileName} />
        </Splitter.Panel>
        <Splitter.Panel>
          <LogFileContent filename={selectedFileName} />
        </Splitter.Panel>
      </Splitter>
    </Card>
  )
}
