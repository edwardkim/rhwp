---
kind: investigation
status: active
last_verified: 2026-10-08
---

# PR #7619 리뷰 — 재조판 본문의 Square 표 띠 회피

## 최종 판정

**승인 — 공개 입력 4건의 재조판 Square 표 줄 배치 범위.** 규칙 근거·focused 12 PASS·단독 Native/fresh WASM 전 4쪽(각 경로) 최저 99.35870% 및 직접 판독을 확인했다. 원 이슈 #7548 전체를 닫지 않는다. 아래 미검증·범위 밖 경계는 전체 해결로 확대하지 않는다.

최종 병합 확인: 작업지시자의 수용 지시 뒤 정확한 trailing head의 required checks·CodeQL 4언어·보조 검사 완료 및 current-base clean merge-tree를 다시 확인했다. merge commit `ea896d6a01e38e7be0f0c495e4d888be1d8378e0`, 시각 게이트 예외 없음.

## 접수와 처리 경로

- 원 PR: https://github.com/edwardkim/rhwp/pull/7619, 작성자 planet6897, base devel.
- 원 head: `0f1153d18c83043c449304996fc462a6b5b15edb`; 검토 base: `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832`.
- 누적 코드 후보: `499998a8888922e4d5171181e6f241fc28fc8654` (`review/planet6897-20261008`). 원 head와 다른 검토용 코드다.
- reviewer: jangster77(기존 지정). 메인테이너는 로컬에서 일괄 검토했다.
- 기본 경로 maintainer_general, 보조 intake_and_review/local_validation/multi_pr_update_branch/visual_fixture_evidence. 큰 diff는 rework_and_exceptions도 적용했다.
- PR 번호순 기능·문서 커밋 5개 `cherry-pick -x`, devel merge 커밋 제외. 최초 `5e8aea70d3911722861a769aca5a6f174f8c4178`, 마지막 `77f3932a12aced7871b7ecab9c3a118375b155b4`. 전체 매핑은 ignored `output/pr-review/planet6897-20261008/pick-mapping.json`.
- 개별 최신 head check 조회에서 실패·진행 중 없음(success 또는 정책상 skipped). Build & Test success가 full Rust 실행을 뜻하지는 않는다. 최신 devel과 개별 head merge-tree는 exit 0.

## 구현과 증거 대조

register_page_anchored_square_table → side_wrap_exclusions → LayoutFrame::carve → plan_plain_text/InlineFlowPlan → layout inline_flow_plans. host 글자 소유는 no_lineseg_host_text_flows_below_square_table을 공유한다.

| 검토 항목 | 현재 판정과 근거 |
| --- | --- |
| 구현 근거와 일반성 | 배제 영역 → 줄 계획 → 예약/실제 배치 소비는 코드상 확인했다. 선언/실측 표 높이 차이와 다음 단·편집 경계는 **미검증**으로 유지한다. 상세는 아래 「조판 규칙과 예외 분기 조사」에 원 head 코드로 연결 |
| 측정·배치 일관성 | 위 실제 소비 경로를 확인했다. 모든 특수 분기의 계약 충족은 미검증 |
| 분할·이어받기 | 영향을 주는 경로는 컷·소유·예약·실제 높이의 추가 대조가 필요하며 미검증; 관련 없는 경로에는 확대 적용하지 않음 |
| 줄 소속과 점유 높이 | 회귀 원본을 누적 candidate에서 focused 검사 통과, 전체 실제 문서 범위는 미검증; helper 성공만으로 전체 배치 충족으로 판단하지 않음 |
| 입력·독립 기대값 | 공개 입력·PDF와 원 SHA의 커밋 상태를 대조하고, 합성 입력과 정상 저장본을 구분함. 외부 입력/PDF는 커밋과 동일성 확인 전 미검증 |
| 기준값 변경 | 기여자의 diff와 설명을 검토함. baseline 증가·기대값 변경을 실행 통과만으로 수용하지 않음 |
| 주장과 검증 범위 | 원 PR 본문의 수치는 기여자 측정이다. 누적 head의 focused·시각 실행은 별도 결과로 기록한다 |

## 검증과 남은 조건

