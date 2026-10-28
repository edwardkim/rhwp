# PR #2372 검토 — direct PageLayerTree PDF export (seo-rii 4번째)

- PR: https://github.com/edwardkim/rhwp/pull/2372 — Refs #536, draft → ready 전환
- 규모: 15파일 +1,847/−169 (pdf.rs +785, skia/renderer.rs +200, 워크플로 2종)

## 변경 본질

P23의 PDF는 SVG→svg2pdf 호환 경로였다. 이번 PR은 `native-skia`에서
`PageLayerTree`를 Skia PDF canvas에 직접 기록하는 **opt-in backend**
(`export-pdf --backend direct`)를 추가 — PageLayerTree frontend/backend 경계가
PDF에서도 replay 가능한지 실증하는 단계.

- **기본 경로 불변의 구조적 확인**: `PdfBackend::default()`=CompatibilitySvg,
  rendering.rs 삭제 0줄(순수 추가 API), src 변경이 pdf.rs·skia/*(feature)·
  main.rs(CLI)에 국한 — 기본 SVG/Canvas·browser 경로 코드 무접촉.
- **조용한 근사 없음**: 현 native replay가 시각 손실하는 op(gradient/pattern/
  shadow shape, multi-line/arrow, 미적용 image adjustment)는 기록 전 거부 후
  `--backend svg` 안내. 이미지 decode·Raw SVG fallback 실패도 placeholder 없이
  export 실패 (strict 모드).
- native PNG와 direct PDF의 replay 공용화(`render_page_to_canvas_strict`) +
  90/270 이미지 effective bbox 를 SVG/Canvas 계약과 정합(단위 테스트 2건).
- render-diff preflight fast-pass 신원 검증 강화(브랜치·repo·생성시각·identity
  step 대조 — 다른 PR/오래된 run 재사용 차단) + direct PDF 게이트 CI 편입
  (selected corpus, compatibility PDF 오라클, 144dpi, 문턱 0.02).

## 시각 검증 거버넌스 적용 판단 (선택 적용 원칙)

기본 시각 출력을 바꾸지 않는 opt-in 백엔드 신설이므로 한컴 정답지 OVL-step
발동 대상 아님. 신경로의 오라클은 PR 설계대로 기존 compatibility PDF —
CI 게이트(biz_plan 0.0119 / kps-ai 0.0077 / tac-case-001 0.0039, 문턱 0.02)로
고정. 90/270 bbox 정합은 feature-gated native 백엔드의 parity 수정이며
render-diff 게이트가 커버.

## 로컬 재실증 (devel merge 충돌 0, merged tree)

| 게이트 | 결과 |
|--------|------|
| fmt / clippy(-D warnings, native-skia lib+bin+테스트 2종) | 통과 |
| skia --lib 56 / render_p37_direct 4 / issue_2225 2 / p23 contract 5 / p37 CLI 2 | 전건 통과 |
| `cargo test --tests` 전체(release-test) | 실패 0 |
| node --check e2e 2종 / MANIFEST 검사 | 통과 (e2e 신규 파일 없음) |
| direct vs svg 스모크 (biz_plan 6쪽, 144dpi) | 페이지 수·크기 일치, 레이아웃 동일. AE(fuzz 5%) 수치 차는 텍스트 굵기/AA 성격(svg2pdf 아웃라인 vs Skia 실폰트) — 구조 편차 아님 |
| CI | 전 항목 green (Render Diff direct 게이트·Native Skia 신테스트 포함) |

## 판단

**merge 권고.** 하지 않는 것(기본 전환·자동 fallback·WASM·glyph parity)을
명시한 경계 규율, strict 실패 정책, oracle 게이트 동봉이 모범적.

## 처리 결과 (2026-07-18)

merge 완료(admin) + 메인테이너 코멘트 게시. #2387 은 close-issues 워크플로가
#2388 merge 직후 자동 close (devel 포함 검증은 앞서 완료). seo-rii 4번째 기여 완결.
