<template>
  <div
    class="soundViewer"
  >
    <canvas
      class="canvas"
      ref="canvas"
    />
    <div class="playingLine"
      v-bind:style="{left: playingLineLeft}"
    />
    <div class="keys">
      <div
        class="key"
        v-for="freq in frequencies"
        v-bind:key="freq"
        v-bind:class="{pressed: tonePlayer && playingFreq === freq}"
        v-on:mousedown="playSound(freq)"
        v-on:mouseup="stopSound()"
      >
        <div>
          {{freq}}
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import * as Tone from 'tone'
import Global from '@/global'

const LINE_LEFT_POSITION = 6

export default {
  name: 'SoundViewer',
  data: function() {
    return {
      playingLineLeftValue: 0,
      playingFreq: 0,
      tonePlayer: null
    }
  },
  mounted: function() {
    this.$refs.canvas.width = 732
    this.$refs.canvas.height = 300
    this.canvasContext = this.$refs.canvas.getContext('2d')
  },
  computed: {
    frequencies: function() {
      return Global.freqTable
    },
    playingLineLeft: function() {
      return `${this.playingLineLeftValue}px`
    }
  },
  methods: {
    playSound: function(freq) {
      this.stopSound()

      this.tonePlayer = new Tone.Player(this.audioBuffer)
      this.tonePlayer.playbackRate = freq / 11024
      this.tonePlayer.volume.value = -20 // TODO: create a slide to select the volume
      this.tonePlayer.toDestination()
      this.tonePlayer.start()

      this.playingLineLeftValue = LINE_LEFT_POSITION
      this.playLen = this.audioBuffer.length / freq
      this.playingTimer = 0
      this.playingFreq = freq
      this.deltaTimer = Date.now()

      this.updatePlayingInfo()
    },
    stopSound: function() {
      if (this.tonePlayer) {
        this.tonePlayer.stop()
        this.tonePlayer = null
      }

      this.playingLineLeftValue = LINE_LEFT_POSITION
    },
    updatePlayingInfo: function() {
      if (!this.tonePlayer) {
        return
      }

      const current = Date.now()
      const delta = (current - this.deltaTimer) / 1000

      this.deltaTimer = current
      this.playingTimer += delta

      if (this.playingTimer >= this.playLen) {
        this.playingLineLeftValue = LINE_LEFT_POSITION
      } else {
        this.playingLineLeftValue = LINE_LEFT_POSITION + (this.playingTimer / this.playLen) * (this.$refs.canvas.width - LINE_LEFT_POSITION)
        setTimeout(this.updatePlayingInfo, 16)
      }
    },
    setAudioBuffer: function(audioBuffer) {
      this.audioBuffer = audioBuffer
      this.playingLineLeftValue = LINE_LEFT_POSITION

      // render sound wave
      const rawData = audioBuffer.getChannelData(0)
      const blockSize = Math.max(Math.floor(rawData.length / this.$refs.canvas.width), 1)
      const halfHeight = this.$refs.canvas.height / 2

      this.canvasContext.fillStyle = '#9C9C9C'
      this.canvasContext.clearRect(0, 0, this.$refs.canvas.width, this.$refs.canvas.height)

      for (let x = 0; x < this.$refs.canvas.width; ++x) {
        let blockTotal = 0

        for (let b = 0; b < blockSize; ++b) {
          blockTotal += rawData[x * blockSize + b]
        }

        const normValue = Math.abs(blockTotal / blockSize) * halfHeight
        this.canvasContext.fillRect(x, halfHeight - normValue, 1, normValue * 2)
      }
    }
  }
}
</script>

<style lang="less" scoped>
  .soundViewer {
    height: 100%;

    .canvas {
      width: 732px;
      height: 300px;
      border: 1px solid white;
      background: #1C2F48;
    }

    .playingLine {
      position:absolute;
      top: 1px;
      left: 0;
      height: 299px;
      background: white;
      width: 1px;
    }

    .keys {
      display: flex;
      justify-content: center;

      .key {
        width: 2.2%;
        height: 90px;
        word-break: break-all;
        cursor: pointer;
        border: 1px solid grey;
        border-bottom-left-radius: 4px;
        border-bottom-right-radius: 4px;
        background: white;
        box-shadow: 0px 2px 0 0 rgba(0, 0, 0, 0.2);
        position: relative;

        div {
          color: black;
          position: absolute;
          top: 5px;
          transition: top 30ms linear;
        }

        &.pressed {
          height: 95px;
          box-shadow: none;
          transition: height 30ms linear, box-shadow 30ms linear;
          background: yellow;

          div {
            top: 10px;
            transition: top 30ms linear;
          }
        }
      }
    }
  }
</style>