export function int2Hex(value, minLen) {
  minLen = minLen || 0

  let hexValue = value.toString(16).toUpperCase()

  if (minLen > hexValue.length) {
    const padding = minLen - hexValue.length

    for (let i = 0; i < padding; ++i) {
      hexValue = '0' + hexValue
    }
  }

  return hexValue
}

export function rgb2Hex(r, g, b) {
  return `#${int2Hex(r, 2)}${int2Hex(g, 2)}${int2Hex(b, 2)}`
}