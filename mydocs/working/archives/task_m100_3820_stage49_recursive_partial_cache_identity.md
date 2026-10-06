---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 49 — 재귀 부분 표 캐시 identity 안정화

## 목적

[Stage 48](task_m100_3820_stage48_issue2007_parent_cut_repair.md)에서 복원한
issue2007 p12→p15 source 소유 경계를 페이지 렌더 순서와 allocator 주소 재사용에
관계없이 결정적으로 그린다. 테스트 직렬화나 페이지별 캐시 초기화로 숨기지 않고,
재귀 부분 표 렌더가 문서 모델의 안정적인 원본 `Table`을 직접 사용하도록 수정한다.

## 시작 기준

- 시작 commit: `d289ecd7b`
- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 집중 회귀 최신 결과: 11 passed / 1 failed
- 실패: p15의 `제기할 수 있다.` paint assertion

## 확정 원인

pagination cut은 정상·실패 실행에서 동일하지만 렌더 트리는 페이지 빌드 순서에 따라
달라진다. `cell_units_cache`는 `Cell`의 raw pointer를 문서 재조판 경계까지 보존한다.
그런데 재귀 부분 표 경로가 페이지마다 `Table::clone()`을 넣은 임시 `Paragraph`를 만들고
즉시 폐기한다. allocator가 이후 clone에 같은 cell 주소를 재사용하면 새 표가 이전 표의
`Arc<Vec<CellUnit>>`을 돌려받는다. LLDB에서도 동일 cell 주소가 서로 다른 임시 표에
재사용되는 것을 확인했다.

`table_nested_text_flag_cache`도 raw table pointer를 사용하므로 임시 clone을 유지한 채
키에 table pointer를 더하는 방식은 충분하지 않다. 두 임시 할당 주소가 함께 재사용될 수
있기 때문이다.

## 수정 원칙

1. `layout_partial_table`의 일반 wrapper는 기존 문단/control 해석과 outer/effective table
   선택, native host 문맥을 그대로 보존한다.
2. 실제 부분 표 렌더 구현은 이미 해석된 안정적인 `&Table`과 필요한 host metadata를
   직접 받는다.
3. 재귀 경로는 임시 `Paragraph`와 `Table::clone()`을 만들지 않고 원본
   `nested_table.as_ref()`를 내부 구현에 전달한다.
4. 캐시를 페이지마다 비우지 않는다. #1949의 O(pages×cell) 성능 개선을 유지한다.
5. `--test-threads=1`로 결함을 감추지 않는다.

## 회귀 계약

- 동일 `DocumentCore`에서 p1→p17을 순차 빌드한 p17 결과와 새 `DocumentCore`에서
  p17만 빌드한 결과의 painted text 또는 안정 해시가 같아야 한다.
- issue2007 집중 회귀 12건과 Stage 48의 p13→p15 paint 소유 assertion이 반복 실행에서
  모두 통과해야 한다.
- p15에 `제기할 수 있다.`가 셀 clip 안에서 보여야 한다.
- 이후 p13→p15를 한컴 PDF와 다시 직접 대조한다.

## 진행 기록

Stage 49 시작 시점에는 코드 변경을 하지 않았다. 먼저 일반 부분 표 호출과 재귀 호출의
host 문맥 사용 지점을 분리해, clone 제거가 반복 outer margin·caption·render-node source
metadata에 미치는 영향을 확인한다.

## 구현

### 안정적인 원본 표 참조

`layout_partial_table`은 기존 공개 시그니처를 유지하는 wrapper로 두고 다음 host 값을 한
번만 해석한다.

- 원본 문단/control index
- native empty-host 반복 margin 여부
- pre-emitted host 높이
- caption에 쓰는 host line spacing

실제 렌더 본문은 `layout_partial_table_resolved`로 옮겨 이미 해석된 원본 `&Table`과
`PartialTableHostContext`를 받게 했다. 정상 호출은 기존 outer/effective table 선택을
그대로 거치고, 재귀 child cursor는 임시 `Paragraph`와 `Table::clone()` 없이 문서 모델의
`nested_table.as_ref()`를 직접 전달한다. 재귀 synthetic host의 기존 계약인 source
metadata `0/0`, repeat margin false, pre-emitted 높이 0, line spacing 0도 명시적으로
보존했다. `layout_partial_table_cells`의 사용되지 않던 외부 `paragraphs` 인자는 제거했다.

