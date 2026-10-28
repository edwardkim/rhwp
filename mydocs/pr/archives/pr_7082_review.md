---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7082 — HWP3 도형의 HWP5 storage 비트 보존 검토

**최종 판정: 메인터너 보정 후 수용 가능.** HWP3 선·다각형의 HWP5 저장 필드에 필요한 storage 비트가 0으로 나가 한컴 개방을 막던 부분을 수정했다. 실제 원본·기준 HWP5·before/후보 HWP5·한컴 PDF를 메인터너 커밋으로 보존했다.

이 문서는 로컬 검증에 따른 수용 판정이다. 최종 통합 head의 GitHub CI·MERGEABLE/CLEAN 확인과 실제 merge 전에는 원 PR을 close하지 않는다. 원 PR의 녹색 CI를 통합 후보의 Full CI로 대신하지 않는다.

## 대상과 코드 계약

- 원 PR: [#7082](https://github.com/edwardkim/rhwp/pull/7082), planet6897 / reviewer jangster77.
- 원 head: `85b8726dd175b25f5efb743c009e8da81e146d1b`. 기준 devel: `897c6a3d8d7559d314bf863c93bbe28c0d65e945`.
- cherry-pick -x 대응: `cabdfb10f (#7075 중복 제외)`. 통합 runtime 보정 head: `568b210d9`.
- 주 작업공간 branch: `review/planet6897-20260913`. 기본 collaborator_external_pr, 보조 intake_and_review·local_validation·multi_pr_update_branch·visual_fixture_evidence.
- 관련 issue: #4680 부분 해결.

HWP5 변환 시 도형의 flip 필드가 0일 때 storage bit 0x80000을 채우고 text box에는 0x1000000도 반영한다. 이 정상화 함수는 HWP3/HWPX→HWP5 공통 경로이므로 두 출처의 0값 도형에 적용된다. 기존 nonzero 값은 보존하고 HWP5 원본의 직접 저장 경로는 이 함수를 타지 않는다. non_hwp3_source_is_untouched 테스트가 실제로 확인하는 반례는 FileFormat::Hwp이며 모든 비-HWP3 포맷을 포괄하는 검사는 아니다. 한컴의 다른 회전/반전 비트를 통째로 복제하지 않으며 저장 표현의 최소 필요 비트만 보정한다.

## 실제 검증과 한계

BodyText의 top-level·group-child SHAPE_COMPONENT를 각각 맞는 offset으로 읽었다. 177개 선+13개 다각형에서 before는 storage 0, 후보는 0x80000 또는 textbox 포함 0x1080000, 독립 한컴은 모두 해당 비트를 가진다. 한컴에는 0xb0000/회전·반전 값 등 추가 비트가 있으므로 필드 전체 동일성을 주장하지 않는다. #7075와 합친 후보의 한컴 326쪽 PDF 생성 성공으로 개방 경로를 확인했다.

최종 통합 전체 nextest **Summary [ 457.270s] 9565 tests run: 9565 passed (7 slow), 46 skipped**, 세 Clippy(native/WASM/workspace all-targets), workspace build, manifest/unit-tier check, Native Skia 3종, WASM build/parity를 통과했다. [공통 검증 기록](pr_7073_review_impl.md)에 exact command·시간·바이너리·24개 입력 provenance를 기록했다. 원 작성자의 실측과 이번 Mac 실행을 구분한다.

한컴 출력 326쪽은 개방 성공의 증거다. 원본과 페이지·도형 방향·문자열이 전수 동일하다는 증거가 아니다. #4680의 pagination 등 잔여 문제는 유지한다.

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
| 측정·배치 일관성 | 비해당 — 저장 format 필드 변경이며 renderer 기하 계산을 수정하지 않는다. |
| 줄 소속·점유 높이 | 비해당 — 문단 줄 높이·소유 관계 변경 없음. |
| 사례·증거 독립성 | 충족 — committed 원본/한컴 출력과 후보를 구분하고 집중·전체 회귀를 실행 |
| 기준값 변경 | 비해당 — baseline/golden/허용치 변경 없음. |
| 주장·검증 범위 | 충족 — 위 잔여 문제와 미실행 작성자 전수 실측을 명시하고 실제 재검증 범위에 한정 |

## Merge 후 원 PR comment 계획

기여에 감사 인사를 남기고 원 head `85b8726dd175b25f5efb743c009e8da81e146d1b`의 반영, 통합 PR 번호·merge SHA와 메인터너 보정을 알린다. [review](pr_7082_review.md)는 `https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/archives/pr_7082_review.md`로 고정한다. #4680은 부분 해결 참조로 유지하며 OPEN 상태를 확인한다. 통합 merge가 확인된 뒤 원 PR을 close하며 contributor fork branch는 삭제하지 않는다.
