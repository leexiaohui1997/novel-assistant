import { Input } from 'antd'

import { BaseEditableCell, BaseEditableCellProps } from './Base'

export type InputEditableCellProps = {} & Omit<
  BaseEditableCellProps<string>,
  'readCell' | 'editCell'
>

export const InputEditableCell = (props: InputEditableCellProps) => {
  return (
    <BaseEditableCell
      {...props}
      editCell={(value, setValue, { size, isError, commitEdit }) => (
        <Input
          size={size}
          value={value}
          status={isError ? 'error' : ''}
          onChange={(e) => setValue(e.target.value)}
          onPressEnter={commitEdit}
          autoFocus
        />
      )}
    />
  )
}