- `git diff --check upstream/devel...HEAD`: 통과.
- 전 PR 공통 focused 검사 18 case source(13 generated suite target)를 공유 target `/home/edward/mygithub/rhwp/target/pr-review`에서 실행했다. **84 PASS / 0 FAIL**, 2,817 skipped는 선택 필터 밖의 검사다. 실행 24.744초(컴파일 제외). 로그: `output/pr-review/planet6897-20261008/logs/focused.log`.
- source/기준 입력이 바뀐 devel merge가 각 PR의 오래된 이미지 SHA 이후에 들어왔다. 기존 본문 PNG URL을 최신 head 증적으로 재사용하지 않는다.
- 누적 Native/fresh WASM 공개 대조 입력 결과를 아래 또는 공통 보고서에 기록했다. 원 PR 전 범위·전체 CI·Clippy는 아직 완료로 판단하지 않는다.
- 관련 이슈 #7548는 부분 범위이므로 자동 종료 표현을 준비하지 않는다.
- 원격 comment/review/merge/push/PR 생성은 수행하지 않았다.

## 보류 해제 조건

현재 base와 최종 후보에서 해당 수정 경계의 focused 검사, 전체 영향 범위 Native/fresh WASM·쪽수·직접 PNG 판독, 입력/PDF 커밋 증거 및 필요한 lint/CI를 확보한다. 낮은 점수·미측정 쪽과 실제 회귀를 구분하고, 실제 배치 차이를 기존 차이라는 이유로 면제하지 않는다.

## 최신 누적 Native 직접 출력

source `499998a8888922e4d5171181e6f241fc28fc8654`, release-test 빌드, 실제 HCR Batang/HCR Dotum을 포함한 Windows 글꼴 공급, Chrome 154에서 인쇄 프로필로 비교했다.

| 입력 | 전쪽 | 최저 실루엣 | 90% 미만 |
| --- | ---: | ---: | ---: |
| 7619-full | 1 | 99.45089% | 0 |
| 7619-lane | 1 | 100.00000% | 0 |
| 7619-narrow | 1 | 99.35870% | 0 |
| 7619-overlap | 1 | 99.45089% | 0 |

TSV: `output/pr-review/planet6897-20261008/native-scores/<key>/silhouette.tsv`. TSV 자체 gate는 not_evaluated로 최종 승인 판정이 아니다. fresh WASM·대표 review/overlay 직접 판독은 별도로 기록한다.

## 일괄 검토 종료 시점의 검증 범위

공통 누적 후보의 공개 7입력·10쪽은 Native/fresh WASM 전쪽 보조값이 동일하고 최저 99.05592%다. Native 대표 review PNG와 #7617 단 시작 입력 전 4쪽을 직접 판독했다. 전폭/옆 차선, 순서·누락·표 외곽·뒤 문단을 확인했고 표 괘선 약 1~2px 차이가 남는다. 이는 해당 공개 입력에 한정된 증거이며 이 PR 전체 영향 범위의 승인으로 확대하지 않는다. 누적 source와 원 head가 다르므로 원 PR 최신 전체 CI 통과로 보고하지 않는다.

이 절은 초기 접수 당시의 상태를 남긴 기록이다. 상세 판정·TSV·24쪽 미해결 출력은 ignored `batch-review-ko.md`에 보존했다. 이후 #7619의 단독 source·fresh WASM 검증 및 정확한 최종 head의 본문 증적·CI 재사용 확인은 아래 최종 절을 따른다.

## 단독 후보의 실제 소비 경로와 범위

