---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7075 — HWP3 홀수 쪽 시작의 유효 control 저장 검토

**최종 판정: 메인터너 보정 후 수용 가능.** HWP3 control 21 kind=0을 존재하지 않는 HWP5 control id 0x15로 저장하던 경로를 PageNumCtrl의 Odd로 보존했다. #7082와 합친 후보는 한컴에서 실제 열어 PDF 출력할 수 있었다.

이 문서는 로컬 검증에 따른 수용 판정이다. 최종 통합 head의 GitHub CI·MERGEABLE/CLEAN 확인과 실제 merge 전에는 원 PR을 close하지 않는다. 원 PR의 녹색 CI를 통합 후보의 Full CI로 대신하지 않는다.

## 대상과 코드 계약

- 원 PR: [#7075](https://github.com/edwardkim/rhwp/pull/7075), planet6897 / reviewer jangster77.
- 원 head: `2f41f46f1fd0a268a324e7187edafc732d739f27`. 기준 devel: `897c6a3d8d7559d314bf863c93bbe28c0d65e945`.
- cherry-pick -x 대응: `dde60e9db`. 통합 runtime 보정 head: `568b210d9`.
- 주 작업공간 branch: `review/planet6897-20260913`. 기본 collaborator_external_pr, 보조 intake_and_review·local_validation·multi_pr_update_branch·visual_fixture_evidence.
- 관련 issue: #4680 부분 해결.

HWP3 kind=0의 원 의미를 이미 있는 PageNumCtrlOdd IR로 옮긴다. kind=1의 숨김 및 다른 kind 처리는 유지한다. HWP5 serializer는 유효 pgct control과 flag 2를 출력한다. 샘플 ID·문서명 분기가 없다.

## 실제 검증과 한계

독일 법령체계 실물 저장본에서 invalid control 0x15가 4개에서 0개로 줄고 pgct payload 7463677002000000이 4개 생겼다. 독립 한컴 HWP5에도 같은 payload 4개가 있다. 기준 devel 저장본은 MCP 900초 timeout으로 FAILED였고, 통합 후보는 61.961초에 326쪽 PDF를 출력했다. timeout을 즉시 open=false로 과장하지 않는다. 한컴 개방 성공은 #7082 포함 통합 결과이지 이 PR 단독 결과가 아니다.

최종 통합 전체 nextest **Summary [ 457.270s] 9565 tests run: 9565 passed (7 slow), 46 skipped**, 세 Clippy(native/WASM/workspace all-targets), workspace build, manifest/unit-tier check, Native Skia 3종, WASM build/parity를 통과했다. [공통 검증 기록](pr_7073_review_impl.md)에 exact command·시간·바이너리·24개 입력 provenance를 기록했다. 원 작성자의 실측과 이번 Mac 실행을 구분한다.

원 작성자가 보고한 원본 264쪽과 후보 출력 326쪽은 일치하지 않는다. 264쪽은 이번 MCP가 반환한 page count가 아닌 이슈의 실측 보고다. 전체 HWP3 38건 한컴 성공률·모든 쪽 시각 일치를 재검증한 것으로 주장하지 않는다.

통합 code candidate `d3dfa17e0f1cb454793b16c19628405e6f02b535`의 [Full CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34741218879), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34741218885), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34741218798)가 완료·success임을 확인한 뒤 이 review-only 기록을 추가했다. 최종 trailing head aggregate는 push 뒤 별도로 확인한다.

이 저장 형식 수정에는 독립 한컴 레코드 대조와 실제 개방/PDF 생성이 직접 증거다. 326쪽 PDF 전수 시각 일치를 수행했다고 기록하지 않는다. 통합 renderer 영향은 다른 PR의 17쪽 비교 및 OVR5로 별도 검증했다.

## 검증 파일의 commit 포함

| 파일 | 마지막 파일 commit | SHA-256 |
| --- | --- | --- |
| [`pdf/german-legislative-system-candidate-2020.pdf`](../../../pdf/german-legislative-system-candidate-2020.pdf) | `2fabd879a` | `f4a4b40f1c9f6938b17612a334ec61392c5b81eb38d37900040b6455470b6ffd` |
| [`tests/fixtures/issue_4680/german-legislative-system-before.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-before.hwp) | `2fabd879a` | `c3926c32c65f29968b7134e175d176b99f2bce37f7205057cf2df16d11e72679` |
| [`tests/fixtures/issue_4680/german-legislative-system-candidate.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-candidate.hwp) | `2fabd879a` | `78d1bf6cc4480619d2044466c6f3742fc2bac7da4b2b6cb9abbdfc158ec63735` |
| [`tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp) | `2fabd879a` | `84bcec55e53692a935dafec0ff509f878b278c9f833e59a3f4c9be1a444444b6` |
| [`tests/fixtures/issue_4680/german-legislative-system.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system.hwp) | `2fabd879a` | `543d67cdb4d84b876949cef4f4ec7435b99716d6fa7feebde57b024c88d40559` |

각 파일을 `git show HEAD:<path>`와 바이트 대조했다. 원본·독립 한컴 기준·후보 검사 산출은 같은 통합 PR의 blob으로 재현할 수 있다. 변환 산출을 독립 oracle로 사용하지 않았다.

## 공통 조판 원칙 검토

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거·일반성 | 충족 — 위 source 계약·한컴 독립 실측에 근거하며 문서 ID 분기·clamp 없음 |
| 측정·배치 일관성 | 비해당 — 이 변경 자체는 parser control 의미·serializer 호환성 변경이다. |
| 줄 소속·점유 높이 | 비해당 — renderer의 줄 높이·점유 로직 변경 없음. |
| 사례·증거 독립성 | 충족 — committed 원본/한컴 출력과 후보를 구분하고 집중·전체 회귀를 실행 |
| 기준값 변경 | 비해당 — baseline/golden/허용치 변경 없음. |
| 주장·검증 범위 | 충족 — 위 잔여 문제와 미실행 작성자 전수 실측을 명시하고 실제 재검증 범위에 한정 |

## Merge 후 원 PR comment 계획

기여에 감사 인사를 남기고 원 head `2f41f46f1fd0a268a324e7187edafc732d739f27`의 반영, 통합 PR 번호·merge SHA와 메인터너 보정을 알린다. [review](pr_7075_review.md)는 `https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/archives/pr_7075_review.md`로 고정한다. #4680은 부분 해결 참조로 유지하며 OPEN 상태를 확인한다. 통합 merge가 확인된 뒤 원 PR을 close하며 contributor fork branch는 삭제하지 않는다.
