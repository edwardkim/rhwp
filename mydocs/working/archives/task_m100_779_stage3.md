# Task #779 Stage 3 — 회귀 검증 + 시각 판정

## 결정적 검증

| 검증 | 결과 |
|------|------|
| `tsc --noEmit` | clean |
| `npm run build` | 성공 (WASM 4.6 MB, index.js 707 KB) |
| `cargo test --lib --release` | 1217 passed (rust lib 무영향) |
| `cargo clippy --release --lib` | 신규 경고 0 |

## 시각 판정 (작업지시자) — ★ 통과

작업지시자 직접 dev 서버 시각 판정: **"해결 완료"**.

### 시나리오 검증

1. **본 결함 해소** ✅
   - 다중 페이지 문서 (예: hwp3-sample10.hwp 763 페이지) 로드
   - 텍스트 클릭 (caret p.1) → 마우스 보유 상태 로 scrollbar 까지 drag → release
   - 결과: scrollbar 위치 보존 (이전 페이지 자동 복귀 부재)

2. **cursor click 정상** ✅ (회귀 부재)
3. **키보드 navigation 정상** ✅ (회귀 부재)
4. **드래그 selection autoscroll (PR #718) 정상** ✅ (회귀 부재)
5. **Wheel scroll 정상** ✅ (회귀 부재)

## 영역 좁힘 본질 정합

`updateCaret(skipScroll: boolean = false)` 시그니처 확장 + opt-in skip 영역 으로 30+ 기존 호출 영역 무영향. `onMouseUp` 의 단일 호출 영역 만 `true` 변경.

PR #718 (Task #661) 의 `updateTextSelectionDragAutoScroll` 영역 (별도 path) 보존 영역 — 회귀 부재 정합.

## Stage 4 진행 영역

- 최종 결과 보고서 (`mydocs/report/task_m100_779_report.md`)
- orders/20260510.md 본인 영역 등록 (closes #779)
- PR 생성