- source `b87dd201179d5ff6e1dacdf8b7d18c7ce2b0058d`, base `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832`, worktree `/tmp/rhwp-planet6897-accepted-20261008`. 원 PR 최신 소스·테스트·입력과 동일하다.
- 생산: `typeset/state/transition.rs:329` → `float_placement.rs:2690`의 Page/Paper Square 배제 영역. 저장 줄 host는 조기 반환하며 TAC/캡션은 배제 영역 등록 대상이 아니다.
- 측정: `typeset/state.rs:827` 배제 영역을 frame input에 전달 → `inline_flow.rs:635` plain text 줄 구성 → `typeset/inline_flow.rs:63` fit·다음 단 후보 확인 → `state.rs:840` 실제 수용한 plan의 end·소속 저장.
- 실제 배치: `layout.rs:10343` FullParagraph에서 확정 plan을 소비하고 `col_area.y + plan.end`를 반환한다. 이 경로 뒤에서 별도 줄 구성·앵커 선택으로 덮어쓰지 않는다. host 글자 소유는 `mod.rs:2300`의 shared 판정을 typeset/layout이 같이 소비한다.
- 분할 경계는 기존 fit/다음 단 이동 경로를 사용한다. 4개 공개 입력은 모두 1쪽이므로 새로운 배제 영역이 예산 실패·다음 단 이월에 미치는 결과는 이 증거로 입증하지 않는다.
- 표 배제 영역은 선언 폭/높이와 바깥여백을 사용하지만 실제 표 외곽은 `table_layout.rs:3220`에서 행 높이 합을 소비한다. 선언/실측 차이 경계는 **코드 검토상 추가 확인 대상(미검증)**이며 실행으로 검출한 결함으로 바꾸어 보고하지 않는다.
- `issue_7548_reflow_square_band` 4 PASS, `issue_7548_page_anchored_square`/`issue_7548_square_lane_successor`/`issue_7481_no_lineseg_square_table_host_text` 8 PASS. source SHA에 고정한 별도 단독 결과다. 로그 `selected-focused.log`, `selected-controls.log`.
- 단독 Native release-test 빌드 성공(2분 28초). Docker 표준 경로 fresh WASM과 단독 시각 비교를 새로 실행했다. 최종 결과는 공통 보고서에 고정하며, 누적 후보의 결과를 단독 결과로 재사용하지 않는다.
- 초기 단계에는 로컬 전체 lint/회귀 실행안을 `selected-full-ci-plan.json`에 준비했다. 최종 단계에서는 이미 녹색인 GitHub Full CI와 자동 bridge·문서 계보를 확인하여 중복 로컬 실행을 생략했다. 새로운 경계의 충분성을 전체 CI 성공만으로 대체하지 않는다.

## 단독 공개 입력 검증 결과

source `b87dd201179d5ff6e1dacdf8b7d18c7ce2b0058d`, Native release-test 및 Docker 최적화 fresh WASM. 4개 공개 입력의 전 4쪽을 각 경로로 비교했고 최저 **99.35870%**, 90% 미만·누락·쪽수 차이 없음, 두 경로 gate 모두 passed. 8개 review PNG도 각각 열어 실제 배치를 판독했다. 표 괘선 약 1~2px 차이는 남는다. 입력별 값·compare/overlay/review 절대 경로와 엄격 보조값은 공통 `batch-review-ko.md`의 단독 완료 표에 기록했다. 이 단계에는 로컬 전체 CI를 실행하지 않았다. 이후 GitHub Full CI 재사용 근거는 아래 최종 절에 기록했으며, 앞서 명시한 추가 경계는 미검증으로 유지한다.

## 조판 규칙과 예외 분기 조사

원 head `0f1153d18c83043c449304996fc462a6b5b15edb`의 merge-base 대비 `src`/`crates` diff와 실제 호출 경로를 대조했다. 원 head별 코드 링크이며 누적 후보의 다른 PR 변경을 이 PR에 귀속하지 않는다. saved diff의 재계산 일치 9/9는 `rule-audit/source-locations.json`에 기록했다.

**분류: 공유 줄 구성 결과에 근거한 규칙.** 배제 영역 → 줄 계획 → 예약/실제 배치 소비는 코드상 확인했다. 선언/실측 표 높이 차이와 다음 단·편집 경계는 **미검증**으로 유지한다.

