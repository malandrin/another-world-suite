module.exports = {
  publicPath: process.env.NODE_ENV === 'production' ? '/another-world-suite/' : '/',
  chainWebpack: config => {
    config.module
      .rule('wasm')
      .test(/\.bin$/)
      .use('arraybuffer-loader')
        .loader('arraybuffer-loader')
        .end()
  }
}