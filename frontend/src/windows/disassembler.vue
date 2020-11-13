<template>
  <Window
    v-bind:title="windowTitle"
    v-bind:top=10
    v-bind:left=10
    v-bind:style="{width: '400px'}"
  >
    <div class="scriptViewer">
      <ScriptViewer
        ref="scriptViewer"
        v-bind:debugging="true"
        v-bind:interactive="true"
        v-bind:threadInfo="activeThread"
        v-on:goto-resource-file="gotoResourceFile"
        v-on:goto-current-part-palette-file="gotoCurrentPartPaletteFile"
        v-on:goto-current-part-poly-file="gotoCurrentPartPolyFile"
      />
    </div>
    <div class="controls">
      <i
        v-show="vmPaused"
        class="button fas fa-play"
        v-on:click="vmContinue"
        title="Continue"
      />
      <i
        v-show="!vmPaused"
        class="button fas fa-pause"
        v-on:click="vmPause"
        title="Pause"
      />
      <i
        class="button far fa-caret-square-right"
        v-bind:class="{disabled: !vmPaused}"
        v-on:click="vmContinueUntilNextFrame"
        title="Continue Until Next Frame"
        style="font-size: 17px"
      />
      <i
        class="button fas fa-step-forward"
        v-bind:class="{disabled: !vmPaused}"
        v-on:click="vmStep"
        title="Step"
        style="font-size: 16px"
      />
      <div class="rightControls">
        <i
          class="button green fas fa-undo-alt"
          v-on:click="vmRestart"
          title="Restart"
        />
        <select
          class="level" v-model="level"
          v-on:change="vmRestart"
        >
          <option value="0xff">Protection Screen</option>
          <option value="0">Intro</option>
          <option value="1">Level 1 (Arrival)</option>
          <option value="2">Level 2 (Jail)</option>
          <option value="3">Level 3 (City)</option>
          <option value="4">Level 4 (Caves)</option>
          <option value="5">Level 5</option>
          <option value="6">Level 6</option>
          <option value="7">Level 7</option>
          <option value="8">Level 8</option>
          <option value="9">Level 9</option>
          <option value="10">Level 10</option>
          <option value="11">Level 11</option>
          <option value="12">Level 12</option>
          <option value="13">Level 13 (Palais)</option>
          <option value="14">Level 14 (Arena)</option>
          <option value="15">Level 15 (Baths)</option>
        </select>
      </div>
    </div>
  </Window>
</template>

<script>
import {int2Hex} from '@/utils'
import Global from '@/global'
import Window from './window'
import ScriptViewer from './script_viewer'

export default {
  name: 'Disassembler',
  props: ['engine', 'vmPaused'],
  components: {
    Window,
    ScriptViewer
  },
  data: function() {
    return {
      script: null,
      scriptId: null,
      level: 0,
      activeThread: {
        id: '00',
        pc: '0000',
        paused: true
      }
    }
  },
  computed: {
    windowTitle: function() {
      return `Disassembler [Thread ${this.activeThread.id}]`
    }
  },
  methods: {
    refresh: function(threadsInfo) {
      this.activeThread = {
        id: int2Hex(threadsInfo.activeThread, 2),
        pc: threadsInfo.threadsPc[threadsInfo.activeThread],
        paused: threadsInfo.paused
      }

      this.scrollTo(this.activeThread.pc)
    },
    scrollTo: function(addr) {
      this.$refs.scriptViewer.scrollTo(addr)
    },
    setScript: function(script, scriptId) {
      if (scriptId === this.scriptId) {
        return
      }

      this.script = script
      this.scriptId = scriptId
      this.$refs.scriptViewer.setScript(script)
    },
    vmStep: function() {
      if (this.vmPaused) {
        this.$emit('step')
      }
    },
    vmContinue: function() {
      this.$emit('continue')
    },
    vmContinueUntilNextFrame: function() {
      if (this.vmPaused) {
        this.$emit('continue-until-next-frame')
      }
    },
    vmPause: function() {
      this.$emit('pause')
    },
    vmRestart: function() {
      this.$emit('restart', parseInt(this.level))
    },
    hasBreakpoint: function(addr) {
      return this.$refs.scriptViewer.hasBreakpoint(addr)
    },
    gotoResourceFile: function(resourceId) {
      this.$emit('goto-resource-file', resourceId)
    },
    gotoCurrentPartPaletteFile(paletteIdx) {
      const paletteFileId = Global.resourcesIdByPart.find(i => i.script === this.scriptId).palette
      this.$emit('goto-resource-file', paletteFileId, paletteIdx)
    },
    gotoCurrentPartPolyFile(bufferId, polyOffset, x, y, zoom) {
      const resInfo = Global.resourcesIdByPart.find(i => i.script === this.scriptId)
      const polyFileId = bufferId === 1 ? resInfo.poly1 : resInfo.poly2
      this.$emit('goto-resource-file', polyFileId, {offset: polyOffset, x: x, y: y, zoom: zoom})
    }
  }
}
</script>

<style lang="less" scoped>
  .content {
    overflow: auto;
    height: 500px;
  }

  .scriptViewer {
    height: 764px;
  }

  .controls {
    height: 30px;
    background: #424242;
    color: #C6C6C6;
    display: flex;
    align-items: center;
    border-bottom-left-radius: 4px;
    border-bottom-right-radius: 4px;
    align-items: center;
    padding: 0 10px;

    .rightControls {
      margin-left: auto;

      .level {
        margin-left: 5px;
        outline: none;
      }
    }

    .button {
      cursor: pointer;
      padding: 3px;
      color: #75BEFF;

      &:not(:last-child) {
        margin-right: 5px;
      }

      &.disabled {
        opacity: 0.5;
        cursor: default;
      }

      &.green {
        color: #89D185;
        margin-right: 0;
      }
    }
  }
</style>