### p15 마지막 실제 줄의 paint viewport

clone 제거 뒤 p15 실패가 결정적으로 재현되어 캐시 결함과 별개인 물리 clip 오차를
분리할 수 있었다. 조달청 자식 1×1 표의 현재 cut은 실제 source unit을 모두 포함하고
바로 다음 unit만 content 없는 trailing reservation이었다. flow에서 그 예약을 제외한
높이로 scalar child cut을 다시 계산하면 셀 padding 때문에 마지막 실제 줄
`제기할 수 있다.`가 fitting budget 밖으로 한 줄 밀렸다.

다음 unit이 같은 문단의 content 없는 trailing reservation이고, 자식 1×1 표가 현재
viewport에서 새로 시작하며, 그 뒤 같은 mixed stream에 실제 source owner가 남지 않은
경우에만 그 실제 reservation 높이를 paint viewport에 보존한다. 뒤에 non-trailing unit
또는 content를 가진 trailing unit이 남아 있으면 reservation은 0이므로 end-cut 없는
scalar child renderer가 미래 콘텐츠를 현재 쪽에 노출하지 않는다. `flow_height`는 기존
`flow_visible`을 유지하므로 다음 sibling 위치와 부모 pagination은 바뀌지 않는다. 고정
px나 문장·페이지 번호는 predicate에 사용하지 않는다.

## 회귀 고정

`issue_2007_recursive_partial_render_is_page_order_independent`를 추가했다.

1. 첫 `DocumentCore`에서 p1→p16을 순차 렌더한 뒤 p17 텍스트를 얻는다.
2. 새 `DocumentCore`에서 p17만 직접 렌더한다.
3. 두 결과가 완전히 같아야 한다.

이는 테스트 직렬화와 달리 페이지 warm 순서가 cache identity를 바꾸지 않는다는 실제
계약을 고정한다. Stage 48의 p15 마지막 줄 paint assertion도 그대로 유지했다.

`trailing_reservation_does_not_extend_before_later_source_owner` 단위 회귀도 추가했다. 빈
reservation 바로 뒤에 같은 stream의 실제 owner가 남은 중간 fragment에서는 viewport
확장이 0이고, 실제 owner가 더 없는 최종 경계에서만 reservation 높이가 보존됨을 함께
고정한다.

## 검증 결과

전용 target과 비증분 빌드를 사용했다.

```bash
CARGO_INCREMENTAL=0 \
CARGO_TARGET_DIR=target/task-3820-3821-fidelity-rebase \
cargo test --profile release-test \
  --test issue_2007_nested_cell_pagination
```

- 집중 회귀: **13 passed / 0 failed**
- 기본 병렬 실행을 새 프로세스로 20회 반복: **20/20 성공**
- trailing reservation 음성 단위 회귀: **1 passed / 0 failed**
- focused clippy (`-D warnings`): 통과
- `cargo fmt --all`: 통과
- `git diff --check`: 통과
- 임시 Stage 49 진단 코드: 제거 확인

최신 바이너리로 p13–p15를 다시 내보내고 한컴 2020 PDF를 Poppler 96dpi로 렌더해
직접 대조했다.

- rhwp SVG/PNG: `tmp/pdfs/stage49/final-svg/`, `tmp/pdfs/stage49/final-png/`
- render tree: `tmp/pdfs/stage49/final-tree/`
- 기준 PDF PNG: `tmp/pdfs/stage49/pdf-96dpi/`
- p13: 감사원 항목 2 없음
- p14: 감사원 항목 2와 금융위원회 제목 표시, 금융위원회 항목 8 없음
- p15: 금융위원회 항목 8 표시, 제목 미반복, 조달청 마지막 문장 표시

Stage 49의 집중 계약은 완료했다. 전체 integration 회귀는 다음 스테이지에서 별도 실행해
실패 시 해당 test log를 기준으로 범위를 분리한다.
