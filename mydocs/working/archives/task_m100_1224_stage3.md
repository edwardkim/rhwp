# Stage 3 완료 보고서 — Task #1224: 대체폰트 번들 + 임베딩/웹 반영

- **이슈**: #1224 (M100)
- **브랜치**: `feature/issue-1224-font-fidelity`
- **단계**: Stage 3 / 4
- **작성일**: 2026-06-02

## 1. 핵심 발견 — 임베딩(@font-face)은 `<text>`에 무효

검증 중 두 가지 구조적 사실이 드러나, Stage 3 접근을 수정했다.

1. **rsvg(librsvg)는 base64 `data:` @font-face 폰트를 무시** — fontconfig 만 사용.
2. **rhwp 임베딩 subsetter(typst, 0.2)는 cmap·OS/2 테이블을 제거** (subset 테이블:
   `glyf head hhea hmtx loca maxp name post prep`). PDF 는 GID 로 글자를 그려 cmap 불요지만,
   **브라우저 SVG `<text>`는 cmap 으로 문자→글리프 매핑** → cmap 없는 임베딩 폰트는
   브라우저도 `<text>` 에 쓸 수 없다.

→ **`--embed-fonts` @font-face 는 rsvg·브라우저 어디서도 `<text>` 글리프 교체에 무효**(기존
구조 한계, 본 task 무관). 글리프를 실제 가볍게 하는 **유일 실효 경로 = 폴백 체인 + 렌더러
폰트시스템(fontconfig/웹폰트)에 대체폰트 존재**.

## 2. weight 확정 — ExtraLight (wght 200)

실제 rsvg 파이프라인으로 페이지 본문 밀도 측정(목표 PDF 0.265, 변경전 0.378):

| weight | 페이지 밀도 | 비고 |
|--------|-----------|------|
| 300 Light | 0.302 | 표준, 견고 |
| 250 | 0.286 | 비표준 명명 |
| **200 ExtraLight** | **0.277** | 표준 명명, 목표 최근접, 전체페이지 깨끗(소형 텍스트 단절 없음) |

작업지시자 확정: **ExtraLight 200**.

## 3. 구현

### 네이티브

- **폴백 체인**(`renderer/mod.rs::generic_fallback`): Stage 2 의 `'Noto Sans KR Light'` →
  `'Noto Sans KR ExtraLight'` 로 갱신(무거운 `'Noto Sans KR'` 직전). 테스트 단언 보강.
- **번들 자산**: `ttfs/opensource/NotoSansKR-ExtraLight.ttf`(wght 200, 한글+라틴 서브셋,
  14,205 글리프, 2.75MB) + `NotoSansKR-OFL.txt` + `README.md`. Git 추적(OFL).
- **임베딩 대체**(`svg.rs`): `korean_gothic_substitute()` 추가 — 돋움/고딕/굴림(·dotum/
  gothic/gulim) 계열은 저작권 폰트 부재 시 `NotoSansKR-ExtraLight.ttf` 를 최후 후보로 서브셋.
  `find_font_file` 탐색 경로 말단에 `ttfs/opensource` 추가(실제 폰트 항상 우선). §1 한계로
  현재 `<text>` 실효성은 없으나, 향후 cmap-보존 subsetter 전환 시 자동 활성화.

### 웹 (rhwp-studio)

- `public/fonts/NotoSansKR-ExtraLight.woff2`(592KB) 추가.
- `font-loader.ts FONT_LIST`:
  - `Noto Sans KR ExtraLight` 등록(체인 말단 해석용).
  - `Haansoft Dotum`(샘플 본문 폰트, 기존 미등록) → ExtraLight 매핑.
  - `돋움·돋움체·굴림·새굴림` 을 NotoSansKR-Regular → **ExtraLight** 로 재매핑(네이티브 정합).
- `public/fonts/FONTS.md` 인벤토리 갱신.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 네이티브 bare 렌더(체인+fontconfig ExtraLight) | 본문 밀도 **0.277** ≈ PDF 0.265 (변경전 0.378) |
| 시각 대조 6쪽 문25/26 | `output/poc/task1224/final_before_after_pdf.png` — 변경후가 PDF 정합 |
| 임베딩 로그 | `Haansoft Dotum → 서브셋 13.9KB`(원본 ExtraLight 2.75MB) |
| `cargo test --lib` | **1494 passed, 0 failed** |
| 웹 TS(`tsc --noEmit`) | font-loader.ts 오류 없음(잔여 canvaskit 오류는 사전 의존성 누락, 무관) |
| 구 폰트명 잔존 | 없음 |

## 5. 전달 경계 (정직한 한계)

- **웹(rhwp-studio)**: woff2 + @font-face 자동 적용 → 별도 설치 불요. ✓ 결정적.
- **네이티브 CLI(rsvg 등)**: SVG 의 font-family 체인을 fontconfig 가 해석하므로, 충실도
  적용엔 ExtraLight 가 **fontconfig 에 설치**돼야 함(`cp ttfs/opensource/*.ttf ~/.fonts`).
  미설치 시 회귀 없이 기존 Noto 폴백 유지.
- **임베딩 @font-face**: §1 한계로 현재 `<text>` 무효. subsetter cmap 보존은 별도 이슈 권장.

## 6. 다음 단계

Stage 4: 다문서 표본 시각 검증 + 레이아웃 불변(`dump-pages` 전후 동일) 증빙 +
`tech/font_fallback_strategy.md` 가이드 갱신 + 최종 보고서.
