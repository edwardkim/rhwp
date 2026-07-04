# PR #1412 검토 보고서 — task 1411 공식 미주 모양 모델 잔여 검증

## 기본 정보

- PR: https://github.com/edwardkim/rhwp/pull/1412
- 제목: `task 1411: 공식 미주 모양 모델 잔여 검증`
- 작성자: `jangster77`
- 관련 이슈: #1411 `PR #1410 후속: 공식 미주 모양 모델 잔여 검증`
- 상태: open
- base: `devel` `a2a1b383`
- head: `task_m100_1411` `c338a1d6`
- merge 상태: `MERGEABLE`, `CLEAN`
- 커밋: 7개
- 변경 파일: 10개, `+831/-0`

## 변경 범위

문서:

- `mydocs/orders/20260615.md`
- `mydocs/plans/task_m100_1411.md`
- `mydocs/plans/task_m100_1411_impl.md`
- `mydocs/working/task_m100_1411_stage1.md`
- `mydocs/working/task_m100_1411_stage2.md`
- `mydocs/working/task_m100_1411_stage3.md`
- `mydocs/working/task_m100_1411_stage4.md`
- `mydocs/working/task_m100_1411_stage5.md`
- `mydocs/report/task_m100_1411_report.md`

소스:

- `src/renderer/layout.rs`

## PR 요약

PR #1410 이후 남은 visual sweep 후보 중 `2022-10` p14의 실제 layout 결함을 보정한다.
textless tall equation tail 뒤 새 미주 문항 제목에서 visible content 기준 gap이 이미 확보된 경우,
layout 단계가 logical note title gap을 한 번 더 보존하지 않도록 제한한다.

동시에 같은 미주 본문까지는 생략한 gap을 유지하고, 다음 미주 제목으로 넘어갈 때만
`HeightCursor` vpos base를 지연 복원하여 후속 미주 전체가 같이 당겨지지 않도록 한다.

`2024-09`와 `2024-11` 잔여 후보는 공식 미주 모양값 계산식 잔여가 아니라
TAC shape/equation/table continuation split 계열 잔여로 문서화했다.

## CI 상태

GitHub Actions:

- `Build & Test`: success
- `Canvas visual diff`: success
- `Analyze (rust)`: success
- `Analyze (python)`: success
- `Analyze (javascript-typescript)`: success
- `CodeQL`: success
- `WASM Build`: skipped

## 로컬 확인

로컬 검토 브랜치:

- `local/pr1412-upstream`

실행 결과:

- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `cargo test --lib compact_endnote_question_title_after_tall_tail_limited_backtrack -- --nocapture`: 통과
  - 1 passed, 1824 filtered out
- `cargo build --bin rhwp`: 통과

## 기술 평가

차단 이슈는 확인되지 않았다.

보정 조건은 다음 축으로 좁혀져 있다.

- `current_is_endnote_question_title`
- `col_content.endnote_flow`
- 직전 문단이 visible text 없는 TAC equation tail
- 직전 tail이 continued partial이 아님
- 현재 y가 단 하단부(`65%` 이후)
- 직전 content bottom 기준으로 필요한 gap이 이미 보이는 상태

따라서 일반 미주 제목 gap 보존 경로 전체를 건드리는 변경은 아니다.

`pending_textless_equation_tail_gap_restore`는 같은 미주 안에서는 복원하지 않고,
다음 미주 제목에서만 `same_endnote_control` 기준으로 다른 control을 만나면 복원한다.
이 설계는 PR 설명의 "같은 미주 본문까지는 생략 유지, 다음 미주 제목에서 지연 복원"과 일치한다.

## 잔여 리스크

- 실제 소스 변경은 `layout.rs`인데, PR에 새 단위 테스트는 추가되지 않았다.
  다만 PR 문서에는 targeted visual sweep 결과가 있고, GitHub `Canvas visual diff`가 통과했다.
- `0.65` 단 하단부 임계값은 샘플 기반 휴리스틱이다. 적용 범위는 좁지만 장기적으로는
  TAC equation tail split 정책과 함께 더 구조적인 기준으로 정리할 여지가 있다.
- `2024-09`, `2024-11` 잔여는 이번 PR에서 수정하지 않고 분류만 한다.
  이는 #1411의 목적에는 부합하지만, 별도 후속 이슈 후보로 남는다.

## 판단

현재 PR 목표에는 부합한다.

수용 전 선택 검증으로는 contributor가 기록한 targeted sweep을 유지보수자 로컬에서 한 번 더
재실행하는 방법이 가장 강하다. 다만 현재 CI와 로컬 focused 검증만으로도 PR 자체의
기본 수용 조건은 만족한 것으로 본다.

## 다음 절차 제안

1. 작업지시자 리뷰 보고서 승인
2. 필요 시 targeted visual sweep 재실행
3. 수용 절차 진행
4. 처리 보고서 `mydocs/pr/pr_1412_report.md` 작성