[Page/Paper Square 표의 배제 상자](https://github.com/planet6897/rhwp/blob/0f1153d18c83043c449304996fc462a6b5b15edb/src/renderer/float_placement.rs#L2690)는 `treat_as_char=false`, `TextWrap::Square`, 기준좌표 `Page/Paper`, 바깥여백, `TextFlow`에서 도출한다. `allow_overlap`은 객체끼리의 겹침 허용과 글줄 배제를 구분한다. 문서 ID·행 개수·특정 높이 범위로 배치를 선택하지 않는다. 캡션 제외는 별도 배제 상자가 필요한 미지원 범위다.

[저장 줄 없는 host의 상자 등록](https://github.com/planet6897/rhwp/blob/0f1153d18c83043c449304996fc462a6b5b15edb/src/renderer/typeset/state/transition.rs#L329) `→ inline_flow_input.exclusions → plan_plain_text/LayoutFrame → typeset_inline_flow fit →` [commit_inline_flow의 plan/end 저장](https://github.com/planet6897/rhwp/blob/0f1153d18c83043c449304996fc462a6b5b15edb/src/renderer/typeset/state.rs#L840) `→` [FullParagraph의 같은 plan 배치](https://github.com/planet6897/rhwp/blob/0f1153d18c83043c449304996fc462a6b5b15edb/src/renderer/layout.rs#L10343) 순으로 확인했다. 이 FullParagraph 분기는 `col_area.y + plan.end`를 반환하므로 같은 분기의 legacy 앵커 계산을 다시 실행하지 않는다. 합성 줄과 실제 저장 줄을 구분하는 guard도 출처에 근거한다.

다만 새 상자는 `common.width/height` 선언값을 사용하고 실제 표 외곽은 실측 행 높이를 사용한다. 선언/실측이 다른 경우까지 동일 점유라고 입증되지는 않았다. 앞선 단독 Native/fresh WASM 4개 대조의 통과와 이 미검증 범위를 함께 유지하며, 새 수용 승인으로 바꾸지 않는다.

이번 추가 조사는 코드 검토다. 기존 실행 증거는 앞 절을 유지하고, 새 반례 실행·전체 CI·원격 게시·통합은 수행하지 않았다.

## 최종 head 증적과 CI 재사용 — 2026-10-08

- 원 contributor code head `0f1153d18c83043c449304996fc462a6b5b15edb`와 단독 검증 source `b87dd201179d5ff6e1dacdf8b7d18c7ce2b0058d`의 src/crates/tests/samples/pdf/Cargo 파일이 동일하다.
- trailing head `5918a07888cedaeb9ab266a87b8b396cc5d588e0`는 `mydocs/pr/assets/pr7619_maintainer_20261008/`의 새 review·overlay 16 PNG 및 README만 추가했다. 원 코드·테스트·기준 PDF·기여자 증적은 변경하지 않았다. 새 head에서 별도 코드 빌드를 했다고 주장하지 않는다.
- [Full CI 37543258011](https://github.com/edwardkim/rhwp/actions/runs/37543258011) head `77f3932a12aced7871b7ecab9c3a118375b155b4`: lint success, Native Skia success, A/B/C/D builder·전체 shard success, Build & Test success, 별도 CodeQL/Render Diff/Adapter/Proptest success.
- 원 code head는 위 candidate와 devel `b3c3047db575d146dff9d39098e6de98c4630b3e`의 자동 merge-tree `9a380f00d3b9dd8d233114781b92bcd7ede10b5c`와 정확히 같다. [latest code-head CI 37637749126](https://github.com/edwardkim/rhwp/actions/runs/37637749126)의 preflight 로그가 candidate SHA 및 `direct-source-build-and-test-green:success`를 확인했다. skipped 작업을 새 Full CI 실행 성공으로 세지 않았다.
- 최종 trailing head와 검토 base `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832` merge-tree `384beb2852d18bdaba2772bedcd8ec1f7387a697`, exit 0, `git diff --check` PASS. 현재 base의 이후 차이는 workflow/CI 운영 코드·문서이고 Rust 제품 코드·입력은 동일하다. README가 실제 merge-tree에 존재함을 확인했다.
- 광범위 로컬 release-test·Native Skia·lint 중복 실행은 녹색 Full candidate·자동 bridge·문서 후행 계보를 재사용하여 생략했다. focused 12개와 실제 Native/fresh WASM 시각 검증은 별도로 수행했다.
- PR 본문을 UTF-8 파일로 PATCH하고 API 원문 일치·BOM/`??` 부재를 확인했다. 정확한 final head의 16개 실제 Markdown 이미지를 표시했다. 새 PNG는 이미 실행한 단독 source 출력이며 원 PR 코드와의 동일성을 함께 공개했다.
- HANGUL-only 원 입력 89.04%는 7언어 fontface 선언 대조군과 별개다. 미실행 편집·다음 단 이월·PAPER 정렬 조합·선언/실측 표 높이 차이, TopAndBottom 및 2px 표 외곽 차이는 부분 해결의 후속 범위로 유지한다.

## Merge 후 contributor PR comment 계획

- 원 PR 및 관련 이슈 #7548을 확인하고, 부분 해결이므로 #7548은 OPEN으로 유지한다.
- archive review와 오늘할일에 실제 merge SHA·최종 head CI·issue 상태를 기록한다. 승인된 maintainer 운영 기록 범위만 반영한다.
- 실제 merge SHA로 `mydocs/pr/assets/pr7619_maintainer_20261008/7619-<full|narrow|overlap|lane>-p001-<native|wasm>-<review|overlay>.png`가 존재함을 확인한다.
- 한국어 존댓말로 merge 사실, 원 기여의 규칙 개선, focused 12 PASS, 전 4쪽/경로 min 99.35870%, 직접 판독한 순서·겹침·차선·전폭 복귀 및 1~2px 외곽 잔차를 적는다. 다른 경계의 완료·이슈 전체 해결은 주장하지 않는다.
- [Visual Sweep GitHub merge comment 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 actual merge SHA로 고정한 Native/fresh WASM review PNG를 실제 Markdown 이미지로 표시한다. review-only 추가 commit은 원 코드 보정으로 표현하지 않는다.
- `--body-file`로 게시하고 API로 실제 본문·BOM·문자 치환을 확인한다. 정확한 comment URL을 이 기록에 남긴다.
- 종료 때 선택 후보/최종 head 전용 clean worktree·local branch만 정리한다. 나머지 8건의 일괄 검토 WIP와 공유 target은 보존한다.

## 실제 병합과 후속 기록

- merge SHA `ea896d6a01e38e7be0f0c495e4d888be1d8378e0`, 병합 시각 `2026-10-07T23:49:49Z`. 작업지시자의 수용 승인 범위에서 일반 merge commit으로 원 contributor 이력을 보존했다.
- 최종 review: [정확한 head 승인](https://github.com/edwardkim/rhwp/pull/7619#pullrequestreview-5449763474).
- 최종 [CI](https://github.com/edwardkim/rhwp/actions/runs/37702673663) 및 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37702673667) 완료. CodeQL JavaScript·Rust·Python·Actions 4개 Analyze가 모두 success. GHAS policy는 neutral이며 실패로 해석하지 않는다. Adapter/Proptest/Render Diff는 정책상 skipped worker와 success preflight를 구분한다.
- [#7548](https://github.com/edwardkim/rhwp/issues/7548)은 OPEN 확인. 원 PR도 Refs만 사용하며 부분 해결이므로 close하지 않았다.
- 문서 처리 결정: **maintainer 직접 반영**. 이 archive review·오늘할일만 운영 기록 commit으로 devel에 반영한다. 시각 asset 16 PNG는 원 PR merge commit에 이미 포함되어 있다.
- 병합 후 [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/37704454691) success를 확인했다. 로그는 `ready:false`, `reason:no-verified-pr-duration-measurements`이며 새 duration 실측을 갱신하지 않은 정상 보류다. 후속 Full CI·CodeQL·Adapter·Proptest·Oracle을 시작하거나 재실행하지 않았다.
- 한국어 [부분 해결 issue 안내](https://github.com/edwardkim/rhwp/issues/7548#issuecomment-6049197367)와 [contributor PR 안내](https://github.com/edwardkim/rhwp/pull/7619#issuecomment-6049197741)를 문서 반영·devel sync 뒤 게시하고 API로 UTF-8 본문 일치·BOM/치환 부재를 확인했다.

#7619만을 위한 clean worktree 2개와 local branch 2개를 제거했다. devel에 동일 source·test·fixture·PDF가 보존됨을 사전 대조했다. 8건의 열린 PR 검토 worktree와 공유 `target/pr-review` 및 contributor fork branch는 보존했다.
