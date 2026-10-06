# PR #2188 검토 — CanvasKit 첨자 direct replay (seo-rii)

- Refs #536 / base devel / MERGEABLE / CI 전 항목 pass / 작성일: 2026-07-11
- 사이클: **재기여자** — CanvasKit replay 계약 축 전담 이력 8건 머지(#1429~#1806), 본 PR 은
  그 연장(P32 진단 고정 → 검증된 첨자 동작의 direct path 이관).

## 검토

- **정책 diff 범위 정확**: superscript/subscript 의 unsupported 분류만 해제 —
  charOverlap/vertical/decoration/outline·shadow 는 fallback 유지, **첨자+charOverlap
  복합은 정책-가시 유지 테스트 신설**로 회귀 차단. GlyphRun 선택 정책 무변경 명시.
- 구현 논지 타당: 70% 축소를 문자열 단위가 아닌 glyph 단위로 적용해 layout advance
  보존, baseline 은 run-내부 좌표임을 placement.runToPage 로 절대화 — SVG/WebCanvas
  기존 비율과 일치시키는 접근.
- 불완전 positions/glyph mapping 은 진단(textRun:layoutPositions/glyphMapping)으로
  강등 — 이 축의 확립된 계약 스타일.

## 메인테이너 재현 검증

무충돌 병합 / canvaskit_policy 18/18 / fmt ✓ / tsc 무관외 0 /
**renderer-contract e2e PASS** (fresh WASM) / rust 전수 3,041/0 / studio 185/185.

## 판단

**approve + merge.** 시각 판정: CanvasKit 은 비-기본 renderer 이고 공개 기본 경로
무변경 + 계약 e2e/정책 테스트가 게이트 — 선택 적용 거버넌스상 표적 테스트로 갈음.

## 시각 판정 결과 (2026-07-11, 작업지시자 판정 — 통과)

renderer-baseline 하네스 3벌 대조(canvas2d/CK이전/CK이후, hwpx_sample2 p0):
커버리지 확대 실재(이전 전면 fallback → 이후 direct 방출). ■ 글머리 tofu 는 **본 PR
비회귀** — web/fonts NotoSansKR 서브셋의 U+25A0/25AA 누락 + P16 단일 typeface 한계가
가시화된 것. 기호 포함 폰트 스왑 실험으로 정상 렌더 실증(output/poc/pr2188-fontswap).
네이티브 SVG/PNG 는 --font-path 로 무관 확인. 후속: 서브셋 기호 보강 이슈 #2190 등록.
merge 완료.
