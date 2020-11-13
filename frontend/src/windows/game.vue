<template>
  <Window
    v-bind:title="'Game'"
    v-bind:top=10
    v-bind:left=420
    ref="window"
  >
    <div class="content">
      <canvas ref="canvas"/>
    </div>
  </Window>
</template>

<script>
import Window from './window'

const KEY_UP     = 1 << 0
const KEY_RIGHT  = 1 << 1
const KEY_DOWN   = 1 << 2
const KEY_LEFT   = 1 << 3
const KEY_ACTION = 1 << 4

export default {
  name: 'game',
  props: ['engine', 'vmPaused'],
  data: function() {
    return {
      canvasColors: {}
    }
  },
  components: {
    Window,
  },
  mounted: function() {
    this.$refs.canvas.width = this.engine.screenWidth
    this.$refs.canvas.height = this.engine.screenHeight

    this.canvasContext = this.$refs.canvas.getContext('2d')
    this.keysDown = {}

    window.addEventListener('keydown', this.onKeyDown)
    window.addEventListener('keyup', this.onKeyUp)
  },
  destroyed: function() {
    window.removeEventListener('keydown', this.onKeyDown)
    window.removeEventListener('keyup', this.onKeyUp)
  },
  methods: {
    render: function(pixels) {
      if (this.$refs.window.minimized) {
        return
      }

      let lastColor = 0
      let lineLen = 0
      let lineStart = 0

      for (let y = this.engine.screenHeight - 1; y >= 0; --y) {
        lineLen = 1
        lineStart = 0
        lastColor = pixels[y * this.engine.screenWidth]

        for (let x = 1; x < this.engine.screenWidth; ++x) {
          const color = pixels[y * this.engine.screenWidth + x]

          if ((color === lastColor) && (x < this.engine.screenWidth - 1)) {
            ++lineLen
          } else {
            if (!this.canvasColors[color]) {
              const r = (color & 0x0000ff)
              const g = (color & 0x00ff00) >> 8
              const b = (color & 0xff0000) >> 16
              this.canvasColors[color] = `rgba(${r}, ${g}, ${b}, 1)`
            }

            if (x === this.engine.screenWidth - 1) {
              ++lineLen
            }

            this.canvasContext.fillStyle = this.canvasColors[lastColor]
            this.canvasContext.fillRect(lineStart, y, lineLen, 1)

            lastColor = color
            lineLen = 1
            lineStart = x
          }
        }
      }
    },
    onKeyDown(event) {
      if (this.keysDown[event.code] || this.$refs.window.minimized || this.vmPaused) {
        return
      }


      switch(event.code) {
        case 'ArrowUp':
        case 'KeyW':
          this.engine.onKeyDown(KEY_UP)
          break

        case 'ArrowRight':
        case 'KeyD':
          this.engine.onKeyDown(KEY_RIGHT)
          break

        case 'ArrowDown':
        case 'KeyS':
          this.engine.onKeyDown(KEY_DOWN)
          break

        case 'ArrowLeft':
        case 'KeyA':
          this.engine.onKeyDown(KEY_LEFT)
          break

        case 'Space':
        case 'Enter':
          this.engine.onKeyDown(KEY_ACTION)
          break
      }

      this.keysDown[event.code] = true
    },
    onKeyUp(event) {
      if (!this.keysDown[event.code]) {
        return
      }

      switch(event.code) {
        case 'ArrowUp':
        case 'KeyW':
          this.engine.onKeyUp(KEY_UP)
          break

        case 'ArrowRight':
        case 'KeyD':
          this.engine.onKeyUp(KEY_RIGHT)
          break

        case 'ArrowDown':
        case 'KeyS':
          this.engine.onKeyUp(KEY_DOWN)
          break

        case 'ArrowLeft':
        case 'KeyA':
          this.engine.onKeyUp(KEY_LEFT)
          break

        case 'Space':
        case 'Enter':
          this.engine.onKeyUp(KEY_ACTION)
          break
      }

      this.keysDown[event.code] = false
    }
  }
}
</script>

<style lang="less" scoped>
  .content {
    padding: 0 5px;

    canvas {
      width: 640px;
      height: 400px;
      image-rendering: pixelated;
    }
  }
</style>