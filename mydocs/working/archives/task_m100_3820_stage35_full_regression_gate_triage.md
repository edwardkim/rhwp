---
kind: analysis
status: complete
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 35 — #3637 보정의 전체 회귀 게이트 분리

## 출발점

Stage 34 commit `06d62e0fa` 뒤 `cargo test --profile release-test --tests`는
`3284 passed; 5 failed; 8 ignored`(exit 101)였다.

- synthetic `typeset` 두 건은 HWP/HWPX 실물 오라클이 아닌 기존 `Paginator`와의 동치 기대다.
  Stage 34 diff는 RowBreak partial-table branch에만 있으므로 table 없는 fixture의 실패를 먼저
  Stage 34 결함으로 가정하지 않는다.
- HWPX `issue1949_giant_cell_nested_tables_perf`의 deferred pagination은 115 fragment
  기대가 116으로 바뀌었다. 이 fixture의 HWP 2020 및 2024 PDF 모두 115쪽이므로,
  이 HWPX-specific 변화는 우선 실제 회귀 후보로 취급한다.

## 판정 방법

1. `RHWP_DIAG_SCAN`으로 #3637과 issue1949에서 Stage 34 mixed-nested guard가 실제로
   발동한 table/cut shape를 비교한다.
2. issue1949의 2020 PDF 115쪽과 현재 원본 render tree 115쪽을 기준으로, mutation 후
   116 fragment가 올바른 source owner 보존인지 아니면 잘못된 extra split인지 분리한다.
3. guard가 issue1949에 과잉 적용된 것이 확인되면 #3637의 terminal 1×1 nested child shape로
   조건을 좁히고, #3637 p26→p27과 issue1949 deferred contract를 함께 재검증한다.
4. synthetic 두 test는 Stage 34 parent와 동일한지 별도 확인한 뒤에만 expectation을 고친다.
   독립 오라클 없는 기대값 갱신은 금지한다.

## 오라클과 원인 분리

### issue1949는 실제 회귀였다

기존 증적 `pdf/issue1949_giant_cell_nested_tables_perf-2020.pdf`와
`pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf`는 각각 A4 115쪽이다. 현재 원본
`samples/issue1949_giant_cell_nested_tables_perf.hwpx`도 `rhwp info --json`에서 115쪽으로
해석된다. 따라서 이 경우에는 새 MCP 2020 변환을 만들 필요 없이, 이미 보관된 HWP 2020
출력이 독립 오라클이다.

Stage 34의 계측에서 #3637은 `18×4` 표의 row 17 안에 `1×1` nested table이 있는
continuation이었고, issue1949는 `3×1` giant-cell 표의 동일한 논리 drift 조건이지만 그러한
nested child가 없었다. 범용 `mixed_nested_owner_guard`가 후자에도 발동하여 115 fragment
계약을 116으로 바꿨다. guard를 **현재 분할 row에 1×1 nested table이 실제로 있는 경우**로
제한했다. terminal HWPX fragment의 content-origin 보정은 source owner가 정해진 뒤에만
동작하는 viewport 보정으로 유지하되, 그 source-owner 보정과 함께 #3637 및 issue1949를 다시
확인했다.

### typeset 두 게이트는 빈 문단 입력이 계약을 흐렸다

- `test_typeset_force_break_before_hint`는 세 문단의 높이만 만들고 텍스트를 넣지 않았다.
  그러면 force-break가 대상으로 삼는 `current_items`가 없으므로 “para 1에서 끊어야 한다”는
  테스트 설명과 입력이 일치하지 않는다. 세 가시 문단으로 교정했다.
- `test_typeset_page_overflow`도 100개의 빈 문단을 사용했다. 그 경우 마지막 blank-only page
  정리 경로가 작동해 3쪽이 된다. 이는 마지막의 인쇄되지 않는 빈 쪽을 제거하는 별도 계약이지
  가시 문단 overflow의 기준이 아니다. 100개의 가시 2000-HWPUNIT 문단으로 바꾸면 4쪽이며,
  페이지 item이 `0..100`을 순서대로 모두 보존함을 직접 검증한다.

