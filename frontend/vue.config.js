module.exports = {
  chainWebpack: config => {
    config.module
      .rule('wasm')
      .test(/\.bin$/)
      .use('arraybuffer-loader')
        .loader('arraybuffer-loader')
        .end()
  }
}