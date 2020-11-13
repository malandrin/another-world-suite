<template>
  <div class="bitmapViewer">
    <canvas
      class="canvas"
      ref="canvas"
    />
  </div>
</template>

<script>

const paletteInfoPerBitmap = {
  0x12: [0x14, 4],
  0x13: [0x14, 8],
  0x43: [0x17, 3],
  0x44: [0x20, 8],
  0x45: [0x20, 8],
  0x46: [0x20, 8],
  0x47: [0x14, 2],
  0x48: [0x7d, 7],
  0x49: [0x7d, 7],
  0x53: [0x7d, 7],
  0x90: [0x26, 3],
  0x91: [0x26, 1]
}

export default {
  name: 'bitmapViewer',
  props: ['engine', 'resources', 'bitmap'],
  mounted: function() {
    this.$refs.canvas.width = this.engine.screenWidth
    this.$refs.canvas.height = this.engine.screenHeight
    this.canvasContext = this.$refs.canvas.getContext('2d')
  },
  methods: {
    setBitmap(resInfo) {
      const paletteInfo = paletteInfoPerBitmap[resInfo.id]

      const buffer = resInfo.content
      const palette = this.resources[paletteInfo[0]]
      const paletteIdx = paletteInfo[1]
      let idx = 0

      for (let y = 0; y < this.engine.screenHeight; ++y) {
        for (let x = 0; x < this.engine.screenWidth; ++x) {
          this.canvasContext.fillStyle = palette.content[paletteIdx][buffer[idx++]]
          this.canvasContext.fillRect(x, y, 1, 1)
        }
      }
    }
  }
}
</script>

<style lang="less" scoped>
  .bitmapViewer {
    height: 100%;

    .canvas {
      width: 640px;
      height: 400px;
      image-rendering: pixelated;
    }
  }
</style>