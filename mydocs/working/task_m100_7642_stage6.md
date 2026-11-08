# 이슈 #7642 — 최신 devel 통합과 쪽 테두리의 독립 출력 대조

이전 단계: [저장된 빈 Square 호스트](task_m100_7642_stage5.md).

## upstream 통합

단계 5를 `dcdbb4288`로 커밋한 뒤 upstream/devel
`60efc4b7b380b2a183c20c3ee0ab24a60bc2d832` 위로 126개 커밋을 rebase했다.
현재 재적용된 단계 5 head는 `e54ebb9c7`이다. rebase 전 이력은 로컬
`backup/equation-metrics-before-rebase-20261008`에 보존했다. cherry-pick 이력은 삭제하지 않았다.

충돌은 쪽수 원장·검증 지침·렌더 정규화·오늘 할 일에서 해결했다. upstream #7626의
폭 없는 저장 줄 폐기와 이번 수식 표/글상자 정규화를 모두 보존하고, 변경된 각 경로가
같은 compose 호출로 이어지도록 연결했다. 이전 base의 시각·회귀 통과를 새 head의
통과로 보고하지 않는다. #7626·빈 호스트·그림·p122·#2069 편집 대조를 함께 실행한다.

## 다음 실패 원인의 독립 근거

원본 `samples/hwp3-sample16-hwp5-2018.hwp`의 Print와 Native 테두리에서 이중선 굵기,
간격·원점이 다르다. 저장 spacing은 1420HU이지만 `body_page_border_outset`이 선 묶음의
최소 3px 폭에 2.5배를 곱해 위·좌우의 원점을 추가 확장한다. `create_border_line_nodes`의
이중선 분해도 선언 굵기를 최소 3px에 맞춰 바꾼다. 이 값을 그대로 기대값으로 고정하지 않는다.

독립 정상 저장·Print 대조군은 종이/본문 기준 × 실선/이중선 × 굵기 3종의 12개다.
기존 정상 저장 반례의 문단을 기반으로 설정을 바꾼 진단 입력을 한컴에서 다시 정상 저장한
뒤 Print 출력한다. 정상 저장본·Print 계약과 저장 값의 변경점을 확인하며 원본 64쪽과 구분한다.
자료 생성기는 ignored output에만 남긴다.

## 소비 경로와 확인 범위

- 선언 테두리 굵기 → 선 묶음 분해 → 수평/수직·대각선 실제 stroke/offset.
- 쪽 spacing·위치 기준 → layout 쪽 테두리 원점 → 실제 root 선 출력.
- 동일 테두리 원점 → `get_page_info_native`의 Studio 표시 값. 별도 clamp/계산 차이도 확인한다.
- 실선과 종이 기준은 정상 대조군으로 유지한다. 바닥 테두리와 후속 내용·쪽수도 직접 확인한다.
- 기준 Print의 기하와 일치하는 공통 결과를 확인한 뒤 구현한다. 새 관계 회귀는 해당
  Native/fresh WASM 전쪽 90% 및 직접 판독 뒤 추가한다. 원본 미달 회귀의 기대값은 유지한다.

## 증적과 상태

