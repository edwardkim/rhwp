// 브라우저 DOM의 family 이름과 CSS 인용 문자열을 구분한다.
export async function installCanvasFont(family, bytes, descriptors, sourceOwner = null) {
  const font = new FontFace(family, bytes, descriptors);
  await font.load();
  document.fonts.add(font);
  // 브라우저와 같은 프로그램만 paint 세션에 전달한다.
  if (typeof sourceOwner?.registerHftOutlineFont === 'function'
      && String(descriptors.weight || '400') === '400'
      && (descriptors.style || 'normal') === 'normal') {
    sourceOwner.registerHftOutlineFont(family, new Uint8Array(bytes), 0);
  }
  return font;
}
