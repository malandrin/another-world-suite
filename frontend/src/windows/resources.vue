<template>
  <Window
    v-bind:title="windowTitle"
    v-bind:top=10
    v-bind:left=1080
    v-bind:style="{width: '755px'}"
  >
    <div class="content">
      <div class="toolbar">
        <i
          class="fas fa-arrow-left button"
          v-bind:class="{disabled: navigationHistory.idx === 0}"
          v-on:click="navigateHistory(true)"
        />
        <i
          class="fas fa-arrow-right button"
          v-bind:class="{disabled: navigationHistory.idx >= navigationHistory.queue.length - 1}"
          v-on:click="navigateHistory(false)"
        />
        <input
          type="text"
          class="input"
          v-on:focus="onInputClick"
          v-model="searchInput"
        />
      </div>
      <div
        class="resourcesListMask"
        v-show="resourcesListVisible"
        v-on:click="resourcesListVisible = false"
      />
      <div
        class="resourcesList"
        v-show="resourcesListVisible"
      >
        <input
          ref="searchInput"
          type="text"
          class="input"
          v-model="searchInput"
          v-on:keydown="onSearchInputKeydown"
        />
        <div
          class="list"
          ref="resourcesList"
        >
          <div v-if="resourcesList.length === 0" class="noResults">
            No results match your search criteria
          </div>
          <div
            v-else
            v-for="(resource, index) in resourcesList"
            v-bind:key="`resource_${index}`"
            v-bind:ref="`resource_${index}`"
            class="resource"
            v-bind:class="{active: index === searchElementActiveIdx}"
            v-on:click="selectResource(index)"
          >
            <div class="badge" v-bind:class="classByType[resource.type]"/>
            {{hexValue(resource.id)}} - {{resourceNameByType[resource.type]}} ({{resource.size}} bytes)
          </div>
        </div>
      </div>

      <div class="resourceViewer">
        <div
          class="noPreviewAvailable"
          v-if="activeResourceInfo.size === 0 || activeResourceInfo.type === 1 || activeResourceInfo.type === 6"
        >
          No preview available
        </div>
        <SoundViewer
          v-else-if="activeResourceInfo.type === 0"
          ref="soundViewer"
        />
        <BitmapViewer
          v-else-if="activeResourceInfo.type === 2"
          v-bind:engine="engine"
          v-bind:resources="resources"
          v-bind:bitmap="activeResourceInfo.content"
          ref="bitmapViewer"
        />
        <PalettesViewer
          v-else-if="activeResourceInfo.type === 3"
          v-bind:palettes="activeResourceInfo.content"
          ref="palettesViewer"
        />
        <ScriptViewer
          v-else-if="activeResourceInfo.type === 4"
          ref="scriptViewer"
          v-bind:interactive="false"
          v-on:goto-resource-file="gotoResourceFile"
          v-on:goto-current-part-palette-file="gotoCurrentPartPaletteFile"
          v-on:goto-current-part-poly-file="gotoCurrentPartPolyFile"
        />
        <PolyViewer
          v-else-if="activeResourceInfo.type === 5"
          v-bind:engine="engine"
          v-bind:resources="resources"
          v-bind:poly="activeResourceInfo.content"
          ref="polysViewer"
        />
      </div>
    </div>
  </Window>
</template>

<script>
import _ from 'lodash'
import Window from './window'
import PalettesViewer from './palettes_viewer'
import ScriptViewer from './script_viewer'
import PolyViewer from './poly_viewer'
import SoundViewer from './sound_viewer'
import BitmapViewer from './bitmap_viewer'
import Global from '@/global'
import {int2Hex} from '@/utils'

