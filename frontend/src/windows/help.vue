<template>
  <Window
    v-bind:title="'Help'"
    v-bind:top=48
    v-bind:left=614
    ref="window"
  >
    <div class="content">
      <div class="list">
        <div class="category">Windows:</div>
        <div class="subItems">
          <div>&#8226; You can move a window by dragging and dropping its title bar.</div>
          <div>&#8226; A window can be minimized clicking in <div class="fas fa-window-minimize"/> or in its name in the bottom bar.</div>
        </div>
        <div class="category">Disassembler:</div>
        <div class="subItems">
          <div>&#8226; To run the game press <div style="color: #75BEFF" class="fas fa-play"/></div>
          <div>&#8226; To pause the game while it is running press <div style="color: #75BEFF" class="fas fa-pause"/></div>
          <div>&#8226; To execute the current instruction, while the game is paused, press <div style="color: #75BEFF" class="fas fa-step-forward"/></div>
          <div>&#8226; To run the game until the next frame press <div style="color: #75BEFF" class="far fa-caret-square-right"/></div>
          <div>&#8226; You can select the starting level selecting it from the dropdown.</div>
          <div>&#8226; To restart the game, starting in the selected level, press <div style="color: #89D185" class="fas fa-undo-alt"/></div>
          <div>&#8226; You can toggle breakpoints clicking in the empty area next to the addresses. The red circle means there is a breakpoint in that line.</div>
          <div>&#8226; You can click in <span style="color: #47C0C0">addresses</span> to scroll the view there. You can also click in <span style="color: #A0C134">any</span> <span style="color: #E8BE1C">resource</span> <span style="color: #3495E4">id</span> to display it in the resources window.</div>
        </div>
        <div class="category">Resources:</div>
        <div class="subItems">
          <div>&#8226; Select a resource from the list to display information about it.</div>
          <div>&#8226; Move to the previous/next selected resource clicking <div class="fas fa-arrow-left"/> and <div class="fas fa-arrow-right"/></div>
        </div>
        <div class="category">Registers:</div>
        <div class="subItems">
          <div>&#8226; Values in <span style="color: red">red</span> mean they are negatives. Values in <span style="color: lightgreen">green</span> mean they are positive and different from 0.</div>
        </div>
        <div class="category">Control keys:</div>
        <div class="subItems">
          <div>&#8226; Arrow left or A: move to left</div>
          <div>&#8226; Arrow right or D: move to right</div>
          <div>&#8226; Arrow up or W: jump</div>
          <div>&#8226; Arrow down or S: crouch</div>
          <div>&#8226; Space or Enter: action</div>
        </div>
        <div
          class="close"
          v-on:click="minimize"
        >
          Close
        </div>
      </div>
    </div>
  </Window>
</template>

<script>
import Window from './window'

export default {
  name: 'help',
  components: {
    Window,
  },
  methods: {
    minimize: function() {
      this.$refs.window.minimize(true)
    }
  }
}
</script>

<style lang="less" scoped>
  .content {
    padding: 0 5px;

    width: 600px;
    text-align: left;
    padding: 0px 20px;
    overflow: auto;

    .category {
      color: white;
      font-size: 16px;

      &:not(:first-child) {
        margin-top: 14px;
      }
    }

    .subItems {
      margin-left: 18px;
    }

    .close {
      text-align: center;
      padding: 5px;
      background: #C0C0C0;
      color: #2c3e50;
      cursor: pointer;
      margin-top: 15px;
      margin-bottom: 15px;
      border-top: 1px solid white;
      border-left: 1px solid white;
      border-right: 1px solid black;
      border-bottom: 1px solid black;
    }

    div {
      margin-bottom: 4px;
    }
  }
</style>