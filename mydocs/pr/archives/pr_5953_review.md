# PR #5953 검토 기록

## 최종 판정

판정: 승인

검토한 변경 범위에서 추가 blocker를 발견하지 않았다. contributor 기능 변경을 그대로 수용하며 메인터너 source 보정은 추가하지 않았다. 작업지시자는 원 PR #5953에 이 기록과 오늘할일을 trailing commit으로 반영하고 최신 CI 통과 뒤 직접 merge하도록 승인했다. 최종 head의 required check와 정책상 skip 근거, MERGEABLE/CLEAN은 merge 직전에 확인한다.

## Metadata와 적용 경로

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#5953](https://github.com/edwardkim/rhwp/pull/5953) |
| 작성자 | lpaiu-cs |
| 사전 지정 reviewer | jangster77 |
| 원 head repository / branch | lpaiu-cs/rhwp / cleanup/5769-review |
| 정렬 전 head | 91b70532a241829d22e2e07108664e5219a4a8a7 |
| devel 정렬 후 head | f1b5f2cfe1b7037982c0421bc51b6901a62ce0ec |
| 정렬한 devel | a7b95f4041ef5d7d3574c4becfea5cb636eaf836 |
| 현재 직접 검토 branch | review/5953-direct-20260905 |
| 로컬 동등성 검증 candidate | c788cdc0a40b0f6087dc2a260f67e9c1f782a6d9 |
| 원 PR 고유 변경 | Frontend source 2개와 테스트 7개; 60줄 추가, 9줄 삭제 |
| 작성 시점 원 PR 상태 | OPEN, MERGEABLE, CLEAN; 정렬 후 head 기준 CI 완료 |
| 검토일 | 2026-09-05 |

GitHub의 정상 Update branch 경로로 최신 devel을 원 source에 병합했다. contributor 이력을 rebase/force-push하지 않았다. 이후 동일 devel 위의 통합 branch에 기능 commit 세 개만 `cherry-pick -x`로 적용했다. base 병합 commit 자체는 체리픽하지 않았다.

| 원 commit | 통합 commit | 범위 |
| --- | --- | --- |
| 24f52a136df6df80d3f5df7bd1110f72ccfbe821 | 563195408 | 교차 구역 선택의 스냅샷 폴백 |
| c4d678c95f7175527b0e679e5034cda823badede | 0e74386e0 | recordWithoutExecute 예산 강제 |
| 91b70532a241829d22e2e07108664e5219a4a8a7 | c788cdc0a | 위치 판정 가드 7개를 codeOnly로 전환 |

충돌 없이 적용됐다. 로컬 검증 candidate와 정렬 후 원 PR의 전체 Git tree가 모두 `0ac7d0c362342867e488b2dd6ebde1f99184ffb3`로 같았다. 따라서 아래 로컬 검증 결과는 같은 source tree에 해당한다.

앞선 승인으로 [통합 PR #6781](https://github.com/edwardkim/rhwp/pull/6781)을 생성했지만, 최신 작업지시자는 원 PR #5953의 직접 merge 경로를 선택했다. #6781은 merge 대상으로 사용하지 않는다. 이 기록과 오늘할일만 정렬된 원 contributor head 위에 문서 전용 trailing commit으로 추가한다. contributor 기능 commit은 다시 작성하지 않는다.

## 변경 검토와 판정 근거

### 교차 구역 삭제의 스냅샷 폴백

같은 구역의 일반 선택은 기존 FragmentDeleteCommand를 유지하고, 다른 구역으로 걸친 일반 선택은 SnapshotCommand로 분기한다. 기존 셀 경로와 selectionBefore의 범위·F3 단계 보존은 유지한다.

이 변경은 단일 구역 조각 캡처의 전제를 피하는 방어 보강이다. 실제 교차 구역 삭제 API의 처리 범위를 확장한 구현으로 판단하지 않았다. 원 PR도 실제 재현 문서를 확보하지 못했다고 명시했다. 아래 로컬 확인 역시 객체 경로와 상태 보존을 확인한 것이며, 실제 다구역 문서의 삭제·undo 완전성을 입증한 것은 아니다.

### 스냅샷 예산 강제

recordWithoutExecute의 일반 push, redo 폐기, maxSize 축출 이후에 기존 예산 강제 함수를 호출한다. wasm이 없는 기존 호출 형태와 command를 실행하지 않는 의미는 보존한다.

100개의 1-slot 모의 command를 기록한 동작 확인에서 실제 execute는 0회였고, 오래된 두 command를 폐기한 뒤 live snapshot 수가 98로 유지됐다. 현재 snapshot을 보유하지 않는 record command를 미래에 확장할 때의 방어이며, 임의의 snapshot 보유 mergeWith 구현까지 보장한다고 확대 해석하지 않는다.

### 위치 판정 가드

위치 비교에 사용하던 원문에서 주석을 공백으로 치환해 주석 속 이름 인용에 대한 오탐을 줄였다. 기존 codeOnly helper는 문자열과 줄 구조를 유지한다.

주석을 구간 경계로 사용하는 두 파일은 원문에서 먼저 구간을 자른 뒤 codeOnly를 적용한다. 따라서 경계 표지가 사라져 추출이 실패하는 문제를 피했다. 이 PR은 모든 source guard를 전수 전환한 변경이 아니며, 기존 helper의 정규식 리터럴 처리 범위를 넓히지도 않는다.

## 완료한 검증

| 항목 | 실제 결과 |
| --- | --- |
| 변경된 테스트 7개를 `node --test`로 실행 | 26개 통과, 실패 0개, skip 0개 |
| Vite SSR 런타임 확인 | 같은 구역 fragment, 교차 구역 snapshot 분기와 선택 단계 보존 통과 |
| Vite SSR history 동작 확인 | 100개 record 후 live 98, 오래된 두 항목 폐기, execute 호출 0회 |
| `npx --no-install tsc --noEmit` | 통과 |
| `git diff --check upstream/devel...HEAD` | code candidate에서 통과 |
| 신뢰된 CI classifier로 변경 경로 분류 | classified, rust_required=false, native_skia_required=false, frontend_mode=package |

classifier는 추가로 render_required=true, codeql_languages=javascript-typescript를 반환했다. 따라서 Frontend-only는 Render Diff나 JavaScript CodeQL까지 불필요하다는 뜻이 아니다. 최종 원 PR head에서 요구되는 worker 결과 또는 신뢰된 fast-pass 증거를 확인한다.

Rust source는 변경하지 않아 Cargo 전체 회귀와 WASM 재빌드는 반복하지 않았다. 원시 로그 및 임시 산출물은 커밋하지 않았다.

## 원 PR의 실제 CI와 skip 근거

정렬 후 [CI run 33969878202](https://github.com/edwardkim/rhwp/actions/runs/33969878202)의 Build & Test와 preflight가 성공했다. Frontend를 포함한 heavy worker는 이번 head에서 다시 실행되지 않았다.

preflight는 기존 code head `91b70532a241829d22e2e07108664e5219a4a8a7`의 성공 CI를 찾았고, 현재 base 병합 tree가 일치함을 확인했다.

- DETECTED_REASON: direct-source-build-and-test-green:success
- MERGE_TREE_VERIFIED: true
- MERGE_TREE_REASON: current-base-merge-tree-match
- 최종 fast-pass 경로: current-base-update-merge-tree-green

[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33969878193), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33969878196), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33969878206), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33969878091)도 preflight 성공 및 정책상 worker skip으로 완료했다. 이 결과를 이번 head의 모든 worker를 새로 실행한 결과라고 기록하지 않는다.

