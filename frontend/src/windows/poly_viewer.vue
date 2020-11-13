<template>
  <div class="polyViewer">
    <div class="toolbar">
      <div class="label">Palette</div>
      <select
        class="palette" v-model="activePaletteIdx"
        v-on:change="render"
      >
        <option
          v-for="i in 16"
          v-bind:key="`palette_${i - 1}`"
          v-bind:value="i - 1"
        >
          {{hexValue(i - 1)}}
        </option>
      </select>
      <div class="coordsTitles">
        <div class="label">X</div>
        <div class="label">Y</div>
        <div class="label">Zoom</div>
      </div>
      <div class="coordsValues">
        <input type="text" v-model="x" v-on:change="render"/>
        <input type="text" v-model="y" v-on:change="render"/>
        <input type="text" v-model="zoom" v-on:change="render"/>
      </div>
      <div class="label">clear on draw</div>
      <input type="checkbox" v-model="clearOnDraw" style="margin-bottom: 5px"/>
      <div class="label">Offsets</div>
      <div class="offsets">
        <div
          v-for="offset in offsets"
          v-bind:key="`offset_${offset}`"
          v-bind:ref="`offset_${offset}`"
          class="offset"
          v-bind:class="{active: offset === activeOffset}"
          v-on:click="selectOffset(offset)"
        >
          {{offset}}
        </div>
      </div>
    </div>
    <canvas
      class="canvas"
      ref="canvas"
      v-on:mousemove="onMouseMove"
      v-on:mousewheel.prevent="onMouseWheel"
    />
  </div>
</template>

<script>
import _ from 'lodash'
import Global from '@/global'
import {int2Hex} from '@/utils'

export default {
  name: 'PolyViewer',
  props: ['engine', 'resources', 'poly'],
  data: function() {
    return {
      activePalette: null,
      activePaletteIdx: 1,
      activeOffset: 0,
      activeOffsetInt: 0,
      resInfo: null,
      x: 0,
      y: 0,
      zoom: 0,
      clearOnDraw: true,
      offsets: [],
    }
  },
  mounted: function() {
    this.$refs.canvas.width = this.engine.screenWidth
    this.$refs.canvas.height = this.engine.screenHeight
    this.canvasContext = this.$refs.canvas.getContext('2d')
  },
  methods: {
    setPolys(resInfo, offset, x, y, zoom) {
      const part = Global.resourcesIdByPart.find(i => i.poly1 === resInfo.id || i.poly2 === resInfo.id)
      const paletteFileId = part ? part.palette : Global.resourcesIdByPart[0].palette

      this.resInfo = resInfo
      this.offsets = resInfo.offsets
      this.x = x || 160
      this.y = y || 100
      this.zoom = zoom || 0x40
      this.activePalette = this.resources[paletteFileId]
      this.activeOffset = offset || this.offsets[0]
      this.activeOffsetInt = parseInt(this.activeOffset, 16)

      this.render()

      const self = this
      _.defer(function() {
        self.$refs[`offset_${self.activeOffset}`][0].scrollIntoView()
      })
    },
    selectOffset(offset) {
      this.activeOffset = offset
      this.activeOffsetInt = parseInt(this.activeOffset, 16)
      this.render()
    },
    render() {
      const buffer = this.engine.drawPolys(this.resInfo.id, this.activeOffsetInt, this.x, this.y, this.zoom)
      let idx = 0

      for (let y = 0; y < this.engine.screenHeight; ++y) {
        for (let x = 0; x < this.engine.screenWidth; ++x) {
          const colorIdx = buffer[idx++]

          if (this.clearOnDraw || colorIdx) {
            this.canvasContext.fillStyle = this.activePalette.content[this.activePaletteIdx][colorIdx]
            this.canvasContext.fillRect(x, y, 1, 1)
          }
        }
      }
    },
    hexValue(value) {
      return int2Hex(value, 2)
    },
    onMouseMove(event) {
      if (event.which === 1 && (event.movementX !== 0 || event.movementY !== 0)) {
        this.x += event.movementX
        this.y += event.movementY
        this.render()
      }
    },
    onMouseWheel(event) {
      this.zoom += event.deltaY / 10
      this.render()
    }
  }
}
</script>

<style lang="less" scoped>
  .polyViewer {
    height: 100%;
    display: flex;

    .toolbar {
      width: 100px;
      height: 100%;
      display: flex;
      flex-direction: column;

      .label {
        text-align: center;
        font-size: 13px;
      }

      .coordsTitles {
        margin-top: 5px;
        display: flex;

        div {
          width: 33%;
        }
      }

      .coordsValues {
        display: flex;
        margin-bottom: 5px;

        input {
          width: 33%;
        }
      }

      .offsets {
        flex-grow: 1;
        overflow: auto;
        border-top: 1px solid #d4d4d4;

        .offset {
          cursor: pointer;
          border: 1px solid transparent;

          &.active {
            background: #4B4B18;
            border: 1px solid #535320;
          }
        }
      }
    }

    .canvas {
      width: 640px;
      height: 400px;
      margin-left: 5px;
      image-rendering: pixelated;
    }
  }
</style>