export default {
  name: 'Resources',
  props: ['engine', 'resources'],
  data: function() {
    return {
      classByType: Global.resources.classByType,
      resourceNameByType: Global.resources.resourceNameByType,
      resourcesListVisible: false,
      searchInput: '',
      searchElementActiveIdx: 0,
      activeResourceInfo: {},
      audioSource: null,
      navigationHistory: {
        queue: [],
        idx: 0
      }
    }
  },
  components: {
    Window,
    PalettesViewer,
    ScriptViewer,
    PolyViewer,
    SoundViewer,
    BitmapViewer
  },
  watch: {
    searchInput: function() {
      this.searchElementActiveIdx = 0
    }
  },
  computed: {
    windowTitle: function() {
      let title = 'Resources'

      if (!_.isEmpty(this.activeResourceInfo)) {
        title += ` [${int2Hex(this.activeResourceInfo.id, 2)}: ${this.resourceNameByType[this.activeResourceInfo.type]}]`
      }

      return title
    },
    resourcesList: function() {
      let resources = []
      const searchString = this.searchInput.toLowerCase()

      for (let i = 0; i < this.resources.length; ++i) {
        const res = this.resources[i]
        let addResource = true

        if (searchString) {
          const resourceSearchString = `${this.hexValue(i)} ${this.resourceNameByType[res.type]}`
          addResource = resourceSearchString.toLowerCase().indexOf(searchString) !== -1
        }

        if (addResource) {
          let info = _.clone(res)
          info.id = i
          resources.push(info)
        }
      }

      return resources
    }
  },
  destroyed: function() {
    if (this.audioSource) {
      this.audioSource.disconnect()
    }
  },
  methods: {
    elementIsVisible: function (elem) {
      var elemBB = elem.getBoundingClientRect()
      var parentBB = this.$refs.resourcesList.getBoundingClientRect()

      if (elemBB.top < parentBB.top) {
        return false
      }

      if (elemBB.top >= parentBB.bottom) {
        return false
      }

      return true
    },
    onInputClick() {
      this.resourcesListVisible = true
      this.searchElementActiveIdx = 0

      const self = this

      _.defer(function() {
        self.$refs.searchInput.focus()
      })
    },
    onSearchInputKeydown(event) {
      switch(event.key) {
        case 'ArrowDown': {
          this.searchElementActiveIdx = (this.searchElementActiveIdx + 1) % this.resourcesList.length
          const element = this.$refs[`resource_${this.searchElementActiveIdx}`][0]

          if (!this.elementIsVisible(element)) {
            element.scrollIntoView()
          }
        }
        break

        case 'ArrowUp': {
          this.searchElementActiveIdx = (this.searchElementActiveIdx > 0 ? this.searchElementActiveIdx : this.resourcesList.length) - 1
          const element = this.$refs[`resource_${this.searchElementActiveIdx}`][0]

          if (!this.elementIsVisible(element)) {
            element.scrollIntoView()
          }
        }
        break

        case 'Enter':
          this.selectResource(this.searchElementActiveIdx)
          break

        case 'Escape':
          this.resourcesListVisible = false
          break
      }
    },
    hexValue(value) {
      return int2Hex(value, 2)
    },
    selectResource(idx, param) {
      this.gotoResourceFile(this.resourcesList[idx].id, param)
      this.resourcesListVisible = false
    },
    gotoResourceFile(fileId, param) {
      const resInfo = this.resources[fileId]
      this.addNavigationHistory()
      this.setResourceInfo(resInfo, param)
    },
    gotoCurrentPartPaletteFile(paletteIdx) {
      const paletteFileId = Global.resourcesIdByPart.find(i => i.script === this.activeResourceInfo.id).palette
      this.gotoResourceFile(paletteFileId, paletteIdx)
    },
    gotoCurrentPartPolyFile(bufferId, polyOffset, x, y, zoom) {
      const resInfo = Global.resourcesIdByPart.find(i => i.script === this.activeResourceInfo.id)
      const polyFileId = bufferId === 1 ? resInfo.poly1 : resInfo.poly2
      this.gotoResourceFile(polyFileId, {offset: polyOffset, x: x, y: y, zoom: zoom})
    },
    setResourceInfo(resInfo, param) {
      this.activeResourceInfo = resInfo

      const self = this

      switch(Global.resources.classByType[resInfo.type]) {
        case 'sound':
          _.defer(function() {
            if (self.$refs.soundViewer) {
              self.$refs.soundViewer.setAudioBuffer(self.activeResourceInfo.audioBuffer)
            }
          })
          break

        case 'script':
          _.defer(function() {
            self.$refs.scriptViewer.setScript(self.activeResourceInfo.content, param)
          })
          break

        case 'palette':
          _.defer(function() {
            self.$refs.palettesViewer.selectPalette(param || 0, param)
          })
          break

        case 'polyBuffer':
          _.defer(function() {
            const params = param || {}
            self.$refs.polysViewer.setPolys(self.activeResourceInfo, params.offset, params.x, params.y, params.zoom)
          })
          break

        case 'bitmap':
          _.defer(function() {
            self.$refs.bitmapViewer.setBitmap(self.activeResourceInfo)
          })
          break
      }
    },
    addNavigationHistory: function() {
      if (!_.isEmpty(this.activeResourceInfo)) {
        // if it's in the middle of the queue, discard from current idx until the end
        if (this.navigationHistory.idx < this.navigationHistory.queue.length) {
          this.navigationHistory.queue.splice(this.navigationHistory.idx, this.navigationHistory.queue.length - this.navigationHistory.idx)
        }

        let historyData = {
          resource: _.clone(this.activeResourceInfo),
          typename: Global.resources.classByType[this.activeResourceInfo.type],
        }

        switch(historyData.typename) {
          case 'script':
            historyData.param = this.$refs.scriptViewer.getScrollPosition()
            break

          case 'palette':
            historyData.param = this.$refs.palettesViewer.getActivePaletteIdx()
            break
        }

        this.navigationHistory.queue.push(historyData)
        this.navigationHistory.idx = this.navigationHistory.queue.length
      }
    },
    navigateHistory: function(back) {
      let entryToUpdate = null

      if (back) {
        if (this.navigationHistory.queue.length > 0 && this.navigationHistory.idx > 0) {
          const navIdx = this.navigationHistory.idx - 1

          if (this.navigationHistory.idx === this.navigationHistory.queue.length &&
              this.navigationHistory.queue[this.navigationHistory.queue.length - 1].resource.id !== this.activeResourceInfo.id) {
            this.addNavigationHistory()
          } else {
            entryToUpdate = this.navigationHistory.queue[this.navigationHistory.idx]
          }

          this.navigationHistory.idx = navIdx

          const navEntry = this.navigationHistory.queue[this.navigationHistory.idx]
          this.setResourceInfo(_.clone(navEntry.resource), navEntry.param)
        }
      } else {
        if (this.navigationHistory.idx < this.navigationHistory.queue.length - 1) {
          entryToUpdate = this.navigationHistory.queue[this.navigationHistory.idx]
          ++this.navigationHistory.idx

          const navEntry = this.navigationHistory.queue[this.navigationHistory.idx]
          this.setResourceInfo(_.clone(navEntry.resource), navEntry.param)
        }
      }

      if (entryToUpdate) {
        switch(entryToUpdate.typename) {
          case 'script':
            entryToUpdate.param = this.$refs.scriptViewer.getScrollPosition()
            break

          case 'palette':
            entryToUpdate.param = this.$refs.palettesViewer.getActivePaletteIdx()
            break
        }
      }
    }
  }
}
</script>

