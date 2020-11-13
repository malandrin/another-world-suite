<template>
  <div
    id="app"
  >
    <div v-if="!engine" class="loadGameDataSection">
      <div v-if="creatingEngine" class="creatingEngine">
        Initializing...
      </div>
      <div v-else class="droppableArea" v-on:drop.prevent="onFileDropped" v-on:dragover.prevent>
        <div class="droppableMessage">
          <span>Drag & drop the Another World game zip file here</span>
          <span style="margin-left: 10px; margin-right: 10px; color: white;">or</span>
          <a class="uploadLink" v-on:click="$refs.fileUploader.click()">Upload it from your computer</a>
          <div class="downloadInfo">(You can download it from <a href="https://archive.org/download/another_world_pc/another_world_pc.zip">archive.org</a>)</div>
          <input type="file" ref="fileUploader" style="display: none;" v-on:change="onFileUploaded">
        </div>
        <div class="credits">
          <div class="me">
            <div>Made with <i class="fas fa-heart" style="color: red"/> by <a href="https://www.cesarbotana.com" target="_blank">César Botana</a></div>
            <div>Source code available at <a href="https://github.com/malandrin/another-world-suite" target="_blank">GitHub</a></div>
          </div>
          <div class="giants">
            <div>This project has been possible by standing on the shoulders of giants:</div>
            <div>
              <a href="http://www.anotherworld.fr/anotherworld_uk/another_world.htm" target="_blank">Eric Chahi</a>
              <span>, </span>
              <a href="https://fabiensanglard.net/anotherWorld_code_review/index.php" target="_blank">Fabien Sanglard</a>
              <span> & </span>
              <a href="https://github.com/cyxx/rawgl/blob/master/docs/Amiga_DOS.md" target="_blank">Gregory</a>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="windows">
      <Disassembler
        ref="disassembler"
        v-bind:engine="engine"
        v-bind:vmPaused="vmPaused"
        v-on:step="vmStep"
        v-on:continue="vmContinue"
        v-on:continue-until-next-frame="vmContinueUntilNextFrame"
        v-on:pause="vmPause"
        v-on:restart="vmRestart"
        v-on:goto-resource-file="gotoResourceFile"
      />
      <Threads
        ref="threads"
        v-bind:engine="engine"
        v-bind:vmPaused="vmPaused"
        v-on:scroll-to-address="scrollToAddress"
      />
      <Registers
        ref="registers"
        v-bind:engine="engine"
        v-bind:vmPaused="vmPaused"
      />
      <Resources
        ref="resources"
        v-bind:resources="resources"
        v-bind:engine="engine"
      />
      <Game
        ref="game"
        v-bind:engine="engine"
        v-bind:vmPaused="vmPaused"
      />
      <Help
        ref="help"
      />
    </div>
    <div class="titleBar">
      Another World Suite
    </div>
    <div class="windowsBar">
      <div
        v-for="(window, index) in windows"
        v-bind:key="`window_${index}`"
        v-bind:class="{minimized: window.minimized}"
        v-on:click="toggleWindowState(window)"
        class="windowsBarObject"
      >
        {{window.title}}
      </div>
    </div>
  </div>
</template>

<script>
import _ from 'lodash'
import {AnotherWorldEngine} from './another_world_engine'
import jsZip from 'jszip'
import Disassembler from './windows/disassembler'
import Threads from './windows/threads'
import Registers from './windows/registers'
import Resources from './windows/resources'
import Game from './windows/game'
import Help from './windows/help'
import Global from '@/global'

const NUM_GAME_DATA_FILES = 14

const BLIT_ACTION_REQUEST        = 2
const LOAD_PART_ACTION_REQUEST   = 3
const PLAY_SOUND_ACTION_REQUEST  = 4

