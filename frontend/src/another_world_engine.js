import AnotherWorldEngineWasm from './assets/awlib.bin'
import {int2Hex, rgb2Hex} from '@/utils'
import Global from '@/global'

const SharedMemorySize = 3 * 1024 * 1204

export class AnotherWorldEngine {
  constructor() {
  }

  async init(gameData) {
    let imports = {
      wbg: {}
    }

    imports.wbg.__wbindgen_throw = function() {
      debugger
    }

    const wai = await WebAssembly.instantiate(AnotherWorldEngineWasm, imports)
    this.wasm = wai.instance.exports
    this.anotherWorldEngine = this.wasm.anotherworldengine_new()

    this.screenWidth = this.wasm.anotherworldengine_get_screen_width(this.anotherWorldEngine)
    this.screenHeight = this.wasm.anotherworldengine_get_screen_height(this.anotherWorldEngine)

    const dataPtr = this.wasm.anotherworldengine_get_shared_memory_pointer(this.anotherWorldEngine)
    let dataArray = new Uint8Array(this.wasm.memory.buffer, dataPtr, gameData.byteLength)
    dataArray.set(new Uint8Array(gameData))

    this.wasm.anotherworldengine_init(this.anotherWorldEngine)
  }

  end() {
    this.wasm.__wbg_anotherworldengine_free(this.anotherWorldEngine)
  }

  getFrameBuffer() {
    this.wasm.anotherworldengine_get_frame_buffer(this.anotherWorldEngine)
    const dataPtr = this.wasm.anotherworldengine_get_shared_memory_pointer(this.anotherWorldEngine)
    return new Uint32Array(this.wasm.memory.buffer, dataPtr, this.screenWidth * this.screenHeight)
  }

  vmGetCurrentPC() {
    return this.wasm.anotherworldengine_vm_get_current_pc(this.anotherWorldEngine)
  }

  vmStep() {
    return this.wasm.anotherworldengine_vm_step(this.anotherWorldEngine)
  }

  vmRestart(part) {
    this.wasm.anotherworldengine_vm_restart(this.anotherWorldEngine, part)
  }

  onKeyDown(key) {
    this.wasm.anotherworldengine_on_key_down(this.anotherWorldEngine, key)
  }

  onKeyUp(key) {
    this.wasm.anotherworldengine_on_key_up(this.anotherWorldEngine, key)
  }

  drawPolys(fileId, offset, x, y, zoom) {
    this.wasm.anotherworldengine_draw_poly(this.anotherWorldEngine, fileId, offset, x, y, zoom)
    const dataPtr = this.wasm.anotherworldengine_get_shared_memory_pointer(this.anotherWorldEngine)
    return new Uint8Array(this.wasm.memory.buffer, dataPtr, this.screenWidth * this.screenHeight)
  }

  getRegisters() {
    const dataPtr = this.wasm.anotherworldengine_get_registers(this.anotherWorldEngine)
    const dataArray = new Int16Array(this.wasm.memory.buffer, dataPtr, 256)
    let registers = []

    for (let i = 0; i < dataArray.length; ++i) {
      registers.push(dataArray[i])
    }

    return registers
  }

  getThreadsInfo() {
    this.wasm.anotherworldengine_build_threads_info(this.anotherWorldEngine)

    const dataPtr = this.wasm.anotherworldengine_get_shared_memory_pointer(this.anotherWorldEngine)
    const dataArray = new Uint16Array(this.wasm.memory.buffer, dataPtr, 65)
    let info = {
      activeThread: dataArray[0],
      threadsPc: []
    }

    for (let i = 1; i < dataArray.length; ++i) {
      const value = dataArray[i]
      info.threadsPc.push(value == 0xffff ? '-' : int2Hex(value, 4))
    }

    return info
  }

