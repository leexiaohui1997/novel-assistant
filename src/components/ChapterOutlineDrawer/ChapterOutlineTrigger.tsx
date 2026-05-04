import { ReadOutlined } from '@ant-design/icons'
import { Button, Tooltip } from 'antd'
import { useRef } from 'react'

import {
  ChapterOutlineDrawer,
  ChapterOutlineDrawerHandle,
  ChapterOutlineDrawerProps,
} from './ChapterOutlineDrawer'

export type ChapterOutlineTriggerProps = Omit<ChapterOutlineDrawerProps, 'ref'>

export function ChapterOutlineTrigger(props: ChapterOutlineTriggerProps) {
  const drawerRef = useRef<ChapterOutlineDrawerHandle>(null)

  return (
    <>
      <Tooltip title="本章大纲" placement="bottom" mouseEnterDelay={1} color="white">
        <Button type="text" icon={<ReadOutlined />} onClick={() => drawerRef.current?.open()} />
      </Tooltip>
      <ChapterOutlineDrawer ref={drawerRef} {...props} />
    </>
  )
}
