// 브라우저 DOM의 family 이름과 CSS 인용 문자열을 구분한다.
export async function installCanvasFont(family, bytes, descriptors) {
  const font = new FontFace(family, bytes, descriptors);
  await font.load();
  document.fonts.add(font);
  return font;
}
