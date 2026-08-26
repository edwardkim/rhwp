---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 109 — 빈 terminal 표 sliver와 host 형제 각주

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `41bd2e5f8`
- 선행 구현: Stage 107의 native HWP5 번호형 RowBreak 표 host 형제 각주 owner 복원
- 감사 시작 상태: `src/renderer/typeset.rs`에 미커밋 보정 초안이 존재했다. 바로
  채택하지 않고 producer/consumer 계약과 실물 재현을 먼저 감사했다.

## 문제 가설

Stage 107의 형제 각주 경로는 같은 표의 terminal `PageItem`이 current 또는 직전
completed page에 있을 때만 각주를 등록한다. 일반 terminal `PartialTable`은
`end_row == row_count && end_cut.is_empty()`로 식별한다.

표 조판에는 마지막 행의 가시 콘텐츠가 직전 fragment에서 모두 방출되고, 다음 반복에
남은 영역이 `MIN_TOP_KEEP_PX`보다 작은 비가시 sliver뿐이면 새 terminal fragment를
방출하지 않고 cursor만 완료하는 `skip_terminal_empty_sliver` 경로가 있다. 이 경우
마지막 가시 `PartialTable`은 `end_row == row_count`이지만 `end_cut`이 남고, 별도
terminal item은 없다. 같은 host 문단의 바로 다음 Body 각주는 표가 실제로 끝났는데도
terminal owner를 찾지 못해 다시 소실될 수 있다는 것이 현재 가설이다.

## 미커밋 초안의 계약

감사한 diff는 다음 방향이었다.

1. 빈 terminal sliver를 생략할 때 `(para_index, control_index, row_count)` 1회 marker를
   남기면서 continuation cursor를 완료한다.
2. 형제 각주가 terminal placement를 조회할 때 동일 exact key의 가장 최신 표 item만
   본다.
3. marker가 정확히 일치할 때만 non-empty `end_cut`을 terminal로 인정한다.
4. 오래된 동일 표 fragment로 역탐색하거나 marker만으로 owner를 만들지 않는다.
5. 조회 뒤 exact marker를 소비하고, 다음 host 문단 시작 시 미소비 marker를 폐기한다.

## 완료 기준

- 실제 native HWP5 fixture/page에서 이 경로가 필요한지 source와 render tree로
  확인한다. 실물 재현이 없으면 일반 renderer 보정으로 채택하지 않는다.
- marker 없이 non-empty `end_cut`인 fragment는 terminal이 아니어야 한다.
- exact current/flushed fragment만 각각 current/flushed terminal로 인정해야 한다.
- 다른 표 key, 더 최신 미완료 fragment, 다음 문단에 marker가 유출되는 반례를
  통과해야 한다.
- 기존 Stage 107 p87/p91/p95 owner 및 215쪽 계약을 유지한다.
- 소스 확정 후 사용자 지정 선행 게이트 `issue_2430_cell_rewrap_threshold`를 가장 먼저
  실행하고, focused unit/integration, 전체 관련 실물 회귀를 순차 검증한다.

## 채택 시 검증 계획

1. 현재 diff와 producer인 `skip_terminal_empty_sliver`, consumer인
   `native_table_host_terminal_fragment_placement`를 read-only 감사한다.
2. 실제 정책연구 fixture와 기존 Stage 107 page owner 중 non-empty terminal
   `end_cut`/sliver 생략 형상을 찾는다.
3. 최소 synthetic 회귀에서 current, flushed, mismatched, stale-latest 항목을 고정한다.
4. 소스가 안정된 뒤 다음 순서로 실행한다.

   ```bash
   cargo test --profile release-test --test issue_2430_cell_rewrap_threshold
   cargo test --profile release-test --lib \
     renderer::typeset::tests::elided_terminal_table_sliver_marks_only_the_latest_exact_fragment_once \
     -- --exact
   cargo test --profile release-test \
     --test issue_3738_rowbreak_table_footnote_fragment
   ```

아래 실물 감사에서 producer 재현이 0건으로 확정돼 소스 채택 단계로 진입하지
않았다. 단위 반례·정책연구 33/33·#1486은 초안 감사 중 통과했지만,
채택하지 않은 코드에 대한 #2430 이후 전체 게이트는 실행하지 않았다.

## 실물 발동 감사

초안의 producer에 `RHWP_DIAG_ELIDED_TERMINAL` 1회 진단을 넣고 다음을
`CARGO_TARGET_DIR=target/pr-review`, `CARGO_INCREMENTAL=0`, `release-test`로 실행했다.
진단 키워드 `ELIDED_TERMINAL_TABLE` 발생 수를 실제 분기 진입 수로 사용했다.

| 게이트 | 결과 | empty-sliver 분기 |
| --- | --- | ---: |
| 정책연구 p87/p91/p95 host 형제 각주 owner 실물 회귀 | 1/1 passed | 0 |
| #1486 terminal RowBreak sliver 실물 회귀 | 1/1 passed | 0 |
| `overflow_cell_lines_do_not_grow` 실물 전수 sweep | 678건(스킵 3), 1/1 passed | 0 |

overflow sweep은 0이 아닌 문서 17종의 688줄을 계수했고 69.95초에 정상 종료했다.
정책연구 fixture의 p87·p91·p95는 모두 정상 terminal item 경로였고, #1486도
함수명과 달리 감사 대상 분기는 타지 않았다. 샘플 전수 sweep에서도 실제
producer 발동은 하나도 없었다.

재현 요약은
[diagnostic-summary.tsv](../pr/assets/task_m100_3820_stage109_elided_terminal_table_sliver/diagnostic-summary.tsv)에
고정했다. 원시 로컬 로그는
`output/task-3820-stage107-empty-sliver-gates/` 아래에 두었다.

## 채택 판정

초안의 exact marker·latest-item 계약 자체는 반례 단위 테스트를 통과했고,
정책연구 33/33과 #1486 회귀도 유지했다. 그러나 현재 저장소의 678건 실물에서
단 한 번도 발동하지 않아, 이 가설을 위해 일반 renderer state와 각주 owner 경로를
늘리는 것은 실사례 근거 원칙에 맞지 않다.

따라서 초안의 코드·단위 회귀·진단은 전부 제거했다. Stage 107의
`end_cut.is_empty()` terminal 계약과 커밋 `7093985f0`·`e9253d21d`는 그대로 유지한다.
향후 실제 HWP/HWPX에서 “직전 가시 fragment가 `end_row == row_count`, non-empty
`end_cut`인 채 빈 terminal sliver가 생략되고, 직후 host 형제 각주가 소실”되는
재현이 추가될 때만 이 문서의 exact one-shot marker 설계를 다시 채택한다.
