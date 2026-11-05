# semanticist21 미지정 PR 17건과 추가 PR #8629 통합 검토 계획

기준 `upstream/devel`: `6e1de406209ba9a58032f6ca6de45b18aa687d1c`.

작업 브랜치: `review/integration-20261105-semanticist21-open`.

2026-11-05 작업지시자의 요청에 따라 리뷰어가 없었던 PR만 `jangster77`로 지정한 뒤 API로 재확인했다. 기존 리뷰어가 있는 12개 PR은 이 통합 대상에서 제외한다. 필요한 메인터너 보정과 원 PR별 최종 리뷰 문서 작성까지 진행한다.

## 처리 순서

| 원 PR | 고정한 원 head | 적용할 일반 커밋 수 |
| --- | --- | ---: |
| [PR #8526](https://github.com/edwardkim/rhwp/pull/8526) | `a4ce64e250512153d3db6e27d88bb9448f120ee2` | 2 |
| [PR #8527](https://github.com/edwardkim/rhwp/pull/8527) | `7243067034a9cb2d64bb0e6f8938f496461b1c34` | 2 |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | `6785bddfb1047ea6e3e74b7fc2eec94efd0d7ce6` | 4 |
| [PR #8536](https://github.com/edwardkim/rhwp/pull/8536) | `0dc6e32b87b0097a9f139f8428b6d17470a31a7f` | 2 |
| [PR #8537](https://github.com/edwardkim/rhwp/pull/8537) | `6c2369bd25c28456b01e629f66c18c7c60176fc7` | 2 |
| [PR #8538](https://github.com/edwardkim/rhwp/pull/8538) | `39669dd94237454610fb6ca522258836912396a6` | 2 |
| [PR #8541](https://github.com/edwardkim/rhwp/pull/8541) | `b7cc596d61a5cebb9b17fa0c1fa26ac5d2f4a007` | 3 |
| [PR #8549](https://github.com/edwardkim/rhwp/pull/8549) | `5ccd6da2adbf628b8a885bf2b2b4fa0e2eb7191b` | 2 |
| [PR #8558](https://github.com/edwardkim/rhwp/pull/8558) | `6e508084575e95681db5dda2794b314f038aa685` | 2 |
| [PR #8564](https://github.com/edwardkim/rhwp/pull/8564) | `5477f2cc6c4daf93ea315828650ae94eaf15b1b7` | 2 |
| [PR #8570](https://github.com/edwardkim/rhwp/pull/8570) | `bd781342ce9ea546cabba540d57b74205c5b5448` | 2 |
| [PR #8571](https://github.com/edwardkim/rhwp/pull/8571) | `dfef7b709773f56535ca1da123bb837eaad85700` | 2 |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e` | 5 |
| [PR #8577](https://github.com/edwardkim/rhwp/pull/8577) | `7af4ed39fbb8c06dec2b202eb29609f11ce4a379` | 2 |
| [PR #8586](https://github.com/edwardkim/rhwp/pull/8586) | `610f0e55c61fbf28a2f301c45163b76b47d8a406` | 1 |
| [PR #8602](https://github.com/edwardkim/rhwp/pull/8602) | `338a75a1afd5bbe62843d71784a5c82b89613620` | 2 |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `5768ed5323238c5b1e1027eb0e0dcba6c62e26d8` | 5 |
| [PR #8629](https://github.com/edwardkim/rhwp/pull/8629) | `d38d02f03ffdc5b6461228394c1aa9f72ffff1f0` | 2 |

## 검증 계약

- 원 PR의 주장마다 실제 입력과 트리거, 독립적인 기대 동작, 현재 통합 코드의 결과를 대조한다. 원본 열기나 높은 TSV 점수로 편집·저장 동작 검증을 대신하지 않는다.
- 원본과 기존 독립 Print PDF를 우선 재사용한다. 기존 기준이 없는 입력에만 버전에 맞는 MCP Print를 추가한다. 실제 사용한 입력·PDF·producer SHA와 쪽 대응을 각 리뷰 문서에 기록한다.
- 조판 영향 문서는 전체 페이지 Native/fresh WASM TSV, 주장한 페이지의 review·overlay와 직접 확대 판독을 확보한다. 미달이나 누락은 메인터너 보정 후 재검증한다.
- 고정 기준과 후보의 기능 보존 및 기존 PR #8389·#8397의 보호 TSV를 대조한다. 원본·글꼴·backend·DPI를 바꾸어 하락을 숨기지 않는다.
- 전체·집중 회귀는 최소 글꼴 환경에서 실행한다. Cargo는 공용 `target/pr-review`, 빌드 잡 8개를 쓰고 겹쳐 실행하지 않는다.
- 원 PR별 리뷰는 최종 검증 근거로 갱신하며 원 head, 체리픽 적용 SHA, 메인터너 보정 SHA를 구분한다. 검증 전에는 승인이나 완료로 기록하지 않는다.

## 준비 자료와 이전 조사

원 PR의 입력·PDF는 `output/pr-review/semanticist21-triage-20261105/pr<번호>/`에 준비했다. 각 폴더의 `manifest.json`으로 다운로드한 원본과 현재 엔진에서 만든 진단 파일을 구분한다. 이전 조사 결과는 기준 `6e1de40620`의 결과이며 통합 후보 검증으로 바꾸어 표시하지 않는다.

읽기 전용 조사 때 실행한 Native 결과는 23개 PR·52개 출력·203쪽이다. fresh WASM은 범위 전환 시까지 8개 PR·15개 출력·31쪽을 완료했으며, 완료하지 않은 전체 검증으로 계산하지 않는다. 당시 대상 중 기존 리뷰어가 지정된 PR도 포함돼 있었으므로 이번 통합 검토 범위와 구분한다.

원 PR의 원격 head와 기존 리뷰어는 변경하지 않는다. 이후 push·통합 PR·원 PR의 종료는 해당 단계의 지침과 작업지시자의 승인 범위에 따라 처리한다.

## 1단계 적용과 공통 보정

18개 PR의 고유 일반 커밋 44개를 모두 체리픽했다. PR #8629는 추가 요청 후 `jangster77` 배정을 API로 확인하고 포함했다. 원 PR head는 변경하지 않았다.

충돌은 PR #8536·#8558·#8571·#8572·#8616·#8629의 6개 커밋에서 해소했다. 10수준 번호의 공통 읽기·쓰기와 5비트 번호 형식, 번호 글자 모양과 글머리표 API, 자리 차지 표와 머리말 경로, 기존 표 높이 보정과 페이지 원점 저장, 구역별 탭 상태와 덧말 스타일, 문단 분할 시 저장 흐름과 원고지 정규화 상태를 모두 보존했다.

첫 `cargo check --tests`에서 Scaffold 공통 생성기의 7수준 초기값이 10수준 모델과 맞지 않는 컴파일 오류를 확인했다. 모델 상수를 사용하도록 정렬한다. 머리말 경로 검사의 이전 인자 목록도 검토해 원래의 글자처럼 취급 모드를 명시하며, 기대값을 완화하지 않는다. 이 단계는 통합·컴파일 보정이며 시각 검증 완료나 최종 승인 판정이 아니다.

공통 보정 뒤 `cargo check --locked --tests --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review`는 종료 코드 0으로 완료했다(2분 19초). 검증 대상은 통합 18건이며, 이 타입 검사 통과를 실제 동작·시각 검증이나 전체 회귀 통과로 대신하지 않는다.

## 2단계 기존 보호와 신규 검사 등록 보정

첫 최소 글꼴 전체 실행은 12,585건 중 12,584건 통과, 1건 실패, 45건 건너뜀으로 종료했다. 실패는 `scaffold_outline_numbering::levels_and_defaults`의 7수준 배열과 확장된 10수준 모델의 불일치다. Scaffold의 1~7수준 계약은 유지하고 8~10수준의 빈 형식과 기본 숫자 모양까지 명시적으로 검사하며, 번호 형식 속성은 새 공통 계약인 bit 5~9를 확인한다. 실물 배치의 기대 좌표를 변경한 것은 아니다.

신규 `tests/cases`를 suite에 반영하는 준비 단계가 첫 실행 전에 누락됐음을 확인했다. 따라서 위 결과는 기존 보호 범위의 실행 기록이며 신규 18개 PR의 전체 검증 완료로 계산하지 않는다. 이후 자동 manifest 준비와 실제 실행 목록 확인을 선행한다. 파생 manifest와 harness는 커밋하지 않는다.

테스트 배치 정책 검사에서는 PR #8616이 제품 소스에 추가한 비공개 helper 테스트 2건이 base 대비 증가로 검출됐다. 이를 기존 `compose_controls_insert.rs`로 이동해 공개 API에서 공백 제외·입력 제한·스타일 비트 설정과 해제를 검사하도록 보정했다. 테스트 목적과 유효한 기대값은 유지한다. 새 구역별 표 생성 API에 맞춰 해당 검사의 중첩 표도 원래 모드인 `treatAsChar=true`를 명시한다. 보정 뒤 unit-tier 검사는 4,099건·296개 모듈로 통과했다.

현재 단계는 테스트 등록과 계약 호환성 보정이며 시각 검증과 최종 판정은 진행 중이다.
