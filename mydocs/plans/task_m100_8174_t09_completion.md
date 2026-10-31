# #8174 T09 — 최신 devel 통합과 PR 제출 완료 범위

2026-10-28. 사용자 「다음 T09 진행을 승인합니다」를 T09 전체의 착수·충돌 보정·필수 검증·
원격 작업 브랜치 push·Open PR 제출의 승인으로 기록한다.
2026-10-29 사용자 추가 승인으로 정확한 head CI 성공·self-review·문서 trailing CI·merge·
후속 처리까지 범위를 확대한다. registry 게시·태그는 이 승인에 포함하지 않는다.
기본 경로는 본인 PR의 `collaborator_self_merge`, 보조는 intake/local_validation/
visual_fixture_evidence이며 PR review·오늘할일은 채번·정확한 code head CI 성공 뒤 기록한다.

- 시작 head: `4ef642c9edf8313ce07bc6e3334f9cd77ca8b350`.
- 1차 고정 devel: `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`.
- 공통 기준: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 본인 열린 PR은 조회 결과 0개다. 이슈는 open·담당자 edwardkim이다.

| ID | 처리 | 종료 조건 |
| --- | --- | --- |
| J01 | 변경/선행 PR·권위 확인 | 들어온 14개 first-parent 통합의 실제 diff·PR 계약과 영향/비영향을 구분한다. 원본 120개 커밋과 T08 증거를 보존한다. |
| J02 | 누적 후보의 최신 base rebase | 원본 작업 브랜치를 보존하고 별도 후보의 누적 변경을 한 커밋으로 정리한 후 최신 devel로 실제 rebase한다. 충돌/자동 적용 경로 모두 새 base 동작과 작업의 예산·실패·복구 계약을 보존한다. 생성 JS/WASM은 재빌드한다. |
| J03 | 통합 기능·필수 로컬 게이트 | 공개 request/CLI·Native 격리·WASM/Studio, Rust lint/build/focused/전체 nextest·Native Skia·Node/타입·CI 계약과 최신 base 정책을 현재 후보에서 통과한다. 실패는 선행 해결/기대 계약을 추적해 보정하며 기준값으로 숨기지 않는다. |
| J04 | 전쪽·편집/저장·새 base 기능 보존 | 기존 작업 38개 문서와 실제 영향을 받는 새 base PR의 재현/대조군을 선정해 같은 Native/fresh WASM 출력·독립 기준으로 확인한다. 쪽수/내용/90%·완료 페이지 98%·base 대비 2%p 하락과 직접 판독을 적용한다. 새 기능 때문에 base에서 실행 불가인 범위는 이유와 T08 대조를 구분한다. |
| J05 | 제출 자료·포장 영향 | 비공개 원본/식별 자료는 로컬에 두고 공개 전쪽 TSV·대표 review/overlay를 정식 경로에 보존한다. package source 변경에 필요한 실제 포장·strict gzip·소비를 갱신한다. 필수 잔여 0건인 후보와 본문을 확정한다. |
| J06 | Open PR·정확한 head CI | 생성 직전 본인 PR을 재조회한다. 승인 범위의 새 작업 branch를 push해 devel 대상 Open PR을 생성한다. 실제 본문 이미지·원문 링크를 확인하고 정확한 head CI를 확인한다. CI 성공 뒤 필요한 self-review/오늘할일 문서-only trailing과 그 head의 CI를 역할별 절차로 처리한다. 최종 CI와 최신 base 병합 가능성을 확인한 뒤 승인된 merge·후속 처리를 완료한다. |

Cargo는 기본 작업공간의 `target/pr-review` 한 곳을 순차 사용한다. 임시 자료는
`output/pr-review/issue8174/t09-completion-20261028/`에 보존한다. 현재 base 확인만을 위한
반복 rebase나 이미 검증한 동일 code head의 불필요한 재실행은 하지 않는다. source가 실제로
바뀐 통합 후보의 검증은 T08 결과로 대체하지 않는다.

2026-10-29 제출 직전 추가 기준은 `f291d772a105d46a4d4d7f40ed13e82391ebdc1a`다. #8348의 기여 PR 20개와 #8350의 기록을 실제 통합하고 같은 T09 범위에서 기능 보존·최종 검증을 계속한다. 이전 `db5cd5bcc` 후보의 통과를 이 기준의 통과로 재사용하지 않는다.
