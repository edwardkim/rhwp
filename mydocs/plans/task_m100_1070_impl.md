# 구현계획서 — Task #1070: TAC 표 문단 후속 텍스트 표 높이만큼 추가 하강 overflow

- 이슈: edwardkim/rhwp#1070
- 브랜치: `local/task1070` (stream/devel `be2a71c4` 기준)
- 수행계획서: `task_m100_1070.md` (승인 완료)

## 관련 코드 (정독 출발점)

- `place_table_with_text`(`typeset.rs:2517`):
  - `pre_table_end_line` 산출 (2529~) — 표줄 인덱스 판정(lh 기반 find).
  - `tac_wrap_split`(2597) — `treat_as_char && pre_end>0 && pre_end<total_lines`.
  - **`post_table_start`(2622~2630)** — 핵심 의심처. 2624 `else if table.attr & 0x01 != 0`
    분기가 HWPX TAC 표(비트0=0)에서 미진입 → else(2629) `pre_table_end_line`(=0) →
    표줄(line0) 포함.
  - `should_add_post_text`(2639) / post PartialParagraph push(2644~).
- 표 디스패치 + TAC 높이 보정(2332, 2421~2465) — attr&0x01 의존 사이트.
- 비교 기준: #1068 실험B revert 기록(`working/task_m100_1068_stage4.md`) — 블록 경로
  attr↔treat_as_char 동일시가 sample16/aift/mel-001 회귀.

## 단계 (4단계)

### Stage 1 — 조사 + 회귀 구조 규명 (소스 무변경)
- DIAG 로 3 재현 파일(기부·답례품 pi=25, hwpx-h-02/해외직접투자 pi=51)의
  `pre_table_end_line / post_table_start / total_lines / 표줄 인덱스` 확정.
- **#1068 실험B 회귀 구조 규명**: sample16/aift/mel-001 의 해당 TAC 표 문단 구조
  (pre_table_end_line, 표 위치, post-text 유무)를 dump 로 비교 → "표줄이 line0 이고
  post-text 가 표줄을 포함하는 케이스" 와 "정상 케이스" 의 판별 신호 도출.
- 산출물: `working/task_m100_1070_stage1.md`.

### Stage 2 — 설계 + 페이퍼 검증 (소스 무변경)
- 정밀 조건 설계: post-text 가 **표줄(pre_table_end_line)을 포함하지 않도록** 보정하되,
  실험B 광범위 회귀를 피하는 최소 게이트.
  - 후보: `treat_as_char` 표 + 표줄이 식별됨(pre_table_end_line 유효) + post-text 가
    표줄을 포함(post_table_start ≤ pre_table_end_line) → post_table_start 를
    `pre_table_end_line + 1` 로 보정.
  - tac_wrap_split(표줄이 마지막 줄이 아닐 때) 와의 경계 명시.
- 비회귀 케이스 표(tac-img-*, tac-case-*, table-in-tbox, sample16/aift/mel-001)로
  모순 점검 — 각 케이스에서 보정 발동 여부 + 기대 동작 일치 확인.
- 산출물: `working/task_m100_1070_stage2.md`.

### Stage 3 — 구현
- `place_table_with_text` post_table_start 산식에 Stage 2 정밀 조건 반영. 최소 변경.
- 단위 검증: 3 파일 overflow 해소 + 6 비회귀 파일(#1068 회귀세트) smoke.
- 산출물: 소스 + `working/task_m100_1070_stage3.md`.

### Stage 4 — 회귀 검증 + 회귀 가드
- 3 재현 파일 overflow 해소. 전수 sweep(samples hwp/hwpx) LAYOUT_OVERFLOW 합 회귀 0.
- 골든 SVG 8 종, `cargo test --release`(lib + 통합), clippy/fmt(변경 파일).
- 공개 픽스처 기반 회귀 가드 1~N 추가(`tests/issue_1070_*.rs`).
- 산출물: `working/task_m100_1070_stage4.md` → `report/task_m100_1070_report.md`.

## 완료 기준
3 재현 파일 표 문단 텍스트 overflow 해소(표 바로 아래 배치) + 비회귀 0 + 골든 회귀 0
+ 회귀 가드 통과.

## 리스크
- post_table_start 보정이 다중 TAC 표/Square wrap/페이지 분할 표와 상호작용 → Stage 2
  경계 명시 + 비회귀 케이스 전수 점검으로 차단.
- 표 바로 아래 텍스트가 한 페이지에 안 들어가는 경우(이월 필요)는 본 타스크 범위 외 가능 —
  Stage 1 에서 3 파일이 "표 fit + 텍스트만 hang" 인지 "표+텍스트 page-larger" 인지 구분.
