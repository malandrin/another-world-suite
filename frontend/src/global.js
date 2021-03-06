export default {
  windowZIndex: 0,

  resources: {
    classByType: ['sound', 'music', 'bitmap', 'palette', 'script', 'polyBuffer'],
    resourceNameByType: ['Sound', 'Music', 'Bitmap', 'Palette', 'Script', 'Poly Buffer', 'Unknown'],
  },

  resourcesIdByPart: [{
    palette: 0x14,
    script: 0x15,
    poly1: 0x16,
    poly2: 0
  }, {
    palette: 0x17,
    script: 0x18,
    poly1: 0x19,
    poly2: 0
  }, {
    palette: 0x1a,
    script: 0x1b,
    poly1: 0x1c,
    poly2: 0x11
  }, {
    palette: 0x1d,
    script: 0x1e,
    poly1: 0x1f,
    poly2: 0x11
  }, {
    palette: 0x20,
    script: 0x21,
    poly1: 0x22,
    poly2: 0x11
  }, {
    palette: 0x23,
    script: 0x24,
    poly1: 0x25,
    poly2: 0
  }, {
    palette: 0x26,
    script: 0x27,
    poly1: 0x28,
    poly2: 0x11
  }],

  freqTable: [
    0x0cff, 0x0dc3, 0x0e91, 0x0f6f, 0x1056, 0x114e, 0x1259, 0x136c,
    0x149f, 0x15d9, 0x1726, 0x1888, 0x19fd, 0x1b86, 0x1d21, 0x1ede,
    0x20ab, 0x229c, 0x24b3, 0x26d7, 0x293f, 0x2bb2, 0x2e4c, 0x3110,
    0x33fb, 0x370d, 0x3a43, 0x3ddf, 0x4157, 0x4538, 0x4998, 0x4dae,
    0x5240, 0x5764, 0x5c9a, 0x61c8, 0x6793, 0x6e19, 0x7485, 0x7bbd
  ]
}
