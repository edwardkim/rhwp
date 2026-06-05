# PR #1255 처리 보고서 - Issue #1253 미주 모양 UI와 HWPX 구분선 굵기 정합

- **작성일**: 2026-06-03
- **PR**: #1255
- **연결 이슈**: #1253
- **브랜치**: `local/pr1255-verify`
- **병합 커밋**: `083b5cac`
- **PR head**: `9080d94560269e8622a988d12ca6cb81defa7649`
- **기준 devel**: `5453b254`

## 1. 처리 요약

PR #1255를 현재 `devel` 기준 검증 브랜치에 병합했다.

이번 PR의 수용 범위는 다음이다.

- rhwp-studio `주석 모양 > 미주 모양` UI를 한컴오피스 2024 대화상자에 가깝게 보강
- 구분선 종류/굵기/색 선택을 미리보기 중심 UI로 변경
- 구분선 길이 항목에 `사용자` 콤보 추가
- HWPX `<hp:noteLine width>`를 일반 선 굵기 코드표로 파싱
- HWPX `<hp:numbering type>`을 미주 번호 매기기 UI에 반영
- HWPX `<hp:placement>`를 미주 위치 UI에 반영
- `0.12mm -> 1`, `0.7mm -> 9` 회귀 검증 추가

PR 본문과 Stage3 문서에 명시된 것처럼 `3-09월_교육_통합_2022.hwp` 9쪽의 실제 미주 조판 차이는 이번 PR의 완료 범위가 아니며 후속 추적 대상으로 남긴다.

## 2. 자동 검증 결과

| 항목 | 명령 | 결과 | 비고 |
|---|---|---|---|
| whitespace | `git diff --check HEAD` | 통과 | 출력 없음 |
| Rust fmt | `cargo fmt --all --check` | 통과 |  |
| Studio build | `npm run build --prefix rhwp-studio` | 통과 | Vite production build |
| 미주 회귀 | `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 43 passed |
| 전체 integration | `cargo test --tests` | 통과 | 전체 통과 |
| WASM | `docker compose --env-file .env.docker run --rm wasm` | 통과 | `pkg/` 산출물 생성 |
| Studio build after WASM | `npm run build --prefix rhwp-studio` | 통과 | public WASM wrapper 동기화 후 재확인 |
| 색 바인딩 보완 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | 구분선 색 초기값/팔레트/native color picker 동기화 확인 |
| 위치 바인딩 보완 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | 미주 위치 alias 정규화 확인 |
| HWPX 미주 위치 파서 | `cargo test test_parse_endnote_placement_end_of_section` | 통과 | `END_OF_SECTION -> sectionEnd` |
| WASM after 위치 바인딩 | `docker compose --env-file .env.docker run --rm wasm` | 통과 | HWPX parser 변경 반영 |
| Studio build after 위치 WASM | `npm run build --prefix rhwp-studio` | 통과 | public WASM 동기화 후 재확인 |
| 위치 radio 방어 보강 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | radio group name 충돌/미선택 상태 방지 |
| 번호 매기기 radio 방어 보강 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | 번호 매기기 alias 정규화/미선택 상태 방지 |
| HWPX 미주 번호 매기기 파서 | `cargo test test_parse_endnote_numbering_restart_section` | 통과 | `ON_SECTION -> restartSection` |
| 미주 shape API 회귀 | `cargo test issue_1139_stage31_endnote_shape_api_updates_section_shape` | 통과 | Rust command API 문자열 alias 보강 후 확인 |
| WASM after 번호 매기기 바인딩 | `docker compose --env-file .env.docker run --rm wasm` | 통과 | HWPX numbering parser 변경 반영 |
| Studio build after 번호 매기기 WASM | `npm run build --prefix rhwp-studio` | 통과 | public WASM 동기화 후 재확인 |
| 리스트 팝업 UX 보완 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | 구분선 종류/굵기/색상 팝업 외부 클릭/포커스 이동 dismiss |
| 리스트 첫 선택값 보정 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | 구분선 종류/굵기 첫 선택 시 빈 값 방지 |
| 구분선 label/control grid 보완 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | `레이블/선택` 쌍 구분 명확화 |
| 대화창 제목 변경 후 Studio build | `npm run build --prefix rhwp-studio` | 통과 | `주석 모양` -> `미주` |

GitHub checks:

| 항목 | 결과 |
|---|---|
| Build & Test | 통과 |
| Canvas visual diff | 통과 |
| CodeQL | 통과 |
| Analyze (rust) | 통과 |
| Analyze (javascript-typescript) | 통과 |
| Analyze (python) | 통과 |
| WASM Build | skipping |

## 3. WASM/Studio 산출물

`docker compose` 빌드 후 `pkg/` 산출물을 `rhwp-studio/public/`에 동기화했다.

이번 PR은 WASM API를 변경하지 않으므로 public wrapper 파일의 Git diff는 발생하지 않았다.

## 4. 메인테이너 피드백 대응

메인테이너 UI 확인 중 `미주 모양 > 구분선 색` 바인딩이 올바르지 않아 보인다는 피드백이 있었다.

원인:

- PR 초기 구현은 기존 native `input[type=color]`를 숨기고 팔레트 팝업만 노출했다.
- 팔레트에 없는 사용자 색을 선택할 경로가 사라졌고, 색상 값 변경 경로가 native input, 팔레트, 미리보기 버튼 사이에서 한 곳으로 모이지 않았다.

보완:

- `separatorColor` 초기값을 `#rrggbb` 형식으로 정규화한다.
- 팔레트 색상 클릭과 native color picker 변경을 모두 `setSeparatorColor()`로 통합했다.
- 팔레트 하단에 `다른 색...` 버튼을 추가해 브라우저 native color picker를 열 수 있게 했다.
- 현재 선택된 색상 swatch에 outline을 표시해 바인딩 상태를 눈으로 확인할 수 있게 했다.

