# Happiness Sans variable exact source

- 공식 배포: <https://thehyundaifont.com/>
- archive: `HappinessSans-Screen.zip`, 6,354,097 bytes,
  `1defb001694a8bd3fd5c7a15c45377e82e654569177c6995016ae321223fb041`
- exact member: `screen/otf/HappinessSansVF.ttf`, 1,503,064 bytes,
  `3bbd254dcc5780f7524f9d07af4aa981ba5e3e84cf32d7d4e04301b3943e8694`
- 동봉 라이선스: `screen/HapinessSans_License.pdf`,
  `f5bd344131ee034f3425517ea376ab6acea9126310e7b2cd74cd18673991b055`

공식 archive member를 byte 그대로 추적한다. 기존 `assets/fonts/HappinessSansVF.woff2`를 변환하거나 font
table·이름·outline을 수정하지 않는다. 공식 TTF와 WOFF2의 table 집합, glyph order, 3,889개 default outline,
GPOS·GSUB·fvar·gvar가 일치하며 outline canonical digest는
`8c213a9e959b0305bf6201dedbea39c1cfd3545cff817ec0b00fa8ca4e26b5b5`다.

이 파일은 W10 variation shaping fixture의 exact source다. 제품 WASM·Studio font bundle에는 자동 포함하지
않으며 폰트 자체를 판매하거나 파생 폰트를 만들지 않는다.
