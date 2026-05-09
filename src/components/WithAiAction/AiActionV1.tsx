import { useCallback } from 'react'

import { WithAiActionBase, WithAiActionBaseExecute, WithAiActionBaseProps } from './Base'

import { useAiAction, UseAiActionProps } from '@/hooks/useAiAction'

export type WithAiActionProps<T = unknown> = Omit<WithAiActionBaseProps<T>, 'execute'> & {
  aiAction: UseAiActionProps
}

export function WithAiAction<T = unknown>({ aiAction, ...props }: WithAiActionProps<T>) {
  const { execute: executeV1 } = useAiAction<T>(aiAction)
  const execute = useCallback<WithAiActionBaseExecute<T>>(
    (forms) =>
      executeV1({
        modelId: forms.modelId,
        userFeedback: forms.userFeedback,
      }),
    [executeV1],
  )

  return <WithAiActionBase {...props} execute={execute} />
}
