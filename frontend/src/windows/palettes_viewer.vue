<template>
  <div class="palettesViewer">
    <div class="list">
      <div
        v-for="(palette, idx) in palettes"
        v-bind:key="`palette_item_${idx}`"
        v-bind:ref="`palette_item_${idx}`"
        class="paletteItem"
        v-bind:class="{active: idx === activePalette.idx}"
        v-on:click="selectPalette(idx)"
        v-bind:title="`${hexValue(idx)}`"
      >
        <div class="paletteItemContent">
          <div
            v-for="(color, cidx) in palette"
            v-bind:key="`palette_item_color_${cidx}`"
            class="paletteItemColor"
            v-bind:style="{background: color}"
          />
        </div>
      </div>
    </div>
    <div class="palette">
      <div
        v-for="(color, idx) in activePalette.colors"
        v-bind:key="`palette_color_${idx}`"
        class="color"
        v-bind:style="{background: color}"
        v-bind:title="`${hexValue(idx)}: ${color}`"
      />
    </div>
  </div>
</template>

<script>
import {int2Hex} from '@/utils'

export default {
  name: 'PalettesViewer',
  props: ['palettes', 'paletteIdx'],
  data: function() {
    return {
      activePalette: {}
    }
  },
  methods: {
    selectPalette(palette, scrollIntoView) {
      this.activePalette = {
        idx: palette,
        colors: this.palettes[palette]
      }

      if (scrollIntoView) {
        this.$refs[`palette_item_${palette}`][0].scrollIntoView()
      }
    },
    hexValue(value) {
      return int2Hex(value, 2)
    },
    getActivePaletteIdx() {
      return this.activePalette.idx
    }
  }
}
</script>

<style lang="less" scoped>
  .palettesViewer {
    height: 100%;
    display: flex;

    .list {
      overflow: auto;
      display: flex;
      flex-direction: column;
      align-items: center;
      width: 100px;
      flex-shrink: 0;

      .paletteItem {
        width: 70px;
        height: 70px;
        display: inline-flex;
        border: 3px solid transparent;
        cursor: pointer;

        &:not(:first-child) {
          margin-top: 5px;
        }

        &.active {
          border: 3px solid #47C0C0;
          border-radius: 2px;
        }

        .paletteItemContent {
          display: grid;
          grid-template-columns: repeat(4, 16px);
          grid-gap: 1px;

          .paletteItemColor {
            width: 16px;
            height: 16px;
          }
        }
      }
    }

    .palette {
      display: flex;
      flex-wrap: wrap;
      overflow: auto;
      padding: 5px;

      .color {
        width: 95px;
        height: 95px;
        margin: 4px;
      }
    }
  }
</style>