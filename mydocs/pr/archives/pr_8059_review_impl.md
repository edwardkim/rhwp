# PR #8059 구현·통합·후속 처리

## 현재 단계

로컬 필수 검증과 정확한 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`의 CI 성공을 확인했습니다. 원 PR source는 수정하지 않고 통합 PR #8148의 문서 trailing으로 기록합니다. 남은 조건은 trailing head의 CI·merge·후속 처리입니다.

## 기준과 범위

- 원 PR #8059: `1367ff84693d4745f40efe5889b56835c2793688`; 현재 기능 tip `1367ff84693d4745f40efe5889b56835c2793688`.
- base `a6e9d18fc1061318055b4eb348daec201427ea03`; 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7` / 실제 검증 runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4` / 새 검사 등록 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`.
- 기본 경로: collaborator 매개 외부 PR. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge. 같은 통합 PR의 trailing 기록에 포함하며 통합 PR 번호용 별도 review나 추가 문서 PR은 만들지 않습니다.
- 원 변경의 실제 범위·전쪽 TSV·직접 판독·기존 이관·최종 판정은 같은 번호의 review와 공통 검증 원장에 연결합니다.

## 원 기여의 현재 대응

| 원 source SHA | 현재 통합 SHA | 실제 제목 |
| --- | --- | --- |
| `717679359e6e128f26610727cf7c04d2f8ac6d5f` | `bf4fc67e54b1cf8a370de1780cf34898faf19fee` | fix(table): 셀 안 글자처럼 취급 중첩 표를 행 사이에서 나누지 않는다 (#7870) |
| `fa10ce00d855bb24920448dc5d787a0604ebf247` | `c94bff7af0923f707ee09e75230d57ffc334b152` | fix(table): 행의 칸들이 같은 저장 높이에서 함께 되감는 쪽 경계는 닫는 줄의 줄간격을 예약하지 않는다 (#8060) |
| `b2d53202610e82325075b90fd1eefbf8fac1b162` | `3026918dd33a3c0f6d4110f30bef61b16ad59490` | fix(table): 칸마다 닫는 줄 높이가 달라도 함께 되감는 저장 쪽 경계를 알아본다 (#8060) |
| `0d76c5658bcf37ddf65fc395a9146f40bb3ee08e` | `27de779ddcd57df31a5695953204239168110eb1` | fix: 쪽 경계에서 나뉜 재조판 행의 첫 조각은 선언 행 높이를 차지 |
| `ac610ad6f9cc75e4cd60794ffb670086179512bf` | `3eeb07f56c99f3942e2945a1f834500ec35bda4a` | style: rustfmt 정리 |
| `804c7b5a9973e1674893974194b1e13988c83e9f` | `fbf14b7e3babe5dd88c11d4dfc7bf4b3fa1eaff4` | test: 71352 의 devel 유래 본문 넘침 1건을 근거와 함께 baseline 에 등록 |
| `3efc589dfbdf6183538cfe3bb9e67979904ec604` | `95cca2ed240e79cda0260be1d1afa8010ddda083` | test: 71352 넘침 baseline 근거에 추적 이슈 #8095 연결 |
| `da1d2280d3f1f901a6aadd84ce14ba7c62cd9a71` | `f210f6796de5172251760136b2cf4800813745be` | fix(table): 쪽 경계에서 되감긴 셀 저장 프레임에서 중첩 표 host 목표를 읽는다 (#8095) |
| `1367ff84693d4745f40efe5889b56835c2793688` | `0a8618dc8e6eb4f6b4d3a34655442a9c1816d049` | docs: 71352 11·12쪽 이어받는 조각 수정 전후 대조 (#8095) |

## 보정과 검증의 실행 순서

1. 최신 devel 위에서 기여를 먼저 적용하고 source 124개와 공통 보정 순서를 고정했습니다. source·현재 SHA·커밋 제목 전체는 [실제 통합 순서 원장](../../../mydocs/pr/assets/planet6897_20261018/documents/integration_commit_order.md)에 연결합니다.
2. 원본·독립 Print·source별 claim을 대조해 공통 컷 소유·여백·줄/표 흐름·수식/폰트/편집을 보정했습니다. 구체적인 생산/소비 코드·반례·전후 관측은 통합 계획과 문서별 증거를 재사용합니다.
3. runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4`의 전체 로컬 검증과 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`의 새 5건·lint를 완료하고 code `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`로 Open PR #8148을 제출했습니다. [실제 검증 명령과 결과](../../../mydocs/pr/assets/planet6897_20261018/documents/validation_commands.md)와 [같은 번호의 최종 검토](../../../mydocs/pr/archives/pr_8059_review.md)에 실제 결과를 연결합니다.
4. 정확한 code head의 [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39705756548) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39705756552) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39705756369) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39705756537) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39705756582) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39705756438)가 모두 성공했습니다. 원 PR 27건의 review/implementation와 오늘할일을 같은 PR의 문서 trailing으로 기록하며, 그 최신 head의 CI·병합 결과·링크를 push 전후 확인합니다.
5. 정확한 trailing head로 merge한 뒤 merge SHA 증거 comment·원 PR open/closed 처리·완료 이슈/하위 이슈 정리·devel 동기화·branch/worktree/output 정리를 수행합니다. 공유 target·사용자 자료·기여자 fork는 보존합니다.

## 보정·되돌림 경계

- 이 원 PR의 기여 SHA는 위 표로 고정하고 공통 보정 및 전후 관측은 [통합 계획·생산/소비 경로·보정 기록](../../../mydocs/plans/pr_review_planet6897_20261018.md)에 연결합니다. 문서별 임계값 완화나 source branch 강제 재작성으로 결과를 맞추지 않습니다.
- 원 PR source는 유지합니다. 미병합 통합 후보의 결함은 해당 공통 계층의 추가 보정으로 처리하고, 이미 통합한 변경을 되돌릴 때는 표의 source SHA 및 후속 공통 보정 의존성을 함께 검토합니다.
- 최종 판정과 GitHub approve/push/merge/close를 구분합니다. 사용자의 기존 승인 범위 안에서 실제 최신 상태와 필수 결과를 확인해 후속 처리를 이어갑니다.
