/**
 * 数字工具函数
 *
 * 提供数字相关的转换/处理能力。
 */

const CN_DIGITS = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九']

/**
 * 将 1~99 的两位数转为中文数字
 *
 * @param num - 1~99 的整数
 * @returns 中文数字字符串
 *
 * @example
 * twoDigitToCn(1)  // '一'
 * twoDigitToCn(10) // '十'
 * twoDigitToCn(12) // '十二'
 * twoDigitToCn(20) // '二十'
 * twoDigitToCn(99) // '九十九'
 */
function twoDigitToCn(num: number): string {
  if (num < 10) {
    return CN_DIGITS[num]
  }

  const tens = Math.floor(num / 10)
  const ones = num % 10
  const tensPart = tens === 1 ? '十' : `${CN_DIGITS[tens]}十`

  return ones === 0 ? tensPart : `${tensPart}${CN_DIGITS[ones]}`
}

/**
 * 将 100~999 的三位数转为中文数字
 *
 * @param num - 100~999 的整数
 * @returns 中文数字字符串
 *
 * @example
 * threeDigitToCn(100) // '一百'
 * threeDigitToCn(105) // '一百零五'
 * threeDigitToCn(110) // '一百一十'
 * threeDigitToCn(999) // '九百九十九'
 */
function threeDigitToCn(num: number): string {
  const hundreds = Math.floor(num / 100)
  const remainder = num % 100
  const hundredsPart = `${CN_DIGITS[hundreds]}百`

  if (remainder === 0) {
    return hundredsPart
  }

  // 十位为 0 时需补 "零"（如 105 → 一百零五）
  if (remainder < 10) {
    return `${hundredsPart}零${CN_DIGITS[remainder]}`
  }

  return `${hundredsPart}${twoDigitToCn(remainder)}`
}

/**
 * 将阿拉伯数字转换为中文数字
 *
 * 支持范围：1 ~ 999，超出范围返回原数字的字符串形式。
 *
 * @param num - 要转换的阿拉伯数字
 * @returns 中文数字字符串
 *
 * @example
 * numToCn(1)    // '一'
 * numToCn(10)   // '十'
 * numToCn(12)   // '十二'
 * numToCn(20)   // '二十'
 * numToCn(99)   // '九十九'
 * numToCn(100)  // '一百'
 * numToCn(105)  // '一百零五'
 * numToCn(999)  // '九百九十九'
 * numToCn(0)    // '0'
 * numToCn(1000) // '1000'
 */
export function numToCn(num: number): string {
  if (num < 1 || num > 999) {
    return String(num)
  }

  if (num < 100) {
    return twoDigitToCn(num)
  }

  return threeDigitToCn(num)
}
