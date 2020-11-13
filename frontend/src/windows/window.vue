<template>
  <div
    class="window"
    v-bind:style="{top: topPosition + 'px', left: leftPosition + 'px', 'z-index': zIndex}"
    v-on:click="bringToFront"
    v-show="!minimized"
  >
    <div
      class="titlebar"
      v-on:mousedown="onMouseDown"
      v-on:mousemove="onMouseMove"
      v-on:mouseup="onMouseUp"
    >
      <div class="title">{{title}}</div>
      <i
        class="fas fa-window-minimize"
        v-on:click="minimized = true"
      ></i>
    </div>
    <slot></slot>
  </div>
</template>

<script>
import Global from '@/global'

export default {
  name: 'Window',
  props: ['title', 'top', 'left'],
  data: function() {
    return {
      moving: false,
      topPosition: 0,
      leftPosition: 0,
      zIndex: 0,
      prevMovingPositionX: 0,
      prevMovingPositionY: 0,
      minimized: false
    }
  },
  created: function() {
    this.topPosition = this.top
    this.leftPosition = this.left
    window.addEventListener('mousemove', this.onMouseMove)
  },
  destroyed: function() {
    window.removeEventListener('mousemove', this.onMouseMove)
  },
  methods: {
    onMouseDown: function(ev) {
      this.bringToFront()
      this.moving = true
      this.prevMovingPositionX = ev.screenX
      this.prevMovingPositionY = ev.screenY
    },
    onMouseMove: function(ev) {
      if (this.moving) {
        const deltaX = ev.screenX - this.prevMovingPositionX
        const deltaY = ev.screenY - this.prevMovingPositionY

        this.topPosition += deltaY
        this.leftPosition += deltaX

        this.prevMovingPositionX = ev.screenX
        this.prevMovingPositionY = ev.screenY
      }
    },
    onMouseUp: function() {
      this.moving = false
    },
    bringToFront: function() {
      ++Global.windowZIndex
      this.zIndex = Global.windowZIndex
    },
    minimize: function(minimize) {
      this.minimized = minimize
    }
  }
}
</script>

<style lang="less">
  .window {
    font-family: consolas, courier;
    font-size: 14px;
    background: #383838;
    color: #D4D4D4;
    border-radius: 4px;
    position: absolute;
    border: 1px solid #D4D0C8;
    top: 0;
    left: 0;
    user-select: none;
    box-shadow: 0px 0px 24px -4px rgba(0,0,0,0.75);

    .titlebar {
      background: #424242;
      border-radius: 4px;
      height: 25px;
      display: flex;
      align-items: center;
      justify-content: flex-start;
      padding: 0 10px;
      color: #C6C6C6;
      margin-bottom: 10px;
      cursor: pointer;

      .fa-window-minimize {
        margin-left: auto;
        padding: 0 5px;

        &:hover {
          color: white;
        }
      }
    }
  }
</style>