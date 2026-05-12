import { Spin } from 'antd'

export type TermChapterDescProps = {
  termId: string
}

export default function TermChapterDesc(_: TermChapterDescProps) {
  return <Spin size="small" />
}
