# Task 307 구현계획서: HWPX→IR LINE_SEG 계산 (인코딩 동등성)

## 1. 설계 원칙
- **렌더러는 하나**: HWP/HWPX 구분 없이 IR만 렌더링 (렌더러 패치 금지)
- **인코딩 단계에서 IR 동등성**: HWPX→IR에서 HWP에 있지만 HWPX에 없는 값을 계산

## 2. IR 비교 분석 (동일 문서 HWP↔HWPX)

| 항목 | HWP IR | HWPX IR (현재) |
|------|--------|---------------|
| 문단 0.0 ls[0] lh | 4091 (TAC 표 높이 포함) | 100 (기본값) |
| 문단 0.4 vpos | 13634 (비-TAC 표 높이 반영) | 2947 (미반영) |

원인: HWPX 본문 문단에 `<hp:linesegarray>`가 원래 없음 → 파서가 기본값 생성

## 3. 구현 계획

### 3.1 단계 1: HWPX 인코딩 시 LINE_SEG 계산
**위치**: `document.rs` (현재 사후 패치 위치를 정규 로직으로 전환)

linesegarray가 없는 문단에 대해:
1. **기본 lh**: 문단 스타일의 글꼴 크기 × 줄간격 비율로 계산
2. **TAC 컨트롤 lh 보정**: lh = max(기본 lh, 컨트롤 높이)
3. **비-TAC TopAndBottom 이후 vpos**: 개체 높이 + v_offset을 후속 문단 vpos에 가산
4. **문단 간 vpos 연쇄 갱신**: running_vpos 누적

### 3.2 단계 2: 렌더러 HWPX 전용 패치 제거
- `layout.rs` vpos 하향 보정 제거 (1102~1128행)
- 렌더러가 IR만 보고 동작하도록 정리

### 3.3 단계 3: 검증
- 비교 대조군 HWPX ↔ HWP dump 비교 (lh, vpos 동일 확인)
- cargo test 716개 전체 통과

## 4. 영향 범위
- `src/document_core/commands/document.rs` — LINE_SEG 계산 정규화
- `src/renderer/layout.rs` — HWPX 전용 패치 제거
