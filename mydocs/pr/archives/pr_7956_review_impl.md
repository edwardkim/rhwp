# PR #7956 구현·통합·후속 처리

## 현재 단계

로컬 필수 검증과 정확한 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`의 CI 성공을 확인했습니다. 원 PR source는 수정하지 않고 통합 PR #8148의 문서 trailing으로 기록합니다. 남은 조건은 trailing head의 CI·merge·후속 처리입니다.

## 기준과 범위

- 원 PR #7956: `6bcefd1ac368ad81dca81239a1e8ee6e1f2ae092`; 현재 기능 tip `6bcefd1ac368ad81dca81239a1e8ee6e1f2ae092`.
- base `a6e9d18fc1061318055b4eb348daec201427ea03`; 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7` / 실제 검증 runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4` / 새 검사 등록 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`.
- 기본 경로: collaborator 매개 외부 PR. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge. 같은 통합 PR의 trailing 기록에 포함하며 통합 PR 번호용 별도 review나 추가 문서 PR은 만들지 않습니다.
- 원 변경의 실제 범위·전쪽 TSV·직접 판독·기존 이관·최종 판정은 같은 번호의 review와 공통 검증 원장에 연결합니다.

## 원 기여의 현재 대응

| 원 source SHA | 현재 통합 SHA | 실제 제목 |
| --- | --- | --- |
| `8e257ddba86fdf45e20434272e590903ad1f3f45` | `4cb8c1e1c287c4bdfab6ce3349a2b674dae9455e` | 수정(조판): 겹친 줄 상자의 한 걸음을 저장 프레임 경계로 세지 않는다 (#7840) |
| `a264f1dd94059116e8c52c3ca06c220a3a310721` | `eab188ed7c1e435387acb5bc514bde6e8a675f82` | 수정(조판): 이어받는 저장 프레임의 측정도 배치처럼 저장 vpos 로 스냅한다 (#7840) |
| `63c39314e348c0a4fd85f93dcf0c1696b4e3f492` | `a187758b9791cb21273754c953ed6b0a321b8c05` | 수정(조판): 이어받는 표 조각의 바깥 위 여백을 한 번만 연다 (#7840) |
| `a8359e632e6f938672c3af88d5dee865e5fe8216` | `5592f8450da78e5ce9e28b2a646da2f7aae7845f` | 수정(조판): 저장 줄의 글자처럼 취급 개체도 글줄과 같은 줄 들여쓰기를 받는다 (#7840) |
| `f80c40045d391c3c0e88e01b6c0f1d8f73f04416` | `634585347b1538e1c74090683bba4fc5b490eaea` | 수정(조판): 저장 사다리가 전진을 증명한 떠 있는 개체 앵커 줄은 축소하지 않는다 (#7840) |
| `fdee27aa366f1d6f4b6699d11f6b285280decb30` | `a64840c344ac7c489c73e50eb66fd1a0d897decb` | 수정(조판): 미룬 위아래 그림 띠는 저장 사다리가 품은 host 글줄을 다시 세지 않는다 (#7840) |
| `7dab69e806a310b6fda9235f8a812bc50c4fcff0` | `13722c1ec37ce60cb33e30212a5d2bdc634ee8e6` | 수정(조판): 상자에 드는 글자처럼 취급 개체 뒤 공백은 줄 끝에 건다 (#7840) |
| `db7f28eba212dbc309c76695b5e662f4e83fb277` | `4048c3f240dce0566f42a4cfcfc8d42e949852bb` | 수정(조판): 저장 줄 없는 머리말/꼬리말 문단도 개체를 담는 줄 상자로 배치한다 (#7840) |
| `dfafbbbf50c9c2cf778174aa3c7c208e99e91c8b` | `56bd582b24f6cf1bacc98b7eab1e345a60f9114c` | docs: #7840 대표 시각 증적(한컴 PDF 대조 review·overlay) |
| `6b85f52721dae2958e9391e7d665e54a32594849` | `62156d2344a69dfe1437c2b63e4c53ca06f7ca52` | test: 156465345 글자 겹침을 devel 회귀 #8095 로 이관 |
| `612926556f05def642d54e5942ac7930ec5c8135` | `8546eba91dcca76cde801a7ba441cdc66b775782` | fix(typeset): TAC 표 줄 뒤 같은 저장 프레임 글줄의 흐름을 조판에 계상한다 (#8095) |
| `6bcefd1ac368ad81dca81239a1e8ee6e1f2ae092` | `3cbaabdf06284b220d0b080e11c2a706f62790e6` | docs: 156465345 1쪽 수정 전후 대조 (#8095) |

## 보정과 검증의 실행 순서

1. 최신 devel 위에서 기여를 먼저 적용하고 source 124개와 공통 보정 순서를 고정했습니다. source·현재 SHA·커밋 제목 전체는 [실제 통합 순서 원장](../../../mydocs/pr/assets/planet6897_20261018/documents/integration_commit_order.md)에 연결합니다.
2. 원본·독립 Print·source별 claim을 대조해 공통 컷 소유·여백·줄/표 흐름·수식/폰트/편집을 보정했습니다. 구체적인 생산/소비 코드·반례·전후 관측은 통합 계획과 문서별 증거를 재사용합니다.
3. runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4`의 전체 로컬 검증과 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`의 새 5건·lint를 완료하고 code `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`로 Open PR #8148을 제출했습니다. [실제 검증 명령과 결과](../../../mydocs/pr/assets/planet6897_20261018/documents/validation_commands.md)와 [같은 번호의 최종 검토](../../../mydocs/pr/archives/pr_7956_review.md)에 실제 결과를 연결합니다.
4. 정확한 code head의 [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39705756548) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39705756552) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39705756369) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39705756537) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39705756582) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39705756438)가 모두 성공했습니다. 원 PR 27건의 review/implementation와 오늘할일을 같은 PR의 문서 trailing으로 기록하며, 그 최신 head의 CI·병합 결과·링크를 push 전후 확인합니다.
5. 정확한 trailing head로 merge한 뒤 merge SHA 증거 comment·원 PR open/closed 처리·완료 이슈/하위 이슈 정리·devel 동기화·branch/worktree/output 정리를 수행합니다. 공유 target·사용자 자료·기여자 fork는 보존합니다.

## 보정·되돌림 경계

- 이 원 PR의 기여 SHA는 위 표로 고정하고 공통 보정 및 전후 관측은 [통합 계획·생산/소비 경로·보정 기록](../../../mydocs/plans/pr_review_planet6897_20261018.md)에 연결합니다. 문서별 임계값 완화나 source branch 강제 재작성으로 결과를 맞추지 않습니다.
- 원 PR source는 유지합니다. 미병합 통합 후보의 결함은 해당 공통 계층의 추가 보정으로 처리하고, 이미 통합한 변경을 되돌릴 때는 표의 source SHA 및 후속 공통 보정 의존성을 함께 검토합니다.
- 최종 판정과 GitHub approve/push/merge/close를 구분합니다. 사용자의 기존 승인 범위 안에서 실제 최신 상태와 필수 결과를 확인해 후속 처리를 이어갑니다.
