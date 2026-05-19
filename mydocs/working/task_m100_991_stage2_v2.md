# 2단계 보고서 v2 — 끝 페이지 패스 유도 방식 수정 완료

- 타스크: 로컬 task991
- 단계: 2/3 (수정 구현 — v3 접근, 성공)
- 1차 시도 실패 기록: `task_m100_991_stage2.md`
- 구현계획서: `task_m100_991_impl_v3.md`
- 작성일: 2026-05-19

## 1. 구현 내용

`src/renderer/layout/table_layout.rs` — `compute_cell_line_ranges` 의 분할 시작/중간 페이지(`has_offset`) 경로를 끝 페이지 패스 유도 방식으로 교체.

- 분할 끝 페이지(`!has_offset`) 로직은 **무수정** → 끝 페이지 렌더링 불변.
- `has_offset` 일 때: 신규 헬퍼 `cell_line_prefix_counts(budget)` 로 prefix 줄 수를 구해
  - 시작 줄 = `prefix(content_offset)`
  - 끝 줄 = `prefix(content_offset + content_limit)` (limit 없으면 문단 전체)
- `cell_line_prefix_counts` 는 `compute_cell_line_ranges` 를 `offset=0, limit=budget` 로 호출(끝 페이지 경로)하여 결과에서 prefix 줄 수를 추출. `offset=0` 이라 재귀는 1단계로 종료.

끝 페이지와 시작 페이지가 동일한 prefix 패스를 공유하므로 `limit_reached`·vpos 리셋·vpos 동기화가 양쪽에서 동일하게 작동 → 줄 중복·누락이 정의상 불가능.

## 2. 검증 결과

### 비공개 샘플 180쪽 전수 (수정 전 대비)

변경된 쪽 5개, 모두 정정. 복원된 내용은 한컴 2022 PDF 와 대조하여 존재 확인:

| 쪽 | 변화 | 종류 | PDF 대조 |
|----|------|------|---------|
| 7 | 중복 `☞ 사용자 편의성…` 제거 | 중복 제거 | — |
| 75 | `청렴계약 이행서약서…` 블록 복원 | 누락 복원 | PDF74 존재 ✓ |
| 96 | `수우미양가` → `수우미양가하` | 누락 복원 | PDF95 존재 ✓ |
| 127 | 중복 `소프트` 제거 | 중복 제거 | — |
| 142 | `□ 민감정보 처리 내역…` 블록 복원 | 누락 복원 | PDF139 존재 ✓ |

- 1차 시도(v2)에서 발생했던 **168쪽 줄 소실 회귀는 해소**됨(변경 목록에서 제외).
- 변경된 5쪽의 인접 쪽은 모두 불변 → 페이지네이션 교란 없음. 전체 페이지 수 180 불변.

### 빌드·테스트

- `cargo build --release` 성공.
- `cargo test --release` 전체 통과 (1297 + 부속 테스트, 0 failed). 골든 SVG 테스트 8건 포함 회귀 없음.
- `cargo clippy --release` 경고 0.

## 3. 다음 단계

3단계: 최종 회귀 점검(분할 표 다중 샘플 교차 확인) + WASM 재빌드 영향 확인 + 최종 결과보고서.
