/**
 * 从未知类型的错误中提取可读的错误消息字符串
 *
 * 优先返回 Error 实例的 message，其他类型则转为字符串
 *
 * @param error - 未知类型的错误对象
 * @returns 错误消息字符串
 *
 * @example
 * // Error 实例
 * getErrorMsg(new Error('网络超时')) // '网络超时'
 *
 * @example
 * // 非 Error 类型
 * getErrorMsg('字符串错误') // '字符串错误'
 * getErrorMsg(404) // '404'
 */
export function getErrorMsg(error: unknown, fallback = '') {
  if (error instanceof Error) {
    return error.message
  }
  return String(error) || fallback
}
