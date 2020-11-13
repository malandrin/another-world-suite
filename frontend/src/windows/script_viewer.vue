<template>
  <div class="scriptViewer">
    <div v-if="loading" class="loadingMessage">
      Loading...
    </div>
    <div v-else class="list" ref="list">
      <div
        v-for="(line, index) in script"
        v-bind:key="`line_${index}`"
        class="line"
        v-bind:class="{interactive: isInteractive, highlighted: isLineHighlighted(line.addr)}"
      >
        <div
          class="breakpointArea"
          v-bind:class="{interactive: isInteractive, active: breakpoints[line.intAddr]}"
          v-on:click="toggleBreakpoint(line.intAddr)"
        />
        <div v-bind:ref="`addr_${line.addr}`" class="address">{{line.addr}}:</div>
        <div class="parts">
          <div
            v-for="(part, pindex) in line.parts"
            v-bind:key="`part_${pindex}`"
            class="part" v-bind:class="part.type"
          >
            <div
              v-on:click="onPartClick(part, line)"
            >
              {{part.value}}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import Vue from 'vue'
import _ from 'lodash'

export default {
  name: 'ScriptViewer',
  props: ['interactive', 'threadInfo'],
  data: function() {
    return {
      isInteractive: this.interactive,
      loading: true,
      script: [],
      breakpoints: {}
    }
  },
  methods: {
    setScript(script, scrollPos) {
      this.loading = true
      this.script = script
      this.breakpoints = {}

      const self = this

      _.defer(function() {
        self.loading = false

        if (scrollPos) {
          _.defer(function() {
            self.$refs.list.scrollTop = scrollPos
          })
        }
      })
    },
    scrollTo(addr) {
      const addrLine = this.$refs[`addr_${addr}`]

      if (addrLine && addrLine.length > 0) {
        addrLine[0].scrollIntoView()
      }
    },
    onPartClick(part, line) {
      switch(part.type) {
        case 'addr':
          this.scrollTo(part.value)
          break

        case 'sound':
        case 'music':
        case 'bitmap':
          this.$emit('goto-resource-file', parseInt(part.value, 16))
          break

        case 'palette':
          this.$emit('goto-current-part-palette-file', parseInt(part.value, 16))
          break

        case 'polyBuffer': {
          const x = parseInt(line.params.x) || 0
          const y = parseInt(line.params.x) || 0
          const zoom = parseInt(line.params.x) || 0x40
          this.$emit('goto-current-part-poly-file', line.params.bufferId, part.value, x, y, zoom)
        }
        break
      }
    },
    isLineHighlighted(addr) {
      if (this.isInteractive && this.threadInfo.paused) {
        return addr === this.threadInfo.pc
      }

      return false
    },
    toggleBreakpoint(intAddr) {
      if (this.isInteractive) {
        Vue.set(this.breakpoints, intAddr, !this.breakpoints[intAddr])
      }
    },
    hasBreakpoint(intAddr) {
      return this.isInteractive && this.breakpoints[intAddr]
    },
    getScrollPosition() {
      return this.$refs.list.scrollTop
    }
  }
}
</script>

<style lang="less" scoped>
  @import '~@/cssvars.less';

  .scriptViewer {
    height: 100%;

    .loadingMessage {
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      height: 100%;
    }

    .list {
      height: 100%;
      overflow: auto;

      .line {
        display: flex;
        align-items: center;
        padding-left: 5px;
        padding-right: 15px;
        border: 1px solid transparent;
        height: 17px;

        &.interactive {
          &.highlighted {
            background: #4B4B18;
            border: 1px solid #535320;
          }
        }

        .breakpointArea {
          width: 15px;
          height: 100%;
          border-top: 1px solid transparent;
          border-bottom: 1px solid transparent;
          margin-right: 5px;

          &.interactive {
            cursor: pointer;

            &.active {
              &::after {
                content: "⬤";
                color: #E51400;
                font-size: 10px;
              }
            }

            &:not(.active) {
              &:hover {
                &::after {
                  content: "⬤";
                  color: #6E1A12;
                  font-size: 10px;
                }
              }
            }
          }
        }

        .address {
          margin-right: 10px;
          color: #47C0C0;
        }

        .parts {
          display: flex;

          .part {
            margin-right: 8px;
          }

          .opcode {
            width: 90px;
            text-align: left;
          }

          .addr {
            color: #47C0C0;
            cursor: pointer;

            &:hover {
              text-decoration: underline;
            }
          }

          .sound,
          .music,
          .bitmap,
          .polyBuffer,
          .palette {
            cursor: pointer;

            &:hover {
              text-decoration: underline;
            }
          }

          .sound {
            color: @resourceSound;
          }

          .music {
            color: @resourceMusic;
          }

          .bitmap {
            color: @resourceBitmap;
            margin-right: 0px;
          }

          .palette {
            color: @resourcePalette;
          }

          .script {
            color: @resourceScript;
          }

          .polyBuffer {
            color: @resourcePolyBuffer;
            margin-right: 0px;
          }
        }
      }
    }
  }
</style>