  getResourcesInfo() {
    this.wasm.anotherworldengine_build_resources_info(this.anotherWorldEngine)

    const dataPtr = this.wasm.anotherworldengine_get_shared_memory_pointer(this.anotherWorldEngine)
    const dataArray = new Uint8Array(this.wasm.memory.buffer, dataPtr, SharedMemorySize)
    const numFiles = dataArray[0]
    let polyOffsets = {}
    let info = []
    let idx = 1

    for (let i = 0; i < numFiles; ++i) {
      let data = {
        type: dataArray[idx++],
        size: dataArray[idx++] | (dataArray[idx++] << 8),
        id: i
      }

      switch(Global.resources.classByType[data.type]) {
        case 'sound': {
          if (data.size > 0) {
            const len = (dataArray[idx] << 8 | dataArray[idx + 1]) * 2
            // const loopLen = (dataArray[idx + 2] << 8 | dataArray[idx + 3]) * 2

            data.audioBuffer = Global.audioContext.createBuffer(1, len, 11024)
            let channelBuffer = data.audioBuffer.getChannelData(0)

            for (let b = 0; b < channelBuffer.length; ++b) {
              channelBuffer[b] = dataArray[idx + 8 + b] / 255
            }
          }

          idx += data.size
        }
        break

        case 'palette': {
          const paletteInfo = this.buildPalettesInfo(dataArray, idx)
          data.content = paletteInfo.content
          idx += paletteInfo.size
        }
        break

        case 'script': {
          const scriptInfo = this.disassembleScript(dataArray, idx)
          data.content = scriptInfo.content
          idx += scriptInfo.size
        }
        break

        default:
          data.content = dataArray.slice(idx, idx + data.size)
          idx += data.size
          break
      }

      info.push(data)
    }

    // postprocess scripts
    for (let file of info) {
      if (Global.resources.classByType[file.type] === 'script') {
        // parse asm code to get the address for a call, or a register value
        for (let line of file.content) {
          let codeParts = line.asmCode.split(' ')
          line.parts = [{type: 'opcode', value: codeParts[0]}]

          switch(codeParts[0]) {
            case 'CALL':
            case 'JMP':
              line.parts.push({type: 'addr', value: codeParts[1]})
              break

            case 'JNZ':
              line.parts.push({type: 'text', value: codeParts[1]})
              line.parts.push({type: 'addr', value: codeParts[2]})
              break

            case 'CJZ':
            case 'CJNZ':
            case 'CJG':
            case 'CJGE':
            case 'CJL':
            case 'CJLE':
              line.parts.push({type: 'text', value: codeParts[1]})
              line.parts.push({type: 'text', value: codeParts[2]})
              line.parts.push({type: 'addr', value: codeParts[3]})
              break

            case 'SETVEC':
              line.parts.push({type: 'text', value: codeParts[1]})
              line.parts.push({type: 'addr', value: codeParts[2]})
              break

            case 'SETPAL':
              line.parts.push({type: 'palette', value: codeParts[1].substring(0, 2)})
              break

            case 'LDRES': {
              const resourceId = parseInt(codeParts[1], 16)

              if (resourceId > info.length) { // this is to load a game part
                line.parts.push({type: 'part', value: codeParts[1]})
              } else {
                line.parts.push({type: Global.resources.classByType[info[resourceId].type], value: codeParts[1].substring(2, 4)})
              }
            }
            break

            case 'SND':
              line.parts.push({type: 'sound', value: codeParts[1].substring(2, 4)})
              line.parts.push({type: 'text', value: codeParts.slice(2, codeParts.length).join(' ')})
              break

            case 'DRAWPOLY1':
            case 'DRAWPOLY2': {
              const offset =  codeParts[1].substring(0, 4)
              const x = codeParts[2].replace(',', '')
              const y = codeParts[3].replace(',', '')
              const zoom = codeParts[4]

              line.parts[0].value = 'DRAWPOLY'
              line.parts.push({type: 'polyBuffer', value: offset})
              line.parts.push({type: 'text', value: `, ${codeParts.slice(2, codeParts.length).join(' ')}`})
              line.params = {bufferId: codeParts[0] === 'DRAWPOLY1' ? 1 : 2, x: x, y: y, zoom: zoom}

              const part = Global.resourcesIdByPart.find(i => i.script === file.id)

              if (part) {
                const polyFileId = codeParts[0] === 'DRAWPOLY1' ? part.poly1 : part.poly2

                if (!(polyFileId in polyOffsets)) {
                  polyOffsets[polyFileId] = []
                }

                if (polyOffsets[polyFileId].indexOf(offset) === -1) {
                  polyOffsets[polyFileId].push(offset)
                }
              }
            }
            break

            default:
              line.parts.push({type: 'text', value: codeParts.slice(1, codeParts.length).join(' ')})
              break
          }
        }
      }
    }

    // postprocess poly files
    for (const polyFileId in polyOffsets) {
      info[parseInt(polyFileId)].offsets = polyOffsets[polyFileId].sort()
    }

    return info
  }

  getActiveScriptFileId() {
    return this.wasm.anotherworldengine_get_active_script_file_id(this.anotherWorldEngine)
  }

  buildPalettesInfo(dataArray, idx) {
    let palettes = []
    let palette = []
    let myIdx = idx
    const numPalettes = dataArray[myIdx++]

    for (let p = 0; p < numPalettes; ++p) {
      for (let c = 0; c < 16; ++c) {
        palette.push(rgb2Hex(dataArray[myIdx++], dataArray[myIdx++], dataArray[myIdx++]))
      }

      palettes.push(palette)
      palette = []
    }

    return {
      content: palettes,
      size: myIdx - idx
    }
  }

  disassembleScript(dataArray, idx) {
    const textDecoder = new TextDecoder()
    let myIdx = idx
    const numEntries = dataArray[myIdx] | (dataArray[myIdx + 1] << 8)
    myIdx += 2

    let disassembledScript = []

    for (let i = 0; i < numEntries; ++i) {
      const addr = dataArray[myIdx++] | (dataArray[myIdx++] << 8)

      let entry = {
        addr: int2Hex(addr, 4),
        intAddr: addr
      }

      const asmCodeLen = dataArray[myIdx++]
      entry.asmCode = textDecoder.decode(dataArray.slice(myIdx, myIdx + asmCodeLen))

      disassembledScript.push(entry)
      myIdx += asmCodeLen
    }

    return {
      content: disassembledScript,
      size: myIdx - idx
    }
  }
}