검증:

- `npm run build --prefix rhwp-studio`: 통과

추가로 `미주 모양 > 미주 위치` 값이 대화창에 바인딩되지 않는다는 피드백이 있었다.

원인:

- Studio UI는 `placement === "sectionEnd"`만 구역 끝으로 인식했다.
- HWPX 파서는 `<hp:placement place="END_OF_DOCUMENT|END_OF_SECTION">`를 `FootnoteShape.placement`에 반영하지 않아 HWPX 출처 문서에서는 위치 정보가 기본값으로 남았다.

보완:

- Studio UI에서 `documentEnd`/`END_OF_DOCUMENT`/`eachColumn`, `sectionEnd`/`END_OF_SECTION`/`belowText` 별칭을 canonical 값으로 정규화한다.
- HWPX `<hp:placement>` 파서에서 `END_OF_DOCUMENT`, `END_OF_SECTION`, `EACH_COLUMN`, `BELOW_TEXT`, `RIGHT_COLUMN`을 내부 `FootnotePlacement`로 매핑한다.
- 파싱 시 HWP5 저장 경로가 참조하는 attr bit 8-9도 함께 동기화한다.
- 추가 확인 중 두 radio가 모두 미선택으로 보이는 피드백이 있어, 대화창 인스턴스별 고유 radio group 이름을 사용하고 DOM 생성 시 기본값을 `문서의 끝`으로 지정했다.

검증:

- `npm run build --prefix rhwp-studio`: 통과
- `cargo test test_parse_endnote_placement_end_of_section`: 통과

추가로 `미주 모양 > 번호 매기기` 값이 대화창에 바인딩되지 않는다는 피드백이 있었다.

원인:

- Studio UI는 `numbering === "restartSection"`만 새 구역 시작으로 인식했다.
- HWPX 파서는 `<hp:numbering type="CONTINUOUS|ON_SECTION|ON_PAGE">` 문서 주석과 달리 `newNum`만 읽고 `FootnoteShape.numbering`을 갱신하지 않았다.
- 위치 radio 보강 전과 같은 이유로, 대화창 재생성/복수 인스턴스 상황에서 radio group 기본 선택값이 DOM 상에서 비어 보일 수 있었다.

보완:

- Studio UI에서 `continue`/`CONTINUOUS`, `restartSection`/`ON_SECTION`, `restartPage`/`ON_PAGE` 별칭을 canonical 값으로 정규화한다.
- `번호 매기기` radio group도 인스턴스별 고유 이름과 기본값 `앞 구역에 이어서`를 가진다.
- HWPX `<hp:numbering type>` 파서에서 `CONTINUOUS`, `ON_SECTION`, `ON_PAGE`를 내부 `FootnoteNumbering`으로 매핑한다.
- 파싱 시 HWP5 저장 경로가 참조하는 attr bit 8-9도 함께 동기화한다.

검증:

- `npm run build --prefix rhwp-studio`: 통과
- `cargo test test_parse_endnote_numbering_restart_section`: 통과
- `cargo test issue_1139_stage31_endnote_shape_api_updates_section_shape`: 통과
- `docker compose --env-file .env.docker run --rm wasm`: 통과
- `npm run build --prefix rhwp-studio`: 통과

추가로 구분선 종류/굵기 selection 팝업에서 값을 선택하지 않고 다른 동작을 해도 리스트가 닫히지 않는다는 피드백이 있었다.

원인:

- 커스텀 팝업 메뉴는 항목 선택 시에만 `closePopupMenus()`가 호출됐다.
- 대화창의 다른 입력칸, native select, 확인/취소 버튼, 대화창 외부 영역을 클릭하거나 포커스가 이동하는 경우를 처리하는 dismiss 경로가 없었다.

보완:

