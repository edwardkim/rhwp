// CanvasKit의 이미지 생성·반복 shader API를 기록하는 계약 검사용 모형이다.
export function makePatternImageReplayApi(events) {
  return {
    ColorType: { RGBA_8888: 'RGBA_8888' },
    AlphaType: { Unpremul: 'Unpremul' },
    ColorSpace: { SRGB: 'SRGB' },
    TileMode: { Repeat: 'Repeat' },
    FilterMode: { Linear: 'Linear' },
    MipmapMode: { None: 'None' },
    MakeImage(info, bytes, rowBytes) {
      events.push({ type: 'image.create', info, bytes: [...bytes], rowBytes });
      return {
        makeShaderOptions(tileX, tileY, filter, mipmap, matrix) {
          events.push({ type: 'shader.pattern', tileX, tileY, filter, mipmap, matrix: [...matrix] });
          return { delete() { events.push({ type: 'shader.delete' }); } };
        },
        delete() { events.push({ type: 'image.delete' }); },
      };
    },
  };
}
