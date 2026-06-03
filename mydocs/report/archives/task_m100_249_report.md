# Task #249: 최종 결과보고서 — Visual Diff 기반 렌더링 호환성 개선

> 최종 보고서 | 2026-04-22
> Issue: [#249](https://github.com/edwardkim/rhwp/issues/249)

---

## 1. 작업 요약

Visual Diff 파이프라인을 구축하여 한컴 렌더링과 페이지별 비교 검증 체계를 확립하고, 이를 통해 발견한 렌더링 불일치 3건을 수정했다.

## 2. 구현 결과

### 단계 1: PUA 심볼 문자 렌더링 ✅ 완료

- `map_pua_bullet_char()` 함수로 U+F000~F0FF 문자를 유니코드 표준 문자로 변환
- SVG, Canvas, HTML 세 렌더러에 일관 적용
- Wingdings 체크마크, 화살표, 도형 심볼이 정상 표시됨

### 단계 2: 문단 border_fill margin 반영 ✅ 완료

- `paragraph_layout.rs`에서 border_fill rect 계산 시 `margin_left`/`margin_right` 반영
- 테두리 박스 위치·폭이 텍스트 영역과 정확히 일치

### 단계 3: 표 외곽 테두리 fallback + clip_rect 개선 ✅ 완료

- `table_layout.rs`: `border_fill_id` fallback 로직 추가, 셀 커버 영역 제외 처리
- `layout.rs`: clip_rect를 콘텐츠 레이아웃 후 확정하여 표 외곽 테두리 클리핑 방지

## 3. 검증 결과

| 항목 | 결과 |
|------|------|
| cargo test | 793개 전체 통과, 0 실패 |
| Visual Diff (PUA 페이지) | □ 두부 문자 → 정상 심볼 |
| Visual Diff (문단 테두리) | 테두리 박스 텍스트 영역 일치 |
| Visual Diff (표 외곽 테두리) | 한컴 렌더링과 일치 |
| Regression (p1, p5, p6, p10) | 없음 |

## 4. 부록: Visual Diff 파이프라인

한컴 PDF를 페이지별 PNG로 캡처(Ground Truth)하고, rhwp SVG를 resvg로 PNG 변환한 후 pixelmatch + SSIM으로 페이지별 픽셀 비교하는 체계를 구축했다.

- 파이프라인 위치: `visual-diff/`
- 성공 기준: SSIM ≥ 95%, 페이지 수 100% 일치
- 이 파이프라인은 #249 이후에도 계속 활용 예정 (#250 Right Tab 수정 검증 등)

## 5. 수정 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/svg.rs` | PUA 변환 적용 |
| `src/renderer/web_canvas.rs` | PUA 변환 적용 |
| `src/renderer/html.rs` | PUA 변환 적용 |
| `src/renderer/layout/paragraph_layout.rs` | border_fill margin 반영 |
| `src/renderer/layout/table_layout.rs` | 표 외곽 테두리 fallback |
| `src/renderer/layout.rs` | clip_rect 콘텐츠 레이아웃 후 확정 |