- 커스텀 팝업 루트에 `data-endnote-popup-root`를 부여했다.
- 대화창 표시 중 document capture 단계의 `pointerdown`/`focusin`을 감시해 팝업 루트 바깥으로 사용자 동작이 이동하면 열려 있는 팝업을 닫는다.
- 대화창이 닫힐 때 전역 dismiss handler를 제거해 누수를 방지한다.

검증:

- `npm run build --prefix rhwp-studio`: 통과

추가로 구분선 종류/굵기를 선택할 때 첫 번째 선택은 빈 값처럼 처리되고 두 번째 선택부터 실제 설정되는 피드백이 있었다.

원인:

- 커스텀 preview 버튼의 화살표를 `span:last-child`로 찾고 있어, preview 내부 `span`을 화살표로 오인할 수 있었다.
- hidden select의 값이 HWPX/HWP 출처 값과 option 목록 사이에서 어긋날 경우 빈 문자열 상태가 남을 수 있었다.

보완:

- preview 버튼 화살표에 `data-dropdown-arrow`를 부여하고, preview 교체 시 이 marker만 참조한다.
- 구분선 종류/굵기 선택값을 `setLineTypeValue()`/`setLineWidthValue()`로 통합해 항상 실제 option 값으로 정규화한다.
- preview 갱신 시에도 hidden select 값을 다시 canonical option 값으로 보정한다.

검증:

- `npm run build --prefix rhwp-studio`: 통과

추가로 구분선 영역의 label/control 배치가 `레이블 | 선택 | 레이블 | 선택` 형태로 평평하게 보이며, 어느 label이 어느 control에 대응되는지 혼동된다는 피드백이 있었다.

보완:

- 구분선 영역에 전용 `pairRow()` helper를 추가해 `label/control` 쌍을 명시적인 2쌍 grid로 배치했다.
- label은 고정폭 우측 정렬로 두고, control과의 column gap을 유지해 `종류`, `길이`, `굵기`, `색`의 소속을 분명히 했다.
- `길이`의 select/input/mm 단위는 하나의 inline control 그룹으로 묶었다.

검증:

- `npm run build --prefix rhwp-studio`: 통과

추가로 미주 설정창 제목을 기존 `주석 모양`에서 `미주`로 변경해 달라는 요청이 있었다.

보완:

- `EndnoteShapeDialog`의 modal title을 `미주`로 변경했다.
- tab 제목 `미주 모양`은 설정 범위 설명으로 유지했다.

검증:

- `npm run build --prefix rhwp-studio`: 통과

## 5. 메인테이너 시각 판정표

| 대상 | 확인 항목 | 판정 | 비고 |
|---|---|---|---|
| rhwp-studio `미주 > 미주 모양` | 대화창 제목이 `미주`로 표시되는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 종류가 미리보기 기반으로 표시되는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 굵기 `0.7mm`가 한컴 기준 선택지로 보이는지 | 통과 | raw `9` |
| rhwp-studio `미주 > 미주 모양` | 구분선 색상 선택이 한컴식 색상 칩/팔레트와 유사한지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 길이 항목에 `사용자` 콤보와 mm 입력이 보이는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 영역 label/control 쌍 구분이 명확한지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 종류/굵기/색상 팝업이 다른 컨트롤 클릭 또는 포커스 이동 시 닫히는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 구분선 종류/굵기 첫 선택이 빈 값 없이 즉시 반영되는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 미주 위치 `문서의 끝`/`구역의 끝`이 문서 설정값과 맞게 선택되는지 | 통과 |  |
| rhwp-studio `미주 > 미주 모양` | 번호 매기기 `앞 구역에 이어서`/`현재 구역부터 새로 시작`이 문서 설정값과 맞게 선택되는지 | 통과 |  |
| HWPX 미주 구분선 | `noteLine width="0.7 mm"`가 raw `9`로 해석되는지 | 자동 통과 | Rust parser 테스트 |
| HWPX 미주 위치 | `placement place="END_OF_SECTION"`가 `sectionEnd`로 해석되는지 | 자동 통과 | Rust parser 테스트 |
| HWPX 미주 번호 매기기 | `numbering type="ON_SECTION"`가 `restartSection`으로 해석되는지 | 자동 통과 | Rust parser 테스트 |

메인테이너 판정:

```text
2026-06-03 통과
```

## 6. 남은 절차

메인테이너 UI 시각 판정이 통과되었으므로 다음 절차를 진행한다.

1. `pr_1255_report.md`에 판정 결과 반영
2. `devel`에 검증 브랜치 반영
3. 검토/보고서 문서 커밋
4. `devel` push
5. PR #1255 및 Issue #1253 후속 처리

## 7. 현재 결론

자동 검증과 메인테이너 UI 시각 판정 기준으로 **수용 가능**하다.

PR #1255는 devel 반영 및 PR/Issue 후속 처리 단계로 진행한다.