로그·전쪽 TSV·중간 JSON·다음 실행 계획은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/`에만 보존한다.
전체 실패 원본·시험지·수식·#1139 전체 회귀 범위는 계속 진행 중이다.
새 PR 생성·CI·merge 완료로 보고하지 않는다.


## 독립 Print의 선 묶음 관측

12개 정상 저장본·Print PDF는 모두 한 쪽, 24개 Save/Print job exit 0이다. Print의
engine 2020 / 한컴 11.0.0.9136 / PrintMethod 0 / one-up을 실행 결과에서 확인했다.
본문 기준 이중선 굵기 index 6의 수정 전 Native review는 16.56282%로 미달이다.
review를 직접 표시했으며 위·좌우 테두리의 위치와 굵기가 실제로 다른 것을 확인했다.

| 굵기 index | 실선 stroke (pt) | 이중선 각 stroke (pt) | 이중선 두 중심 사이 거리 (pt, 근사) |
| --- | --- | --- | --- |
| 1 | 0.36 | 0.12 | 0.36 |
| 6 | 1.079 | 0.24 | 0.72 |
| 10 | 2.878 | 0.72 | 2.16 |

종이·본문 기준에서 같은 선 묶음 계약이 관측됐다. 선언 굵기를 4등분한 각 stroke를
한컴의 1/600 inch 격자에 반올림하며, 두 선 사이 빈 간격은 각 stroke의 두 배다.
이는 3px 최소 폭이나 현 화면의 위치를 기준으로 삼는 보정과 다르다.
실선과 이중선 중심은 모두 기준 영역과 저장 spacing으로 정해진다. stroke 굵기의
몇 배를 위/좌우 원점에 추가하는 방식은 독립 Print와 맞지 않는다.
PDF의 printer 좌표 양자화·종이 정수 pt 반올림은 source 공간의 배치 계약과 구분한다.


수정 전 12개 Native 전쪽 비교에서는 본문 기준 이중선 index 1/6/10이 각각
18.09789% / 16.56282% / 21.44591%, 본문 기준 굵은 실선 index 10이 60.27082%로
미달이다. 나머지 8개는 100%지만 종이 기준 이중선의 실제 stroke는 독립 PDF와 다르다.
2px 이웃 관용 점수는 미세한 선 폭 차이를 면제하지 않으므로 선 묶음의 직접 기하
관측과 확대 판독을 별도로 유지한다. 수정 전 실행은 단계 5 보존 Native의 증거이며,
rebase 뒤 새 head의 시각 결과로 바꾸어 보고하지 않는다.


## 보정 구현과 첫 결과

`border_geometry::page_border_rect`가 저장 spacing과 실제 `PageLayoutInfo`의 종이/본문
영역에서 선 중심의 박스를 생산한다. `build_page_borders`와 `get_page_info_native`가
그 박스를 직접 소비한다. 선 굵기에 비례한 추가 원점 확장과 정보 경로의 별도 clamp를
제거했다. 양쪽 모두 실제 페이지의 제본·본문 영역을 사용한다. stroke 잉크 범위는
`LineNode::ink_bbox`가 별도로 소유하며 본문 흐름·쪽 예산은 바꾸지 않는다.

같은 모듈의 `double_line_parts`는 수평/수직 생성자와 대각선 수직 법선 생성자가 함께
소비한다. 기존 3px 최소 폭·고정 gap은 제거했다. 실선의 선언 굵기 양자화와 다른
복합선의 분해 규칙은 이 단계에서 변경하지 않았다. 대각선 실물 출력은 아직 미검증이다.

Native 빌드는 exit 0, 5분이며 보존 binary SHA-256은
`7380734a3e0232d2ab6e0e8a51db3753a5a2983dbc5b501719ed2770e6b2e828`이다.
`page-border-provenance.json`에 source 4개·head/base를 고정했다. 첫 12개·12쪽 Native
전쪽 TSV는 모두 100%, exit 0, 누락 쪽 없음이다. TSV의 `not_evaluated`를 승인 gate로
보고하지 않는다. 대표 PNG·fresh WASM·원본 전체 결과와 정식 회귀/lint는 계속 진행 중이다.

rebase 경계 검사는 13분 57초 컴파일 뒤 23 PASS / 0 FAIL, exit 0이다.
`rebase-boundary-nextest.log`에 보존했다. 이 결과는 테두리 보정 전 source의 검증이며
보정 후 source의 회귀 통과로 대신하지 않는다.


## 최종 테두리 검증

fresh WASM wrapper는 15분 53초, exit 0이며 bg SHA-256은
`b3d4763ea886e2db82e0eb60d5dfd1729cad50b46a85dbb1883b8b105d5cdb05`다.
JS SHA-256은 `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`로
기존 tracked JS와 동일하다. pkg/Studio가 일치한다. 12개·12쪽의 최종 Native/fresh
WASM TSV는 모두 100%, exit 0, 누락 쪽 없음이며 생산 source 4개 해시도 유지된다.

본문/종이 기준 대표 Native·WASM review/overlay를 각각 직접 확인하고
[8개 자산](../pr/assets/issue7642_page_border/)을 보존했다. 대표 gate는 `passed`, 전쪽
TSV만 산출한 gate는 `not_evaluated`다. 선 중심·상대 간격·물리 stroke는 독립 PDF와
일치한다. 96dpi에서 Native의 얇은 선은 회색 안티앨리어싱으로, PDF 래스터는 더 검게
보이는 농도 차이가 남는다. 같은 검은 벡터 stroke의 0.24pt = 0.32px@96dpi 기하와
저해상도 래스터 표현을 구분하며, 점수 100%를 RGBA 픽셀 동일성으로 보고하지 않는다.

두 경로의 전쪽 기준을 확인한 뒤 정상 저장 HWP/PDF 12쌍과 관계 회귀 2건을 등록했다.
검사는 경계·저장 spacing의 소속, 네 변 선 묶음 누락/중복, 독립 Print 선 굵기의 종이
너비 비율, 두 선 중심 거리/굵기 비율, Studio 정보/실제 paint의 같은 중심을 확인한다.
절대 px 위치·영역·SVG 해시를 고정하지 않는다.

정식 suite 019의 같은 새 검사에서 CLI만 보정 전 실행 파일로 바꿔 **2 FAIL, exit 101**을
확인했다. 실패는 spacing 원점·독립 Print stroke 비율이며 빌드/환경 실패가 아니다.
보정 후 suite 019·020·016·015의 관련 23건은 **23 PASS / 0 FAIL, exit 0**이다
(컴파일 7분 59초). prepare 재배정으로 저장 호스트가 suite 011로 옮겨진 것을 확인해
누락된 기존 2건도 별도로 실행했고 **2 PASS / 0 FAIL, exit 0**이다(컴파일 14.76초).
두 실행을 합한 관련 검증은 25 PASS이며 전체 회귀 통과로 승격하지 않는다.
로그는 `stage6-formal-before.log`, `stage6-formal-boundary-nextest.log`,
`stage6-stored-host-nextest.log`다. 파생 suite/manifest는 커밋하지 않는다.

| 검증 | 결과 | 로그 |
| --- | --- | --- |
| fmt check | PASS, 21.67초 | `stage6-fmt-check.log` |
| Native Clippy | PASS, 76.24초 | `stage6-clippy-native.log` |
| WASM lib Clippy | PASS, 67.67초 | `stage6-clippy-wasm.log` |
| workspace build | PASS, 165.81초 | `stage6-workspace-build.log` |
| workspace all-targets Clippy | PASS, 146.02초 | `stage6-clippy-all-targets.log` |
| integration manifest | PASS, 1491 sources / 48 targets | `stage6-manifest-check.log` |
| source unit tier 정책 | PASS, 4199 tests / 298 modules | `stage6-unit-tiers-check.log` |

위 정책 비교 base는 `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832`다.
`stage6-lint-results.json`에 모든 실행·exit 0·시간을 고정했다.

## 원본의 남은 미달

원본 24개 Native 순회를 완료했다. p122는 전 3쪽 100%, 2022 10월 HWP는 18쪽 최저
91.24244%를 유지했다. 다른 원본은 미달 또는 쪽수 불일치/측정 불가다. HWP3 변환본은
2018/2024 각 64쪽 최저 31.13628%, 1쪽 85.32386%, 3쪽 46.26170%다. 3쪽의 새
review를 직접 표시했고 본문·글상자와 다음 문단의 원점 차이가 남는 것을 확인했다.
`page-border-failed-native/results.json`과 `page-border-failed-native-status.tsv`를 따른다.
failed-18의 Chromium 기동 timeout은 렌더 결함과 구분해 기존 로그를 보존하고 해당
문서만 재실행한다. 미달 원본의 기존 px/hash 회귀·기대값은 변경하지 않았다.

독립 CFB 원문 읽기와 한컴 정상 재저장 진단 모두 p69의 문단 앞 간격 raw 1704HU,
저장 vpos 852HU를 확인했다. 현재 파서는 이를 각각 852HU·0으로 바꾸고 HWP3 좌표
경로를 적용한다. 원문 간격/vpos를 보존하고 일반 HWP5 캐시 경로로 재생한 3쪽 진단은
92.80141%다. 원문 값만 복원하고 기존 좌표 경로를 남긴 실험은 46.68349%, vpos만은
67.25535%, 간격만은 50.75703%였다. 생산 코드는 아직 이 실험으로 변경하지 않았다.
이 결과는 원본 전체·최종 Native/fresh WASM 통과 증거가 아니라 다음 원인 보정의 진단이다.
진단 파일·다음 실행 계획은 ignored output에만 보존한다.

전체 실패 회귀·#1139 전체·시험지·수식 확대 판독 범위는 계속 진행 중이며 새 PR은
없다. code head CI 성공 뒤 self review/오늘 할 일 trailing 기록을 작성하는 절차를 유지한다.
