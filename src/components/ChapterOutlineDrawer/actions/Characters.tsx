import { FormInstance } from 'antd'

import { CharacterSuggestHandle } from '@/components/CharacterSuggest'
import { WithAiAction } from '@/components/WithAiAction'
import { useEditorForm } from '@/providers/EditorFormContext'

type CharactersResult = {
  existing_character_ids?: string[]
  new_character_descriptions?: string[]
}

export type CharactersActionProps = {
  label?: string
  novelId: string
  chapterId?: string
  formRef?: React.RefObject<FormInstance | null>
  suggestRef?: React.RefObject<CharacterSuggestHandle | null>
}

export function CharactersAction({
  label = '出场角色',
  novelId,
  chapterId,
  formRef,
  suggestRef,
}: CharactersActionProps) {
  const { title, content } = useEditorForm()

  const handleResult = (result: unknown) => {
    const data = result as CharactersResult
    if (!data) return

    mergeExistingCharacterIds(data)
    showNewCharacterDescriptions(data)
  }

  const mergeExistingCharacterIds = (data: CharactersResult) => {
    if (!data.existing_character_ids?.length) return
    if (formRef?.current) {
      const current = (formRef.current.getFieldValue('characterIds') as string[]) || []
      const merged = Array.from(new Set([...current, ...data.existing_character_ids]))
      formRef.current.setFieldsValue({ characterIds: merged })
    }
  }

  const showNewCharacterDescriptions = (data: CharactersResult) => {
    suggestRef?.current?.setSuggests(data.new_character_descriptions || [])
  }

  return (
    <WithAiAction
      tip={`AI 识别${label}`}
      triggerSize="small"
      placement="rightTop"
      showFeedback
      classNames={{ root: 'w-full items-center!', left: '' }}
      aiAction={{
        actionName: 'edit_chapter_characters',
        getParams: () => ({
          novel_id: novelId,
          chapter_id: chapterId ?? undefined,
          title: title || undefined,
          content: content || undefined,
        }),
      }}
      onResult={handleResult}
    >
      <span>{label}</span>
    </WithAiAction>
  )
}
