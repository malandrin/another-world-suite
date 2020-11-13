<template>
  <Window
    v-bind:title="'Threads'"
    v-bind:top=500
    v-bind:left=1080
  >
    <div v-if="!vmPaused" class="paused">
      Pause the game to see the values
    </div>
    <div v-else class="table">
      <div class="colHeader">
        <div
          v-for="i in '0123'"
          v-bind:key="`row_${i}`"
        >
          {{i}}
        </div>
      </div>
      <div class="group">
        <div class="rowHeader">
          <div
            v-for="i in '0123456789ABCDEF'"
            v-bind:key="`row_${i}`"
          >
            {{i}}
          </div>
        </div>
        <div class="content">
          <div
            v-for="(pc, index) in threadsInfo.threadsPc"
            v-bind:key="`thread_${index}`"
            class="thread"
            v-bind:class="{inactive: pc === '-', active: index === threadsInfo.activeThread}"
            v-on:click="gotoAddress(pc)"
          >
            {{pc}}
          </div>
        </div>
      </div>
    </div>
  </Window>
</template>

<script>
import _ from 'lodash'
import Window from './window'

export default {
  name: 'Threads',
  props: ['engine', 'vmPaused'],
  components: {
    Window
  },
  data: function() {
    return {
      threadsInfo: {
        activeThread: 0,
        threadsPc: []
      }
    }
  },
  methods: {
    refresh: function(threadsInfo) {
      this.threadsInfo = _.clone(threadsInfo)
    },
    gotoAddress: function(addr) {
      if (addr !== '-') {
        this.$emit('scroll-to-address', addr)
      }
    }
  }
}
</script>

<style lang="less" scoped>
  .paused {
    width: 650px;
    height: 104px;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 18px;
  }

  .table {
    display: flex;
    width: 650px;

    .colHeader {
      padding: 0px 5px 10px 15px;
      display: grid;
      grid-template-rows: repeat(4, 17px);
      grid-gap: 3px;
      margin-top: 17px;
      margin-left: 2px;
    }

    .group {
      display: flex;
      flex-direction: column;

      .rowHeader {
        padding: 0px 5px 0px 15px;
        display: grid;
        grid-template-columns: repeat(16, 34px);
        grid-gap: 3px;
      }

      .content {
        padding: 0px 5px 10px 15px;
        display: grid;
        grid-template-columns: repeat(16, 34px);
        grid-gap: 3px;

        .thread {
          display: flex;
          align-items: center;
          justify-content: center;
          color: #47C0C0;
          border: 1px solid transparent;
          height: 15px;

          &:hover:not(.inactive) {
            cursor: pointer;
            text-decoration: underline;
          }

          &.inactive {
            color: #C6C6C6;
          }

          &.active {
            background: #4B4B18;
            border: 1px solid #535320;
          }
        }
      }
    }
  }
</style>