export default {
  name: 'App',
  components: {
    Disassembler,
    Threads,
    Registers,
    Resources,
    Game,
    Help
  },
  data: function() {
    return {
      engine: null,
      vmPaused: true,
      resources: [],
      windows: [],
      creatingEngine: false
    }
  },
  destroyed: function() {
    if (this.animFrameId) {
      window.cancelAnimationFrame(this.animFrameId)
    }

    if (this.engine) {
      this.engine.end()
    }
  },
  methods: {
    createEngine: async function(zipFile) {
      // TODO: check all required files exist in the zip file
      const content = await jsZip.loadAsync(zipFile)
      let filesInfo = {}

      this.creatingEngine = true

      for (const k in content.files) {
        const file = content.files[k]
        const lname = file.name.toLowerCase()
        let findex = -1

        if (lname.indexOf('memlist.bin') !== -1) {
          findex = 0
        }
        else if (lname.indexOf('bank0') !== -1) {
          findex = parseInt(lname.substr(lname.indexOf('bank0') + 5, 1), 16)
        }

        if (findex !== -1) {
          const fileData = await file.async('uint8array')
          filesInfo[findex] = fileData

          if (Object.keys(filesInfo).length === NUM_GAME_DATA_FILES) {
            // build all the gamedata in one single array
            let gameDataLen = NUM_GAME_DATA_FILES * 4 // = size(all_files) + NUM_GAME_DATA_FILES * 4 (bytes to indicate the lenght of the file)

            for (let i = 0; i < NUM_GAME_DATA_FILES; ++i) {
              gameDataLen += filesInfo[i].length
            }

            let gameData = new Uint8Array(gameDataLen)
            let arrayIdx = 0

            for (let i = 0; i < NUM_GAME_DATA_FILES; ++i) {
              const flen = filesInfo[i].length
              gameData[arrayIdx++] = flen & 0xff
              gameData[arrayIdx++] = (flen >> 8) & 0xff
              gameData[arrayIdx++] = (flen >> 16) & 0xff
              gameData[arrayIdx++] = (flen >> 24) & 0xff
              gameData.set(filesInfo[i], arrayIdx)
              arrayIdx += flen
            }

            this.initAudio();

            const engine = new AnotherWorldEngine()
            await engine.init(gameData)

            this.animFrameId = window.requestAnimationFrame(this.tick)
            this.engine = engine
            this.resources = this.engine.getResourcesInfo()
            this.activeScriptFileId = this.engine.getActiveScriptFileId()
            this.creatingEngine = false
            this.lastTimeStamp = Date.now()

            const self = this
            _.defer(function() {
              self.refreshWindows()

              for (const key in self.$refs) {
                if (self.$refs[key]) {
                  self.windows.push(self.$refs[key].$children[0])
                }
              }
            })
          }
        }
      }
    },
    initAudio: function() {
      const audioContext = window.AudioContext || window.webkitAudioContext
      Global.audioContext = new audioContext()
    },
    tick: async function() {
      if (!this.vmPaused) {
        for (let i = 0; i < 64 && !this.vmPaused; ++i) {
          await this.vmStep()
        }
      }

      this.animFrameId = window.requestAnimationFrame(this.tick)
    },
    vmStep: async function() {
      const actionRequested = this.engine.vmStep()

      if (actionRequested !== 0)  {
        const action = actionRequested >> 24

        if (action === BLIT_ACTION_REQUEST) {
          this.$refs.game.render(this.engine.getFrameBuffer())

          if (!this.vmPaused) {
            if (this.vmPauseInNextBlit) {
              this.vmPause()
              return
            }

            const actionParam = (actionRequested >> 16) & 0xff
            const delay = (Date.now() - this.lastTimeStamp) / 1000
            const timeToSleep = actionParam - delay

            if (timeToSleep > 0) {
              await this.sleep(timeToSleep)
            }

            this.lastTimeStamp = Date.now()
          }
        } else if (action === LOAD_PART_ACTION_REQUEST) {
          const self = this

          _.defer(function() {
            self.activeScriptFileId = self.engine.getActiveScriptFileId()
          })
        } else if (action === PLAY_SOUND_ACTION_REQUEST) {
          // TODO: to implement
        }
      } else if (!this.vmPaused) {
        if (this.$refs.disassembler.hasBreakpoint(this.engine.vmGetCurrentPC())) {
          this.vmPause()
        }
      } else {
        this.refreshWindows()
      }
    },
    vmContinue: function() {
      this.vmPaused = false
      this.lastTimeStamp = Date.now()
      this.refreshWindows()
    },
    vmContinueUntilNextFrame: function() {
      this.vmPaused = false
      this.vmPauseInNextBlit = true
      this.refreshWindows()
    },
    vmPause: function() {
      this.vmPaused = true
      this.vmPauseInNextBlit = false
      this.refreshWindows()
    },
    vmRestart: function(part) {
      this.engine.vmRestart(part)
      this.activeScriptFileId = this.engine.getActiveScriptFileId()
      this.refreshWindows()
    },
    toggleWindowState: function(window) {
      window.minimize(!window.minimized)
    },
    gotoResourceFile: function(resourceId, param) {
      this.$refs.resources.selectResource(resourceId, param)
    },
    scrollToAddress: function(addr) {
      this.$refs.disassembler.scrollTo(addr)
    },
    refreshWindows: function() {
      const threadsInfo = this.engine.getThreadsInfo()

      threadsInfo.paused = this.vmPaused
      this.$refs.disassembler.refresh(threadsInfo)
      this.$refs.disassembler.setScript(this.resources[this.activeScriptFileId].content, this.activeScriptFileId)
      this.$refs.threads.refresh(threadsInfo)
      this.$refs.registers.refresh()
    },
    sleep: function(ms) {
      return new Promise(resolve => setTimeout(resolve, ms))
    },
    onFileDropped: async function(event) {
      for (const f of event.dataTransfer.files) {
        if (f.type === 'application/x-zip-compressed') {
          this.createEngine(f)
          break
        }
      }
    },
    onFileUploaded: async function(event) {
      for (const f of event.target.files) {
        if (f.type === 'application/x-zip-compressed') {
          this.createEngine(f)
          break
        }
      }
    }
  }
}
</script>

