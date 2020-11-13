module.exports = {
  publicPath: process.env.NODE_ENV === 'production' ? '/another-world-suite/' : '/',
  presets: [
    '@vue/cli-plugin-babel/preset'
  ]
}
