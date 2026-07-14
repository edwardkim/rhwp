# PR #2257 검토 — RowBreak rowspan 블록 쪽 하단 밴드 필 (planet6897, #2097 계열)

- 검토일: 2026-07-14 / base: devel / 9파일 +478/−16 / 12커밋 / MERGEABLE, CI 11 green
- 요지: 쪽 하단 경계에서 plain 블록 컷 walk 의 fully_consumed 오판(행 시작
  y 무시)·기각 경계에서 오프셋 컷 재시도(밴드 필) + 쪽나눔=None 표의
  fresh-쪽 초과 통째 배치. 프로브 2건은 반증 기록만(동작 불변).

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| 전수 `--tests --no-fail-fast` | **3,158 / 0** |
| fmt / clippy(all-targets) | 통과 / 0 |
| 핀 FAILED 실증 (patch-revert, src 만 되돌림) | `issue_2097_band_fill` 3248363 쪽수 불일치로 실패 재현 → 복원 후 통과 |
| OVR 5샘플 (±2px, 분리 폴더 `output/poc/pr2257/`) | 회귀 0건 |

## 구조 검토

- **하드코딩 금지 준수**: 샘플명(3248363 등)은 주석에만 등장, 코드 분기 없음.
  가드는 전부 구조·기하 술어 — `(fully_consumed || !allow_block_split) &&
  allows_row_break_split && can_intra_split && !rowbreak_use_row_offsets &&
  r > cursor_row && blk_start_cut.is_empty() && block_h > budget &&
  budget >= MIN_TOP_KEEP_PX`.
- None 표 통째 배치는 `below_body_slack` 상한으로 경계 — 미관측 극단(용지
  초과)은 기존 분할 폴백 유지. 반증-교정 이력(1220000 스페이싱 포함 판정
  회귀 → 순수 표 높이로 정정) 기록됨.
- table_layout.rs 는 헬퍼 1개(각주 앵커 컷 예산 산정) + env var 진단 훅
  2개(동작 불변).
- 진단(RHWP_DIAG_BLKCUT/SCAN/FN)은 전부 env var 게이트 — 기본 경로 무영향.

## 컨트리뷰터 측 검증 (Windows + 한컴 — 1차 정답지 환경)

한글 2022 COM PageCount 실측 핀 5건, 잔존 36건 스크린 RESOLVED 5→8 회귀 0,
92 컨트롤셋 gate 85 일치, 358건 recount REGRESSED 0, 쪽 경계 내용의 한글
PDF 행/글자 단위 정합, roundtrip baseline hwpx 4/0·hwp5 4/0. 반증 표본으로
일반 규칙을 기각한 프로브 기록 2건 포함 — 케이스별 가드 원칙 정합.

## 판단

**approve → merge 수용 권고.** 시각 판정 관련: 발동 샘플의 쪽 경계 정합이
컨트리뷰터의 한컴 COM/PDF 오라클(1차 정답지 등급)로 이미 검증되어 있어,
작업지시자 추가 판정은 선택 사항 (필요 시 3248363 쪽 2 before/after PNG
즉시 산출 가능).