<style lang="less" scoped>
#app {
  font-family: Roboto, Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;

  .loadGameDataSection {
    display: flex;
    align-items: center;
    justify-content: center;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 40px;
    background: #3A6EA5;

    .droppableArea {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      display: flex;
      align-items: center;
      justify-content: center;

      .droppableMessage {
        padding: 50px;
        background: #383838;
        color: #d4d4d4;
        border-radius: 5px;

        .uploadLink {
          cursor: pointer;
          text-decoration: underline;
          color: #47C0C0;
        }

        .downloadInfo {
          margin-top: 20px;
          font-size: 14px;

          a {
            color: #47C0C0;
          }
        }
      }

      .credits {
        color: #d4d4d4;
        font-size: 18px;

        a {
          color: #47C0C0;
        }

        .me {
          position: fixed;
          left: 30px;
          bottom: 75px;
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          line-height: 23px;
        }

        .giants {
          position: fixed;
          right: 30px;
          bottom: 75px;
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          line-height: 23px;
        }
      }
    }

    .creatingEngine {
      background: #383838;
      color: #d4d4d4;
      padding: 10px 100px;
      border-radius: 15px;
    }
  }

  .windows {
    position: fixed;
    top: 40px;
    bottom: 40px;
    left: 0;
    right: 0;
    background: #3A6EA5;
    overflow: auto;
  }

  .titleBar {
    position: fixed;
    top: 0;
    height: 40px;
    left: 0;
    right: 0;
    background: #C0C0C0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 26px;
    font-weight: 700;
    border-bottom: 1px solid #F4F1EC;
  }

  .windowsBar {
    position: fixed;
    bottom: 0;
    height: 40px;
    left: 0;
    right: 0;
    background: #C0C0C0;
    display: flex;
    align-items: center;
    padding: 0 10px;
    border-top: 1px solid #F4F1EC;

    .windowsBarObject {
      padding: 5px 10px;
      border-top: 1px solid black;
      border-left: 1px solid black;
      border-bottom: 1px solid white;
      border-right: 1px solid white;
      background: #E7E5E0;
      cursor: pointer;

      &.minimized {
        border-top: 1px solid white;
        border-left: 1px solid white;
        border-bottom: 1px solid black;
        border-right: 1px solid black;
        background: #C0C0C0;
      }

      &:not(:last-child) {
        margin-right: 5px;
      }
    }
  }
}
</style>
