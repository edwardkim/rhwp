---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 48 — issue2007 부모 RowCut 소유 경계 보정

## 목적

[Stage 47](task_m100_3820_stage47_issue2007_parent_cut_analysis.md)에서 확정한
issue2007 p12→p15 연쇄 소유권 오차를 부모 `RowCut`에서 수정한다. 한컴 PDF의 결과를
맞추되 p10→p11 저장 프레임 경계와 p17 terminal child cursor 계약은 유지한다.

## 시작 기준

- 시작 commit: `8b20f3405`
- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 시작 집중 회귀: 11 passed / 1 failed
- 시작 실패:
  `issue_2007_continuation_frame_restarts_and_drops_previous_page_residual`

## 수정 원칙

1. 텍스트 내용, 문단 번호, 고정 페이지 번호로 분기하지 않는다.
2. 모든 continuation을 고정 px 또는 고정 unit 수만큼 이동하지 않는다.
3. 부모 cut이 이미 source unit을 다음 페이지에 넘겼다면 child scalar viewport에서 다시
   보정하지 않는다.
4. 실제 paint tree의 페이지별 소유권으로 검증하고 clip 밖 source text는 성공으로 세지 않는다.
5. p12 하단의 짧은 제목이 뒤의 page-scale recursive block과 source상 한 묶음임을 표현하는
   기존 unit marker가 충분한지 먼저 판정한다. 불충분하면 marker 생성 지점부터 보완한다.

## 완료 조건

- p12에 `4 국가인권위원회`가 남지 않고 p13에서 시작한다.
- p13에 감사원 항목 2가 없고 p14에 있다.
- p14에 금융위원회 항목 8이 없고 p15에 있다.
- p15 상단 continuation frame과 조달청 마지막 문장이 모두 보인다.
- 기존 issue2007 집중 회귀 12건이 모두 통과한다.
- 변경으로 직접 영향받는 회귀를 먼저 통과시킨 뒤 전체 integration gate로 확장한다.

## 진행 기록

Stage 47에서 시도한 광범위 `mixed_nested_starts_after_table` tail rewind는 p11까지 바꿔
폐기했다. 이번 단계는 p12 경계와 p10 경계의 실제 `CellUnit` source metadata 차이를 먼저
확정한 뒤 그 차이만 predicate로 사용한다.

## 원인과 수정

### p12 → p13: 재귀 block prelude 의미 소실

원본 자식 셀은 다음 세 문단으로 구성된다.

1. 빈 구분 문단
2. 한 줄 제목 `4 국가인권위원회`
3. 정확히 하나의 1×1 표를 host하는 문단

재귀 투영 뒤에는 자식 `para_idx`가 부모 host 문단 인덱스로 평탄화되어 이 관계가
사라졌다. 종전 실험의 `next.height > avail * 0.5`와 80px 잔여 제한은 같은 source
계약을 표현하지 못하므로 제거했다.

`RecursiveBlockPreludeRole`을 `CellUnit`과 `NestedFlowFragment`에 추가했다. source에서
정확히 한 개 empty unit + 한 줄 제목 + 다음 1×1 표인 경우에만 두 prelude unit을
표시하고, 모든 재귀 투영 단계에서 그대로 전달한다. capacity stop에서 다음 재귀 unit이
실제로 들어가지 않을 때만 표시된 두 unit을 함께 되감는다. 텍스트·페이지 번호·고정 높이는
판정에 사용하지 않는다.

### p14 → p15: 단일 문단간 저장 프레임 경계 소실

금융위원회 항목 7의 마지막 저장 좌표는 `vpos=32932, line_height=1000`이고 항목 8은
`vpos=0`에서 시작한다. `33932HU → 0` 되감기는 기존 body-half 물리 프레임 판정을
충족한다. 관련 para style의 `keepWithNext`, `keepLines`, `widowOrphan`,
`pageBreakBefore`는 모두 false여서 문단 keep 문제가 아니다.

기존 canonical 재귀 gate는 hard boundary가 둘 이상이거나 같은 문단 내부 저장 경계가
있을 때만 열렸다. 문단 사이의 authoritative 경계 한 개는 legacy scalar fallback으로
빠졌고, fallback이 `hard=false`, `stored=false`로 고정해 부모 unit에서 경계를 잃었다.
문단 내부·사이 구분 없이 기존 `is_hwp5_stored_frame_rewind`를 통과한 경계 하나도 canonical
child cursor로 투영하도록 수정했다. 작은 local reset은 기존 body-half 판정에서 계속
제외된다.

### 재귀 viewport의 마지막 줄 clip

재귀 child cursor는 continuation 시작 source offset을 첫 가시 unit만큼 되감지만 표시
viewport는 늘리지 않아 end cut 안의 마지막 줄이 clip 밖으로 나갈 수 있었다. 비종료
재귀 continuation의 표시 높이만 같은 첫 가시 unit만큼 복원하고 flow 높이는 유지했다.
child cut이 source 끝을 제한하므로 다음 페이지 owner를 다시 그리지 않는다.

## 회귀 고정

기존 continuation frame 테스트에 실제 paint clip을 적용한 다음 소유권 assertion을
추가했다.

- p13: 감사원 항목 2 미표시
- p14: 감사원 항목 2 및 금융위원회 제목 표시, 금융위원회 항목 8 미표시
- p15: 금융위원회 항목 8 표시, 제목 미반복, 조달청 마지막 문장 표시

## 검증 결과와 다음 단계

전용 target에서 임시 진단 코드를 제거한 뒤 다음을 확인했다.

```bash
CARGO_INCREMENTAL=0 \
CARGO_TARGET_DIR=target/task-3820-3821-fidelity-rebase \
cargo test --profile release-test \
  --test issue_2007_nested_cell_pagination -- --test-threads=1
```

- 최초 직렬 실행: **12 passed / 0 failed**
- 동일 명령 최신 재실행: **11 passed / 1 failed**
  - 실패: p15의 `제기할 수 있다.` paint assertion
  - pagination cut은 동일하며 아래 포인터 캐시 충돌에 따라 재귀 셀이 간헐적으로
    잘못 복원되는 별도 결함으로 분리했다.
- p13–p15 새 SVG와 한컴 PDF 직접 대조:
  - `tmp/pdfs/stage48/current-final-legacy/`
  - `tmp/pdfs/stage48/pdf/`
- `cargo fmt --all`, `git diff --check`: 통과

다만 새 p15 paint assertion을 반복 실행하면 같은 소스와 같은 cut에서도 간헐적으로 빈
재귀 셀이 만들어졌다. pagination cut 로그는 동일하므로 이번 source 경계 수정과 별개다.
조사 결과 `cell_units_cache`가 임시 재귀 `Table::clone()`의 raw cell pointer를 키로 오래
보존하고, allocator가 다음 페이지 clone에 같은 주소를 재사용하면서 다른 표의 `Arc<Vec<CellUnit>>`
을 반환하는 order-dependent 결함으로 확정했다. 테스트 직렬화나 baseline 완화로 숨기지 않고,
Stage 49에서 임시 clone을 제거하고 안정 주소의 원본 중첩 표를 직접 렌더한 뒤 전체 12건과
새 paint assertion을 반복 검증한다.
