import { FormInstance } from 'antd'

import { WithAiAction } from '@/components/WithAiAction'
import { useEditorForm } from '@/providers/EditorFormContext'

export type PositioningActionProps = {
  label?: string
  novelId: string
  chapterId?: string
  formRef?: React.RefObject<FormInstance | null>
}

export function PositioningAction({
  label = '本章定位',
  novelId,
  chapterId,
  formRef,
}: PositioningActionProps) {
  const { title, content } = useEditorForm()

  return (
    <WithAiAction
      tip={`AI 编辑${label}`}
      triggerSize="small"
      placement="rightTop"
      showFeedback
      classNames={{ root: 'w-full items-center!', left: '' }}
      aiAction={{
        actionName: 'edit_chapter_positioning',
        getParams: () => ({
          novel_id: novelId,
          chapter_id: chapterId ?? undefined,
          title: title || undefined,
          content: content || undefined,
          positioning: formRef?.current?.getFieldValue('positioning') || undefined,
        }),
      }}
      onResult={(result) => {
        const positioning = (result as { positioning?: string })?.positioning
        if (positioning) {
          formRef?.current?.setFieldsValue({ positioning })
        }
      }}
    >
      <span>{label}</span>
    </WithAiAction>
  )
}