### #1116 font 게이트는 XML 직렬화 표기가 잘못된 기대였다

전체 integration 재실행 중 두 font test가 실패했지만, 실패 출력은 이미
`font-family="&apos;Palatino Linotype&apos;,..."`였다. 즉 HCI Poppy가 아니라 Palatino Linotype을
첫 family로 사용하고 있었다. 테스트만 XML의 `&apos;`를 고려하지 않은 채 비이탈 문자열을
찾았다. family-list를 XML entity 정규화 후 검사하도록 고쳐, SVG 직렬화 방식이 아니라 실제
font mapping을 검증한다.

## 변경과 확인

1. HWPX mixed-nested owner guard를 현재 row의 실제 1×1 nested child로 한정했다.
2. #3637 focused regression, issue1949의 #2214 deferred-pagination, #2424 resumable
   pagination/delete를 모두 다시 통과시켰다. issue1949 HWP/HWPX의 fragment 수는 각각 115다.
3. 두 synthetic test를 설명과 같은 가시 문단 계약으로 고치고 focused green을 확인했다.
4. #1116의 SVG entity-robust font assertion을 통과시켰다.
5. 전체 `cargo test --profile release-test --tests`는 위의 #1116 게이트 오류를 고친 상태에서
   다시 시작했다. 아래 handoff에 적은 기존 native-HWP 축을 만날 때까지 앞선 실패는 재발하지
   않았다.

## 59043 pagination pin 재판정

재실행은 `issue_1921_59043_pagination_pin`의 오래된 41쪽 exact pin에서도 멈췄다. 이
테스트의 실패 메시지는 41쪽 미만을 “개선이므로 핀과 37쪽 정답지를 갱신”하라고 명시한다.
실제 한글 2022 Print 1-up PDF는 37쪽이며, direct pair
`--text-only --export-all-svg --layout-ledger` 결과는 rhwp SVG/render-tree가 모두 **39쪽**,
즉 기준보다 +2쪽임을 확인했다.

이것은 완전 fidelity 해결 판정이 아니다. ledger는 p33–34의 visible text-excess와 p37 이후로
밀린 sequence 후보를 남긴다. 따라서 현 stage에서는 기존 41쪽보다 정답지에 가까운 도달값
39만 pagination pin으로 갱신하고, p33–34는 별도 page-layout fidelity 후보로 남긴다. 41쪽을
강제해 두 쪽을 다시 늘리는 것은 한글 PDF와 반대 방향의 게이트 회귀다.

## 전체 게이트 handoff

재실행은 이후 `issue_2007_nested_cell_pagination`에서 9개 중 4개 실패로 멈췄다.

- 17쪽 HWP 2020 계약이 18쪽으로 과분할된다.
- p11의 continuation 첫 줄이 physical nested-cell clip 위에 다시 나타나고, p12/p15의
  source owner도 뒤로 밀린다.

이 표본은 native HWP이고, Stage 35가 바꾼 owner guard는 `hwpx_stored_layout()`에서만
실행된다. guard를 재귀 중첩으로 넓히는 실험도 이 결과를 전혀 바꾸지 않았으므로 되돌렸다.
따라서 #2007은 Stage 35 변경의 허위 회귀가 아니라 별도 native-HWP continuation viewport
결함이다. Stage 36에서 `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`의 p10–p17을
기준으로 source-unit/clip origin을 분석하고 수정한다.

## 완료

Stage 35는 #3637 보정의 과잉 적용을 issue1949 HWP 2020/2024 115쪽 오라클로 제거하고, 실제
렌더링과 불일치하던 synthetic/SVG/pagination 게이트를 정정했다. 전체 스위트의 다음 차단점은
위 Stage 36 native-HWP 축으로 이월한다.
