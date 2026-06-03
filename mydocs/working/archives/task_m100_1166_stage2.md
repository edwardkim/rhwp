# Stage 2 보고서 — Task M100-1166: HWPX 저장 시 용지 방향 보존

## 목표
HWPX 직렬화 + HWP5 직렬화(HWPX→HWP 저장)에서 용지 가로/세로 보존.

## 변경
### src/serializer/hwpx/section.rs (replace_page_pr 신규)
템플릿 pagePr 하드코딩(landscape="WIDELY" width=59528 height=84186) → IR page_def 값으로 치환.
landscape: page_def.landscape ? "NARROWLY" : "WIDELY".

### src/serializer/control.rs (serialize_page_def)
HWP5 직렬화 시 landscape 를 attr bit0 에 동기화.
- 파싱: body_text.rs `pd.landscape = attr & 0x01`
- HWPX 출처는 pd.landscape 만 set, attr bit0=0 → 직렬화 시 손실되던 결함.
- 정정: `attr = if landscape { attr|0x01 } else { attr & !0x01 }`.

### tests/issue_1166_landscape.rs
- landscape_hwpx_roundtrip_preserves_narrowly: HWPX 저장 NARROWLY 유지 + 재로드 landscape=true
- portrait_hwpx_roundtrip_preserves_widely: 세로 WIDELY 유지
- landscape_hwpx_to_hwp5_save_preserves_landscape: HWPX→HWP5 저장 후 재파싱 landscape=true (작업지시자 지적 결함 가드)

## 검증
- issue_1166 6 passed (파싱 3 + round-trip 3)
- cargo test --tests 전수 통과, fmt=0

## 다음 (Stage 3)
통합 검증 + e2e/시각 (가로 렌더 + HWP 저장본 가로 유지).
