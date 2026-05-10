import { Spin, Tree, TreeDataNode } from 'antd'
import { useMemo } from 'react'

import { useLogFiles } from './hooks/useLogFiles'

export type LogFileTreeProps = {
  value?: string
  onChange?: (value: string) => void
}

export function LogFileTree({ value, onChange }: LogFileTreeProps) {
  const { logFiles, loading } = useLogFiles()

  const treeData = useMemo(
    () =>
      logFiles.map<TreeDataNode>((file) => ({
        key: file.filename,
        title: file.filename.split('.')[1],
        isLeaf: true,
      })),
    [logFiles],
  )

  return (
    <Spin className="min-h-full" spinning={loading}>
      <div className="p-2">
        <Tree.DirectoryTree
          treeData={treeData}
          selectedKeys={value ? [value] : []}
          expandAction={false}
          onSelect={(keys) => onChange?.(keys[0] as string)}
          virtual={treeData.length > 50}
        />
      </div>
    </Spin>
  )
}
