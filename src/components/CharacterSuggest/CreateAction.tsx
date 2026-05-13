import { PlusOutlined } from '@ant-design/icons'
import { Button, Tooltip } from 'antd'
import { useCallback } from 'react'

import { useAiAction } from '@/hooks/useAiAction'
import { GeneratedCharacter } from '@/pages/CreationDetail/pages/character/components/CharacterModal'
import { useEditorForm } from '@/providers/EditorFormContext'

export type CreateActionProps = {
  novelId: string
  suggest: string
  modelId: string
  onResult?: (result: GeneratedCharacter) => void
}

export default function CreateAction({ novelId, modelId, suggest, onResult }: CreateActionProps) {
  const { content } = useEditorForm()
  const getParams = useCallback(
    () => ({
      novel_id: novelId,
      reference_content: content,
    }),
    [novelId, content],
  )

  const { execute, loading, result } = useAiAction<GeneratedCharacter>({
    actionName: 'generate_character',
    getParams,
  })

  const handleClick = useCallback(() => {
    if (result) {
      onResult?.(result)
    }

    execute({
      modelId,
      userFeedback: suggest,
      onSuccess: onResult,
    })
  }, [execute, modelId, suggest, onResult, result])

  return (
    <Tooltip title="创建角色">
      <Button
        size="small"
        variant="text"
        color="primary"
        loading={loading}
        icon={<PlusOutlined />}
        onClick={handleClick}
      ></Button>
    </Tooltip>
  )
}
