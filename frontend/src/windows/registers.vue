<template>
  <Window
    v-bind:title="'Registers'"
    v-bind:top=460
    v-bind:left=420
    v-bind:style="{width: '650px'}"
  >
    <div v-if="!vmPaused" class="paused">
      Pause the game to see the values
    </div>
    <div v-else class="table">
      <div class="colHeader">
        <div
          v-for="i in '0123456789ABCDEF'"
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
            class="register"
            v-for="(value, index) in registers"
            v-bind:key="`reg_${index}`"
            v-bind:class="{negative: value < 0, hasValue: value > 0}"
            v-bind:title="value"
          >
            {{hexValue(value)}}
          </div>
        </div>
      </div>
    </div>
  </Window>
</template>

<script>
import _ from 'lodash'
import {int2Hex} from '@/utils'
import Window from './window'

export default {
  name: 'Registers',
  props: ['engine', 'vmPaused'],
  components: {
    Window
  },
  data: function() {
    return {
      registers: []
    }
  },
  methods: {
    refresh() {
      this.registers = _.clone(this.engine.getRegisters())
    },
    hexValue(value) {
      return int2Hex(Math.abs(value), 4)
    }
  }
}
</script>

<style lang="less" scoped>
  .paused {
    width: 650px;
    height: 344px;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 18px;
  }

  .table {
    display: flex;

    .colHeader {
      padding: 0px 5px 10px 5px;
      display: grid;
      grid-template-rows: repeat(16, 17px);
      grid-gap: 3px;
      margin-top: 17px;
      margin-left: 12px;
    }

    .group {
      display: flex;
      flex-direction: column;

      .rowHeader {
        padding: 0px 15px 0px 10px;
        display: grid;
        grid-template-columns: repeat(16, 33px);
        grid-gap: 3px;
      }

      .content {
        display: grid;
        grid-template-columns: repeat(16, 33px);
        grid-gap: 3px;
        padding-left: 10px;
        padding-right: 15px;

        .register {
          justify-self: center;
          align-self: center;
          width: 100%;
          height: 17px;
          font-size: 13px;
          display: flex;
          align-items: center;
          justify-content: center;

          &.negative {
            color: red;
          }

          &.hasValue {
            color: lightgreen;
          }
        }
      }
    }
  }
</style>