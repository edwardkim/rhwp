# ttfs/opensource — 오픈소스 대체 폰트 (Git 추적)

`ttfs/` 루트·하위(`hwp/`, `windows/`)는 한컴/MS 저작권 폰트용으로 `.gitignore` 처리되나,
이 디렉터리는 **재배포 가능한 오픈소스(OFL) 폰트**만 담아 Git 으로 추적한다.

## 수록 폰트

| 파일 | 폰트 | 라이선스 | 용도 |
|------|------|---------|------|
| `NotoSansKR-Regular.ttf` | Noto Sans KR Regular (wght 400, 한글+라틴+글머리/도형 subset) | SIL OFL 1.1 (`NotoSansKR-OFL.txt`) | Task #2190 — CanvasKit 기본 typeface의 KS 기호/Box Drawing coverage |
| `NotoSansKR-ExtraLight.ttf` | Noto Sans KR ExtraLight (wght 200, 한글+라틴 서브셋) | SIL OFL 1.1 (`NotoSansKR-OFL.txt`) | Task #1224 — 한컴 돋움(Haansoft Dotum)·돋움·굴림 계열의 **획 두께 정합** 대체 |

`NotoSansKR-Regular.ttf`와 대응 WOFF2는 Google Fonts `ofl/notosanskr` variable source를
`wght=400`으로 instance화한 뒤 `tools/subset_noto_sans_kr_regular.py`로 생성한다. 기존
Regular cmap에 `U+2500-257F`, `U+25A0-25FF`를 더해 CanvasKit에서 `■`·`▪` 글머리와
표 테두리 문자가 tofu로 보이지 않게 한다.

- 출처: Google Fonts `ofl/notosanskr` 가변폰트에서 `wght=200` 인스턴스화 후 한글+라틴 서브셋.
- 한컴 돋움 획 두께(페이지 밀도 0.265)에 근접(rsvg 페이지 밀도 0.277). 기존 폴백
  Noto Sans CJK KR Regular(0.378)는 +43% 두꺼워 PDF 대비 본문이 과도하게 굵게 보였다.
- **독립 family 명명**: family/typographic family 모두 `"Noto Sans KR ExtraLight"`, weight
  class 400. wght=200 글리프를 갖되 **"Noto Sans KR" 의 weight-variant 로 등록하지 않는다.**
  (typographic family 를 "Noto Sans KR" 로 두면 fontconfig 가 일반 `"Noto Sans KR"` 요청까지
  ExtraLight 로 가로채 시스템 전역에 부작용 — Task #1224 Stage 4 에서 교정.)

## 사용 경로

1. **SVG 폴백 체인**(`renderer/mod.rs::generic_fallback`): sans 체인에 `'Noto Sans KR
   ExtraLight'` 가 무거운 `'Noto Sans KR'` 직전에 위치. rsvg/브라우저가 이 폰트를 **폰트
   시스템에서 찾을 수 있을 때** 가볍게 렌더.
2. **임베딩**(`svg.rs::find_font_file` → `--embed-fonts`): 고딕 계열 문서폰트의 저작권
   파일 부재 시 이 폰트를 최후 후보로 서브셋. **단, 현 subsetter(typst)는 cmap 을
   제거하므로 브라우저 `<text>` 매핑엔 무효** — 실효 경로는 ① 폴백 체인이다.

## 네이티브 CLI(rsvg) 충실도 활성화

`rhwp export-svg` 의 SVG 를 rsvg 등으로 래스터할 때 ExtraLight 가 적용되려면 이 폰트가
**fontconfig 에 설치**되어 있어야 한다(외부 래스터라이저는 SVG 의 font-family 체인을
fontconfig 로 해석):

```sh
cp ttfs/opensource/NotoSansKR-ExtraLight.ttf ~/.fonts/ && fc-cache -f
```

웹 canonical source는 `assets/fonts/NotoSansKR-ExtraLight.woff2`다. rhwp-studio build는
`rhwp-studio/public/fonts` 링크를 통해 이를 runtime `fonts/`에 포함하고, `font-loader.ts`의
`@font-face` 등록으로 자동 적용한다(별도 설치 불요).
