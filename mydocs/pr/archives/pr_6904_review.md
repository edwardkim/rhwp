# PR #6904 검토: 검증된 fork PR의 post-merge CI 재사용

## 판정: #6901 해결 완료 판정은 머지 보류

PR #6904 자체는 이미 병합되었다. 이 판정은 병합 취소가 아니라 Issue #6901의 전체 요구사항 충족 여부에 대한 판정이다. 원 fork PR #6880을 실제 병합해 CodeQL 재사용은 확인했지만, CI의 필수 duration artifact 게시 조건에 fork 제한이 남아 전체 재사용 완료로 수용할 수 없다. #6901은 OPEN으로 유지한다.

## 확인된 문제와 최소 후속 보정

1. **CI duration artifact 생산 경로가 fork를 제외한다.** `.github/workflows/run-nextest-archives.yml:120`의 게시 단계는 `head.repo.full_name == github.repository`인 PR만 허용한다. [최종 PR Archive B](https://github.com/edwardkim/rhwp/actions/runs/34241531638/job/102116007897)는 `target_durations=9 case_durations=484`를 계산했지만, 해당 run의 13개 artifact에는 B/C/D duration artifact가 없다. 따라서 [devel CI verifier](https://github.com/edwardkim/rhwp/actions/runs/34243396600/job/102119067399)의 `candidate-full-lane-evidence-unavailable` 및 Full lane은 누락된 증거에 대한 fail-closed 결과다. 후속 코드 변경에서는 fork 게시와 소비 양쪽에 기존 repository/PR/head/base/merge-tree 신뢰 검증을 유지해야 하며, 소비자 검사를 완화하는 방식으로 해결해서는 안 된다.
2. **trailing 문서의 PR fast-pass 후보 탐색에도 별도 제한이 관측됐다.** 성공한 base 정렬 merge head `58e41882137b154570c7056dc3051024a8f7168d`가 있었지만 [최종 PR CodeQL preflight](https://github.com/edwardkim/rhwp/actions/runs/34241531683/job/102112708677)는 이전 취소 head `df1db1f20e113460a0ce8d042f4a1171014a7553`의 `Analyze (rust):cancelled`를 이유로 fast-pass를 거부했다. [CI preflight](https://github.com/edwardkim/rhwp/actions/runs/34241531638/job/102112691694)도 `no-green-build-candidate`였다. 이는 post-merge 재사용과 별개인 PR 후보 탐색 경로의 관측 결과이며 원인 전체를 확정하거나 수정 완료로 주장하지 않는다.
3. **Adapter/Proptest 재사용은 입증되지 않았다.** 두 devel verifier는 `review-tail-candidate-merge-tree-evidence-unavailable`로 거부했고 실제 worker를 실행해 성공했다. 성공한 실행을 재사용 성공으로 바꾸어 기록하지 않는다.

이번 옵션 2 PR은 기록 전용이다. 위 코드 보정이나 새로운 정책 예외를 포함하지 않는다.

## 대상과 provenance

- 개선 PR: [#6904](https://github.com/edwardkim/rhwp/pull/6904), 관련 이슈 [#6901](https://github.com/edwardkim/rhwp/issues/6901).
- 개선 코드 head: `6abd02d58e231bb0661c73c687624893a1485846`.
- 개선 merge SHA: `8d0fd9e34c4bcfaf38e4ef7c3d215fd96fe4af58`.
- 실제 검증 대상: [#6880](https://github.com/edwardkim/rhwp/pull/6880), `planet6897/rhwp`의 `fix/6860-float-anchor-line-host-text` 원 fork PR. 최초 계획의 #6886 대신 사용자 지정 #6880을 사용했다.
- 메인터너 보정: `df1db1f20e113460a0ce8d042f4a1171014a7553`.
- 최신 base 정렬 코드 head: `58e41882137b154570c7056dc3051024a8f7168d`, base `c35e47ae4e647277f1d85d4fc570bac0728b655d`.
- 리뷰·오늘할일 trailing 최종 head: `44c5e50b83ef95e180d6c386b19e746e4dbe96bf`.
- 실증 merge SHA: `dce069537db81261b9c69963c960ee29e0f06d61`, 병합 시각 2026-09-09 00:13:43 KST.
- 리뷰어 승인, 최종 head의 CI, `MERGEABLE` 및 `CLEAN` 확인 후 match-head 일반 merge commit으로 병합했다.

## 실제 CI 증거

| 구간 | 실행 | 결과 |
| --- | --- | --- |
| #6904 PR | [CI](https://github.com/edwardkim/rhwp/actions/runs/34227762601), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34227762535) | 성공 |
| #6904 devel | [CI](https://github.com/edwardkim/rhwp/actions/runs/34229215974), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34229215923) | 성공. enforcement 변경 자체의 Full lane은 정상이며 fork 재사용 실증이 아님 |
| #6880 정렬 코드 head | [CI](https://github.com/edwardkim/rhwp/actions/runs/34239219541), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34239219556) | 성공 |
| #6880 최종 trailing head | [CI](https://github.com/edwardkim/rhwp/actions/runs/34241531638), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34241531683) | Full lane 성공. CI A-D 및 필수 worker, CodeQL 3개 언어 분석 성공 |
| #6880 최종 head 부가 gate | [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34241532214), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34241531955), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34241531249) | 성공, 정책에 따른 worker skip |
| #6880 devel CI | [CI attempt 2](https://github.com/edwardkim/rhwp/actions/runs/34243396600/attempts/2) | 성공. Full lane 및 duration refresh 성공. attempt 1은 Archive B 다운로드 HTTP 403 실패 |
| #6880 devel CodeQL | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34243396712) | 성공. 최종 PR CodeQL 재사용, 분석 worker skip |
| #6880 devel 부가 gate | [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34243396542), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34243396570), [Close Issues](https://github.com/edwardkim/rhwp/actions/runs/34243396198) | 성공. Adapter/Proptest worker는 실제 실행 |

[CodeQL verifier](https://github.com/edwardkim/rhwp/actions/runs/34243396712/job/102119066431)의 실제 결과는 `reuse=true reason=review-tail-final-head-green-pr-workflow-reused source_run_id=34241531683 refresh_duration_data=false`였다. CI와 CodeQL을 묶어 모두 재사용했다고 주장하지 않는다. 이 merge SHA의 devel Render Diff 실행은 확인되지 않아 실행 성공으로 기록하지 않는다.

[실패한 Archive B 다운로드](https://github.com/edwardkim/rhwp/actions/runs/34243396600/job/102121810747)는 테스트 실행 전 `Failed to ListArtifacts`, HTTP 403으로 중단됐다. 실패 job만 재실행했으며 [attempt 2 Archive B](https://github.com/edwardkim/rhwp/actions/runs/34243396600/job/102124912922), [aggregate](https://github.com/edwardkim/rhwp/actions/runs/34243396600/job/102126531977), [duration refresh](https://github.com/edwardkim/rhwp/actions/runs/34243396600/job/102126531996)가 성공했다. 제품 코드 수정으로 해결한 실패가 아니다.

## 로컬 계약 검증과 신뢰 경계

- 이번 실증 작업에서 JS trusted-postmerge 계약 120건 및 Python `test_trusted_postmerge*.py` 12건이 통과했다. 과거 #6904의 Python 228건 결과와 실행 범위가 다르다.
- base 이동 전에는 event base/head와 PR merge ref 불일치로 merge-tree artifact를 게시하지 않는 fail-closed 동작도 확인했다. 해당 오래된 CI/CodeQL은 base 정렬을 위해 취소했으며 제품 결함으로 분류하지 않는다.
- 정렬 후 fork repository/PR/head/base/tree를 포함하는 artifact가 생성됨을 확인했다. 증거 생성 자체만으로 duration artifact 및 모든 workflow 재사용이 충족되는 것은 아니다.
- 문서 전용 후속 변경에는 Cargo/WASM/제품 회귀 테스트를 반복하지 않는다. 제품 검증과 시각 증거는 [#6880 리뷰](pr_6880_review.md)에 기록된 범위만 인용한다.

## 옵션 2 후속 처리 및 코멘트 계획

이 문서와 [오늘할일](../../orders/20260909.md)을 최신 devel 기반 문서 전용 PR 한 건으로 병합한다. 해당 최종 head 및 merge SHA의 CI가 성공한 다음 후속 코멘트와 소유 브랜치 정리를 수행한다. #6901은 닫지 않는다.

- #6904에는 [기존 후속 댓글](https://github.com/edwardkim/rhwp/pull/6904#issuecomment-5585845686)을 수정하여 예정된 #6886 검증을 실제 #6880 결과로 갱신한다. 동일 결과를 중복 등록하지 않는다.
- #6901에는 실증 merge SHA, CodeQL 재사용 성공, CI duration publisher 제한, 실제 CI 링크와 이 기록을 남긴다. 완료 및 close를 주장하지 않는다.
- #6880과 #6860에는 실제 병합·검증 결과와 #6880 리뷰의 시각 범위를 남긴다. [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment)를 포함하고, 아래 두 PNG를 merge SHA 고정 URL로 직접 표시한다. #6879는 별개 범위로 닫지 않는다.

![#6880 1쪽 시각 대조](https://raw.githubusercontent.com/edwardkim/rhwp/dce069537db81261b9c69963c960ee29e0f06d61/mydocs/pr/assets/pr_6880_maintainer_20260908/review_001.png)

![#6880 2쪽 시각 대조](https://raw.githubusercontent.com/edwardkim/rhwp/dce069537db81261b9c69963c960ee29e0f06d61/mydocs/pr/assets/pr_6880_maintainer_20260908/review_002.png)

대조 범위는 한컴 재저장 HWPX와 그 기준 PDF의 2쪽이다. flagged 0/2, 평균 픽셀 일치율 91.02424%, 평균 잉크 차이 proxy 8.99822%이며 사람의 정확도 점수가 아니다. 원래 잘라낸 입력의 3쪽과 전체 원본 문서의 모든 페이지가 해결됐다고 확대하지 않는다. 표 하단 높이와 글꼴 픽셀 차이는 남아 있다. 임시 PNG/SVG/JSON 및 로그는 추가하지 않는다.
