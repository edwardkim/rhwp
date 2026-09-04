# Green CI 외부 PR batch - 통합·검토 계획

## 적용 경로

```text
기본 경로: collaborator_external_pr.md
보조 경로: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 선정 결과

2026-09-04 열린 PR 목록에서 요청대로 #6713을 제외했다. draft #6702, #6685, #6670,
#5953과 재조회 시 `CONFLICTING/DIRTY`였던 #6637도 제외했다. #6683, #6690, #6698,
#6703, #6705, #6709, #6710만 최신 source head의 CI, CodeQL, Render Diff, Adapter
inter-diff, Proptest가 성공 또는 정책상 skip이었다.

## 통합 이력

기준은 `upstream/devel` `009e30fe1`이며 브랜치는
`review/green-ci-batch-20260904-full`이다.

| 원 PR | source commit | 통합 commit |
| --- | --- | --- |
| #6683 | `e5f385c17`, `e5dde4373` | `dd8ca73a2`, `4c333ab94` |
| #6690 | `f9d76b11c`, `c37925771` | `eb84bbbc7`, `9f0455b6f` |
| #6698 | `7e47ef691` | `c0145ec66` |
| #6703 | `0bb2f2b08`, `219868e86` | `ceadaf94a`, `2dd41febf` |
| #6705 | `0bfcd04cd`, `05325df7c` | `7b15d9582`, `dda7902e5` |
| #6709 | `df9c6c612`, `36b550089` | `e6b9a3ed5`, `ffd47191e` |
| #6710 | `ceb649a2d`, `4a1eb7c27` | `c340bd7a8`, `61cd71fb9` |

각 PR의 모든 source commit을 원래 순서로 `git cherry-pick -x` 적용했다. source head만
적용하면 #6683과 #6690의 선행 동작 commit을 빠뜨리므로 사용하지 않는다.

## 메인터너 보정과 fixture 등록

- `872f3d4c5`은 #6202 그림 이동과 #5057 HWPX export/ZIP 처리의 fixture 발견 뒤 오류를
  성공으로 삼키지 않도록 fail-closed로 바꾼다.
- `samples/issue6202/`와 `samples/issue5057/`에 원본 HWP, SHA-256 manifest, 한국어
  README를 등록한다. 테스트는 환경 변수와 Windows 개인 경로를 제거하고 이 정식 fixture를
  반드시 읽는다.

## 남은 수용 게이트

1. 새 HWP fixture의 IR field sweep 및 overflow-cell 원장 절차를 수행한다.
2. 전용 target에서 통합 head Rust lint와 관련 focused test를 실행한다.
3. 통합 head에서 한컴 기준 PDF와 representative visual asset을 만들고 직접 판정한다.
4. 위 결과를 사실대로 review·오늘할일에 반영한 trailing commit 뒤, 작업지시자 승인 후에만
   push 또는 PR 생성을 진행한다.

이 단계에서 GitHub comment, close, push, PR 생성, approve, merge는 수행하지 않았다.

## 2026-09-04 정식 fixture 통합 검증 결과

정식 fixture 등록 후 전체 integration test를 분리 target에서 실행했다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

실행은 종료 코드 `101`로 실패했다. `ir_field_sweep_baseline`이 신규 fixture 두 건에서 다음 HWP5 재저장 차이를 검출했기 때문이다.

- `issue5057/21484591-gimcheon-sewage-ordinance.hwp`: `list_header_width_ref 0 -> 367`
- `issue6202/156483689-turmeric-industry-standardization.hwp`: `list_header_width_ref 0 -> 35`

baseline은 의도된 정규화인지 확인하지 않은 상태에서 갱신하지 않았다. native-skia release 빌드로 생성한 PNG·SVG, N-up 조건, 렌더 진단 및 직접 관찰 결과는 [통합 시각 sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)에 분리 기록했다.
