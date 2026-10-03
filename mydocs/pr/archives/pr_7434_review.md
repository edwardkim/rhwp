---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7434 검토 — 수정(조판): 조각 상자의 쪽 상한을 다행 표에도 적용한다 (#7095 확장, #7063)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7434, 작성자 planet6897, base `devel`, 정확한 head `1fd09e538bf340a8fcf905d225d3fc74f5262e1b`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `3cb7daba69fc734879e672434f1547c4a7d24100` | stacked/rebased duplicate | 수정(조판): 자리차지 RowBreak 조각의 바깥 위 여백을 원본 HWPX 계보에도 연다 (#7063) |
| `040fa9b2bf9a343c6117a670235be1f518e06963` | stacked/rebased duplicate | 수정(조판): 쪽 중간 조각의 상자·내용·정렬 높이를 가른다 (#7063 레인②) |
| `e3a4456bd263884d73fe59f759421b1ce222baf6` | stacked/rebased duplicate | 수정(조판): 쪽을 끝내는 조각의 마지막 행 상자를 배치 뒤에 접는다 (#6976) |
| `55a6e010d049af03cc7d8c6e43e32ae21bb1c126` | candidate | 수정(조판): 조각 상자의 쪽 상한을 다행 표에도 적용한다 (#7095 확장, #7063) |
| `1fd09e538bf340a8fcf905d225d3fc74f5262e1b` | merge excluded | Merge branch 'devel' into work/7095-multirow-cap |

## 변경 범위와 검토 계획

- 원 PR 변경 992줄 추가·36줄 삭제·42파일입니다.
- 생산 경로: `src/document_core/queries/cursor_rect.rs`, `src/renderer/layout/table_partial.rs`, `src/wasm_api/tests.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

### 누적 후보 검증 상태 갱신 (2026-10-02)

- GitHub를 다시 조회한 결과 선택 당시의 원 PR head와 CI green 상태가 유지됩니다. #7435는 현재도 non-green이며 선택에 포함하지 않았습니다. 누적 후보는 `review/planet6897-green-20261002`, source 고유 커밋63개입니다.
- 최초 전체 nextest는 10,283개 중10,250PASS/33FAIL/50SKIP, exit100으로 완료했습니다. 그 뒤 PR별 보정과 focused 재검증으로 최초 실패28개를 처리했으며 5개가 남았습니다. 전체 재실행 통과로 바꾸어 보고하지 않습니다.
- 각주 빈 번호·합성 사다리·다열 표 대조군·중첩 표 후속 원점의 잔존 5개와 whole fixture 시각 보류를 [현재 검증 기록](../assets/planet6897_green_20261002/review_progress.json)에 기록했습니다. 원 PR별 기존 분석·커밋 출처는 위 내용을 유지합니다.
- **현재 통합 승인/머지 보류**입니다. 원 PR의 green CI는 누적 후보의 실패 또는 미완료 Native/fresh WASM 시각 검증을 대체하지 않습니다. 새 통합 PR 생성·push·머지는 하지 않았습니다.

### 원 PR 최신 head·CI 재확인 — 통합 검증과 구분

- 2026-10-04 API 재조회: 원 PR은 OPEN, head `1fd09e538bf340a8fcf905d225d3fc74f5262e1b`로 기존 접수 기록과 같습니다. 원본 저장소의 해당 SHA check 32건은 skipped 20건, success 12건이며 실패·진행 중인 check는 없습니다.
- 현재 통합 후보 `a73100f16`에서 form002는 Native/fresh WASM 전10쪽 최저92.96827%와 기존 관련42건·SVG 묶음7건의 통과를 확인했습니다. 원 PR CI 통과를 통합 후보 전체 통과로 대체하지 않습니다. 76076 실제 물리6쪽의 본문/쪽번호 겹침, 다른 시각 보류 및 최종 전체 회귀·Skia 검증이 남아 있어 최종 승인·PR 제출은 계속 보류합니다.
- [정확한 source SHA별 check 증적](../assets/planet6897_green_20261002/source_ci_refresh_after_form002.json). 원 PR mergeability와 통합 분기 충돌 여부는 별개이며, 원 PR의 직접 병합은 수행하지 않았습니다.
