# 최종 결과 보고서 — Task #1224: 폰트 충실도 (한컴 돋움 폴백 글리프 과대·과굵)

- **이슈**: #1224 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1224-font-fidelity` (base: `stream/devel` 4f5a8e22)
- **기간**: 2026-06-01 ~ 2026-06-02
- **성격**: 렌더 시각 충실도 — 폰트 폴백/대체 (레이아웃·메트릭 무변경)

## 1. 문제와 재규명

이슈 진단: 한컴 돋움 미설치 시 Noto Sans CJK KR 로 폴백되어 글리프가 PDF보다 "크고
두껍게"(em-fill 0.87 vs 0.67) 보임.

**실측 재규명(Stage 1)**: 동일 페이지 픽셀폭(1240px)으로 고해상도 측정 결과 —

| | 잉크 높이 | 획 밀도 |
|--|--|--|
| rhwp (Noto Sans CJK KR Regular) | 17px | 0.378 |
| PDF (한컴 돋움) | 17px | 0.265 |

- **잉크 높이 동일** → 크기/em-fill 차이는 없음(이슈의 0.67 은 em=15px 저해상도 오차).
- **진짜 원인 = 획 두께**: Noto Regular 가 한컴 돋움보다 +43% 두껍다. "두껍게" 진단은 정확.

## 2. 핵심 설계 인사이트

SVG 텍스트는 메트릭-DB advance 위치에 클러스터별 `<text>`를 방출하고 `textLength`로
가로폭까지 제약 → **위치·자간·가로폭은 모두 제어**되고 **유일 미제어 차원은 글리프 획
두께(weight)** 로, 이는 폴백 폰트가 결정. 따라서 weight 가 한컴 돋움에 근접한 대체폰트로
바꾸면 **레이아웃·메트릭 무변경으로 시각이 교정**된다.

## 3. 해결 — Noto Sans KR ExtraLight (wght 200)

오픈소스 한글 고딕은 모두 0.83~0.89 의 큰 글자면이라 크기는 무관, **weight 만 조절**.
rsvg 페이지 밀도(목표 0.265): Regular 0.378 → Light(300) 0.302 → **ExtraLight(200) 0.277** 채택.
동일 Noto 계열(중립 고딕)·OFL·한글 11,172 완비.

## 4. 구현 (4단계)

| 단계 | 내용 |
|------|------|
| Stage 1 | 후보 11종 4방식 실측 → 원인=획 두께 규명, ExtraLight 선정 (`tech/font_fidelity_measurement_1224.md`) |
| Stage 2 | `generic_fallback` sans 체인에 ExtraLight 를 무거운 Noto 직전 삽입 |
| Stage 3 | OFL 번들(`ttfs/opensource/`, `web/fonts/`) + `korean_gothic_substitute` 임베딩 후보 + 웹 `font-loader.ts`(Haansoft Dotum·돋움·굴림 → ExtraLight) |
| Stage 4 | 레이아웃 불변 증빙 + 다문서 검증 + 폰트 family 정합성 수정 + 가이드 §11 |

### 변경 파일

- `src/renderer/mod.rs` — generic_fallback 체인 + 테스트
- `src/renderer/svg.rs` — `korean_gothic_substitute()`, `find_font_file` 탐색 경로
- `ttfs/opensource/NotoSansKR-ExtraLight.ttf` + OFL + README (네이티브 번들)
- `web/fonts/NotoSansKR-ExtraLight.woff2` + FONTS.md, `rhwp-studio/src/core/font-loader.ts` (웹)
- `mydocs/tech/font_fallback_strategy.md` §11, `font_fidelity_measurement_1224.md`

## 5. 검증

| 항목 | 결과 |
|------|------|
| 본문 충실도(3-09 문26) | 0.378 → **0.277 ≈ PDF 0.265** |
| 다문서(3-10·3-11) | cur < base 일관, 3-11 PDF 정합 |
| 레이아웃 불변 | dump-pages 3문서 **바이트 동일** |
| 회귀 | `cargo test --lib` **1494 passed, 0 failed** |
| 웹 TS | font-loader.ts 오류 없음 |
| fontconfig 분리 | `"Noto Sans KR"` 섀도잉 해소(독립 family) |

시각 자료: `output/poc/task1224/{final_before_after_pdf,weight_compare,multidoc_compare}.png`

## 6. 주요 발견 (이슈 진단 보정)

1. **원인은 크기가 아니라 획 두께(weight)** — em-fill 동일, Noto Regular 가 +43% 두꺼움.
2. **`--embed-fonts` @font-face 는 `<text>` 에 무효** — typst `subsetter` 가 cmap 을 제거해
   브라우저가 문자→글리프 매핑 불가, librsvg 는 data 폰트 자체를 무시. 실효 경로는 **폴백
   체인 + 폰트시스템(fontconfig/웹폰트) 설치**. (cmap 보존 서브셋은 별도 이슈 권장.)
3. **대체폰트 family 명명 주의** — typographic family 를 "Noto Sans KR" 로 두면 fontconfig 가
   일반 요청까지 가로챔 → 독립 family 명명 필수.

## 7. 전달 경계 (정직한 한계)

- **웹(rhwp-studio)**: woff2 + @font-face 자동 적용, 설치 불요 — 결정적.
- **네이티브 CLI(rsvg 등)**: ExtraLight 가 fontconfig 에 설치돼야 적용
  (`cp ttfs/opensource/*.ttf ~/.fonts && fc-cache`). 미설치 시 회귀 없이 기존 폴백 유지.
- **임베딩**: cmap 한계로 현재 `<text>` 무효(번들·wiring 은 유지, 향후 자동 활성).

## 8. 후속 권장 (범위 밖)

- 임베딩 subsetter 를 cmap 보존 방식으로 전환 → `--embed-fonts` SVG 자기완결화 (별도 이슈).
- serif(바탕/명조) 계열 weight 충실도 점검(본 task 는 고딕 한정).
- CI 시각 비교 파이프라인에 `ttfs/opensource` 폰트 설치 단계 추가 검토.
