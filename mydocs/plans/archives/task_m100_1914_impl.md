# 구현 계획서 — Task M100 #1914

## 단계

1. **판별 확정** — 서베이 53건(EOCD 실패 .hwpx) 매직 바이트 분류 + 제품 로드
   경로(`rhwp info`) 전수 재검 → 제품 경로 정상(49/49), 오분류 표면 = 게이트
   CLI 2종으로 확정.
2. **게이트 스니핑** — `hwpx_roundtrip_batch`/`hwp5_roundtrip_batch` 의
   `roundtrip_one` 진입부에 `detect_format` 판별:
   - `RoundtripRow.format_skip: Option<&'static str>` 신설 (실체 포맷명)
   - `status()` 최우선 분기 `FORMAT_SKIP`, `is_hard_fail()` 제외 (exit 0)
   - error 컬럼에 실체 + 올바른 게이트 안내
   - `Unknown`(빈 파일/DRM)은 종전 흐름 유지 (파싱 실패 = 정당한 거부)
3. **핀 + 전수 검증** — `tests/issue_1914.rs` (제품 스니핑 3포맷 핀 +
   CARGO_BIN_EXE 게이트 FORMAT_SKIP 핀), 서베이 53건 전수 재검, 전체 스위트.

## 비수정 범위

- 제품 로드 경로(`parse_document`/`DocumentCore`/`HwpDocument`) — 이미 내용
  기반. 수정 불필요 (핀만 추가).
- `render-diff` — parse_document 경유로 이미 정상.
- 배치 수집 필터(확장자 글롭) — 수집은 확장자, 판별·분류는 실체로 이원화.
