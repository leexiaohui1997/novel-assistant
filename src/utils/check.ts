/**
 * 校验字符串是否为合法的 JSON 格式
 *
 * @param value - 待校验的值
 * @returns 如果值是合法的 JSON 字符串则返回 true，否则返回 false
 */
export function isJson(value: unknown) {
  try {
    JSON.parse(value as string)
    return true
  } catch {
    return false
  }
}