<style lang="less" scoped>
  @import '~@/cssvars.less';

  .content {
    height: 445px;
    display: flex;
    flex-direction: column;
    position: relative;

    .toolbar {
      height: 30px;
      display: flex;
      align-items: center;
      padding: 0 20px 5px 5px;
      justify-content: flex-start;
      border-bottom: 1px solid #c6c6c6;
      position: relative;

      .button {
        padding: 0 10px;
        cursor: pointer;
        font-size: 16px;

        &.disabled {
          opacity: 0.5;
          cursor: default;
        }
      }

      .input {
        width: 100%;
        outline: none;
        border-radius: 10px;
        height: 20px;
        border: none;
        margin-left: 5px;
        padding: 0 10px;
        background: #f1f3f4;
      }
    }

    .resourcesListMask {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 2;
    }

    .resourcesList {
      position: absolute;
      top: 39px;
      left: 78px;
      z-index: 2;

      .input {
        background: white;
        width: 502px;
        outline: 0;
        border: none;
        height: 22px;
        padding: 0 10px;
        border-top-left-radius: 8px;
        border-top-right-radius: 8px;
      }

      .list {
        width: 522px;
        max-height: 200px;
        overflow: auto;
        background: white;
        border-bottom-left-radius: 8px;
        border-bottom-right-radius: 8px;
      }

      .noResults {
        height: 25px;
        color: #202124;
        display: flex;
        align-items: center;
        padding-left: 10px;
      }

      .resource {
        display: flex;
        align-items: center;
        padding: 0 10px;
        cursor: pointer;
        height: 25px;
        color: #202124;

        &.active {
          background: #DBDBDC;
        }

        &:hover:not(.active) {
          background: #E8E8E9;
        }

        .badge {
          width: 11px;
          height: 11px;
          background: white;
          border-radius: 50%;
          margin-right: 8px;

          &.sound {
            background: @resourceSound;
          }

          &.music {
            background: @resourceMusic;
          }

          &.bitmap {
            background: @resourceBitmap;
          }

          &.polyBuffer {
            background: @resourcePolyBuffer;
          }

          &.palette {
            background: @resourcePalette;
          }

          &.script {
            background: @resourceScript;
          }
        }
      }
    }
  }

  .resourceViewer {
    position: absolute;
    top: 40px;
    left: 5px;
    right: 5px;
    bottom: 5px;

    .noPreviewAvailable {
      font-size: 22px;
      text-transform: uppercase;
      margin-top: 10px;
    }
  }
</style>