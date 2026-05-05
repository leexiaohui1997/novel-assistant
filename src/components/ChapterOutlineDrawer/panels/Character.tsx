import { useCallback, useImperativeHandle } from 'react'

import { ChapterOutlinePanelHandle } from './common'

export type CharacterProps = {
  ref?: React.RefObject<ChapterOutlinePanelHandle>
}

export function Character({ ref }: CharacterProps) {
  const save = useCallback(async () => {}, [])

  useImperativeHandle(ref, () => ({ save }))

  return <div>Character</div>
}