## 시각 증적 범위

renderer/layout/paint 코드와 문서 fixture는 변경하지 않았다. 특정 HWP/PDF의 페이지 출력 개선을 주장하지 않으므로 별도의 PDF 변환이나 visual sweep 이미지를 만들지 않았다. 편집 이력의 경로·예산 검증은 위의 실제 런타임 및 source contract 결과로 기록한다. 최종 원 PR의 Canvas visual diff가 실행되면 그 실제 결과를, skip이면 그 정책 근거를 CI 증거로 확인한다. 로컬에서 하지 않은 문서 시각 검증을 완료했다고 표시하지 않는다.

## PR #6780 CI 개선의 실제 검증 계획

[PR #6780](https://github.com/edwardkim/rhwp/pull/6780)은 동일 저장소 PR의 Frontend-only 최종 head 증거를 이용한 post-merge 재사용을 추가했다. 최신 지시에 따라 fork 원 PR #5953을 직접 merge하므로, 이 사례의 예상 결과는 같은 저장소 조건에 의한 재사용 거부다. Frontend-only라는 사실만으로 그 신뢰 경계를 우회하지 않는다.

1. 원 PR에 문서 전용 trailing commit을 push하고, 그 정확한 head의 required check, CodeQL 및 필요한 Render Diff와 fast-pass 근거를 확인한다.
2. 승인된 정상 merge commit 경로로 원 PR을 병합한다. #6781을 대신 merge하지 않는다.
3. 실제 merge SHA의 devel push에서 reuse reason과 source_run_id, classifier와 각 worker 결과를 대조한다.
4. 같은 저장소 조건에 의해 reuse=false이면 예상된 제한으로 보고한다. 정상적인 Frontend package 재실행을 재사용 성공이나 전체 worker skip으로 표현하지 않는다.
5. Rust 불필요 판정과 duration 갱신 조건도 따로 확인한다. 실패, 취소 또는 분류와 맞지 않는 실행은 성공으로 간주하지 않는다.

현재 이 post-merge 실증은 미완료다. 이번 사례에서 신뢰 경계 유지가 확인되더라도, 동일 저장소 Frontend-only PR의 재사용 성공까지 실증한 것은 아니다.

## Merge 후 contributor PR comment 계획

[post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. 원 PR #5953의 merge SHA와 devel 검증 결과가 확정된 뒤에만 처리한다.

- 원 PR에 contributor의 기능 변경과 reviewer의 문서 전용 trailing commit을 직접 merge한 사실, merge SHA, 실제 PR/devel CI와 로컬 검증 결과를 한 번 기록한다.
- 기존 comment를 확인해 중복 게시하지 않는다. UTF-8 body file로 게시하고 API로 본문을 재조회한다.
- 이 검토 문서는 merge SHA에 고정한 GitHub 파일 링크로 남긴다. 시각 이미지를 판단 근거로 쓰지 않았으므로 임시 PNG·SVG·JSON·로그를 첨부하지 않는다.
- 원 PR은 직접 merge되므로 별도 close를 반복하지 않는다. contributor fork branch는 보존한다.
- #5769, #6332, #2328, #3416 등의 배경 참조를 포괄적인 이슈 해결 또는 자동 close 지시로 해석하지 않는다.
