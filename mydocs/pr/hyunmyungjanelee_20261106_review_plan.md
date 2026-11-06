# hyunmyungJaneLee PR 5건 누적 검토 계획 — 2026-11-06

## 범위와 현재 단계

사용자 지시: 기여자 hyunmyungJaneLee의 열린 PR을 묶어 처리한다. 현재는 로컬 검토·검증 단계다. 원격 push·댓글·새 PR·merge는 결과와 정확한 범위를 확정한 뒤 승인 범위로 구분한다. 원 PR 5건의 reviewer edwardkim을 각각 지정하고 API로 확인했다.

기준 devel: `170b886f167b705b6e630c5b34bb638d92fae0a9`.
검토 branch: `review/integration-20261106-hyunmyungjanelee`.
검토 worktree: `/home/edward/mygithub/rhwp-worktrees/review-integration-20261106-hyunmyungjanelee`.
공용 Cargo target: `/home/edward/mygithub/rhwp/target/pr-review`. 기본 작업공간과 다른 작업의 #8296 worktree를 보존한다.

base route: `maintainer_general.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`, `rework_and_exceptions.md`, `post_merge.md`.
loaded documents: 위 정본·선택표 및 개발·문서/Git·GitHub 운영·시각 검증 문서. 첫 기여자 문서는 분류를 위해 확인했으며, #8585의 #8601 통합 수용 이력으로 이번 5건을 최초 기여로 단정하지 않는다. GitHub의 FIRST_TIME_CONTRIBUTOR 표시는 실제 수용 이력과 구분한다.

## 원 PR과 순서

| 순서 | PR | 원 head | 원 branch |
| --- | --- | --- | --- |
| 1 | [#8515](https://github.com/edwardkim/rhwp/pull/8515) | `1e16858afba62b58866aa2f7dfca925b9cd720a5` | `fix/textbox-floating-table-offset` |
| 2 | [#8599](https://github.com/edwardkim/rhwp/pull/8599) | `0f2dd6b7fe901344538c1d21473ee67cb0672a37` | `task/8579-partial-table-control-only-para` |
| 3 | [#8608](https://github.com/edwardkim/rhwp/pull/8608) | `5554b6f93c5d105b305d30802ed5bd2436b83b21` | `task/8584-short-row-column-boundary` |
| 4 | [#8631](https://github.com/edwardkim/rhwp/pull/8631) | `b76d7b9312815d6099b97e63923b0a2585fe03cd` | `task/8594-bindata-link-slot` |
| 5 | [#8656](https://github.com/edwardkim/rhwp/pull/8656) | `e635037bb95740382c0ac25c93ba103512c0c90b` | `task/8593-picture-soft-edge-dim` |

모두 `kidsnote/rhwp`의 Open·non-Draft이며 기존 reviewer는 없었다. 5개 원 head의 CI와 개별 current-base merge-tree를 확인했다. 모두 충돌 없이 병합 가능하며 #8656의 마지막 Render Diff도 성공했다. source SHA를 바꾸는 push는 수행하지 않았다.

## 확인할 계약과 검증

1. 최신 devel의 Native·최적화 fresh WASM을 고정해 보관하고, 같은 6개 입력·7쪽의 원 문제가 남았는지 먼저 확인한다.
2. 원 기여 커밋과 원 head를 보존하는 누적 병합으로 후보를 만든다. 누적 충돌이 생기면 임의로 해결하지 않고 구체적인 해결 방침을 제시한다.
3. #8515는 글상자 inner area·문단 원점·offset/margin → 표 배치 → 뒤 문단/개체 흐름을 확인한다. #8585의 TAC 줄 소속·가운데 정렬과 #8122의 저장 줄 없는 세로 정렬을 정상 대조로 유지한다.
4. #8599는 전체/분할 셀의 control-only 문단 소유와 일반 표·부분 표의 문단 원점, 뒤 텍스트가 있는 경계를 실제 출력으로 확인한다. 기존 #7524/#8585와 부분 표 원점·소유 보정 이력을 먼저 대조한다.
5. #8608은 공통 경계 결과의 측정·프레임·paint 소비를 추적한다. 96HU 최소 간격의 독립 근거, Storage 자원 예산을 우회하는 새 할당과 행×셀 순회의 실제 노출 여부를 확인한다. 단순 관측 좌표나 기대값으로 구현을 정당화하지 않는다.
6. #8631은 HWP5 레코드 순번과 stream id의 기존 계약을 먼저 대조하고 HWP/HWPX 저장·재열기, 외부 그림 채움·일반 내장 그림을 확인한다.
7. #8656은 그림 효과 flag/필드 길이를 사양·실제 저장 데이터와 대조한다. 작성자의 화면 PNG 대비 수치를 독립 Print PDF 통과로 취급하지 않는다. Print가 화면의 약 75%로 출력되는 차이는 Desktop Print로 교차 확인한다. 한컴 2020 Print 요청은 대기열 8건 포화로 접수되지 않아 아직 미검증이며 다른 작업을 계속한다.
8. 동일 원본·독립 출력·실제 Native/fresh Canvas의 전쪽 TSV를 산출하고, 주요 그림·표 경계와 미세 간격을 직접 판독한다. parser 영향의 실제 SVG 소비와 저장 재열기도 확인한다. 정식 글꼴 예외는 미리 가정하지 않는다.
9. 측정 결과가 선행 조건을 충족한 뒤 신규 회귀 첫머리의 실제 측정/시각과 source SHA를 보완하고 정답지 쪽수 원장을 등록한다. 원 assertion·올바른 기존 baseline을 임의로 바꾸지 않는다.
10. 후보 source가 확정되면 필수 Rust lint·focused·전체 회귀·Native 공식 3종·새 샘플 보안·383쪽 편람을 한 묶음으로 완료한다. 원 head CI를 새 통합 후보의 통과로 대체하지 않는다.
11. 최종 판정과 보정·실행 증거는 원 PR별로 남긴다. 이 계획 또는 CI 녹색만으로 원격 수용을 선언하지 않는다.

## 현재 확보한 자료

입력·기준 PDF는 각 원 commit에 포함된 공개 합성 대조군이며 실제 바이트 해시를 고정했다. 작성자의 사내 코퍼스 집계는 참고 주장으로만 구분하고 비공개 원문이나 식별 목록을 요구·공개하지 않는다. 로컬 시각 검증의 글꼴 프로그램은 검사에만 쓰고 저장소에 추가하지 않는다.

기준 Native 빌드와 6개 입력 7쪽 비교를 완료했다. 5개 원 문제는 최신 devel에서도 남아 있으며 r4 대조군은 별도로 구분한다. 최적화 기준 WASM 빌드도 완료했다. 96HU는 PDFium으로 독립 PDF의 벡터 선 간격 약 0.96pt를 확인했으며, 일반 적용 근거는 추가 대조한다. 이 문서는 최종 검증 결과가 아니다.
