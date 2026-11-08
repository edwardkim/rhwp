# 이슈 #7642 — 저장 줄이 없는 그림의 공통 흐름 보정

이전 단계: [저장 그림 상자와 실패 원본 검증](task_m100_7642_stage3.md).
시작 커밋: `67cade445`. 전체 실패 회귀와 시험지 검증 범위는 유지한다.

## 독립 근거와 원인

정상 한컴 저장본의 그림 크기를 보존하고 저장 LineSeg만 제거한 대조군을 한컴 2020
MCP Print로 출력했다. 그림은 두 입력 모두 2쪽에 있고, `affect_line_spacing`을 켜면
후속 문단이 아래로 이동한다. 현재 RHWP는 그림을 누락하고 상대 비율 2500을 HWPUNIT
높이로 소비한다. 원본 p122의 저장 경로 100%를 이 재조판 경로의 통과로 대신하지 않는다.

## 보정 계획과 실제 소비 경로

- `inline_flow::plan`의 기존 텍스트·개체 순서와 물리 줄 결과에 글자처럼 그림을 연결한다.
  저장 줄이 없는 그림 문단은 이 소유자가 줄바꿈·기준선·줄 간격을 한 번 결정한다.
  저장 줄을 가진 입력은 원래 저장 경로를 유지한다. 캡션은 별도 배치 계약이 있으므로
  이번 공통 그림 흐름으로 넘기지 않는다.
- 공통 그림 크기 생산자 → plan의 줄 상자/여백/기준선 → `typeset_inline_flow`의 단 예산
  확인 → `commit_inline_flow`의 예약 높이와 동일 plan 보존 → `layout_inline_flow_plan`의
  최종 그림·글줄 소유와 다음 문단 원점까지 추적한다. paint는 줄을 다시 나누지 않는다.
- 같은 줄 그림은 기준선·하강부를 포함한 영역으로 합치며, 가용 폭을 넘으면 다음 줄로
  넘긴다. 그림의 줄 간격 영향 속성을 켠 경우 그 줄의 간격 기준에 그림 상자를 반영한다.
  속성을 끄면 호스트 글자 크기의 간격을 사용한다. 문서 ID나 화면 좌표 상수는 쓰지 않는다.
- 그림의 줄 상자 HWPUNIT과 상대 크기의 인쇄 잉크를 구분한다. 남아 있는
  `tac_object_flow_height_hu`도 같은 단위 생산자를 소비한다.
- 한 단보다 큰 문단은 기존 분할 소유자에 남는다. 이 경로를 공통 plan의 통과로 보고하지
  않으며, 실제 실패가 검출되면 분할 계약과 독립 Print를 추가 확인한다.

## 검증 순서

1. false/true 재조판 대조군의 Native 전체 쪽과 그림·후속 문단 확대를 먼저 비교한다.
2. 여러 그림의 같은 줄/너비 부족, 명시적 줄바꿈, 가운데 세로 정렬과 단 이월의
   독립 Print 반례를 확보해 실제 소유·높이·후속 내용 보존을 확인한다.
3. fresh WASM에서 같은 범위를 재검증하고 저장 경로 p122·상대 크기 반례 및 영어/국어
   대조군을 다시 비교한다. 관련 모든 쪽이 90% 이상이고 직접 판독이 맞을 때만 관계
   회귀를 추가한다. 실패 원본의 기존 기대값은 아직 변경하지 않는다.
4. 원본 24개 재대조의 환경 오류는 재산출하고, 실제 미달 문서는 별도 원인을 계속 보정한다.
   최종 전체 회귀·lint·PR·CI·후속 처리는 완료되지 않았다.

실행 로그·진단 입력·TSV는 ignored `output/pr-review/issue7642/failed-inputs-20261008/`에
보존한다. 정상화 대조군과 원본을 구분하고, 검증된 HWP·Print PDF·대표 review PNG는
증적 커밋에 포함한다.

## 첫 보정과 확대 판독

- 첫 보정의 Native/fresh WASM은 저장 대조군 57쪽과 재조판 반례 14쪽, 합계 71쪽에서
  쪽수가 일치하고 90% 이상이다. 재조판 false는 100%, true는 최저 99.99937%,
  여러 그림·개행·가운데 정렬·쪽 이월 5개는 전쪽 100%다. 최초 그림 누락 코드에서는
  같은 재조판 입력의 최저값이 0.48922~5.88752%였다.
- 첫 fresh WASM 빌드는 exit 0, 15분 44초로 종료했다. `picture-rows-first-provenance.json`
  에 source 사본·Native/WASM 해시를 연결한다. 후속 텍스트 보정 전 코드의 증거이며,
  변경된 현재 source의 최종 검증으로 재사용하지 않는다.
- 그림 사이에 글자를 놓은 CENTER 반례는 99.72171%를 통과했지만 글자가 한컴보다 위에
  놓였다. 그림과 글꼴의 서로 다른 기준선을 같은 정렬점으로 취급한 것이 원인이다.
  물리 줄의 정렬점과 실제 글꼴 기준선을 분리하고, 그림 옆 글자 폭의 누적 반올림도 제거한다.
  before review를 직접 표시했다. 고득점만으로 완료하거나 회귀를 추가하지 않았다.
- 글자가 앞에 있는 그림과 바깥 여백 반례도 독립 정상 저장 → LineSeg 제거 → Print로
  확인했다. 각 2쪽 모두 첫 보정의 Native에서 100%다. 추가 텍스트 보정 후 다시 산출한다.
- 페이지 끝의 그림 높이/줄 간격 경계 2개도 Print했다. 한컴과 첫 Native 모두 그림·
  후속 문단을 2쪽으로 이월한다. 후행 줄 간격을 fit에서 임의로 빼면 된다는 가정은
  이 독립 출력에서 확인되지 않았으므로 해당 구현은 변경하지 않는다.
- 실패 폰트 추적 검사의 짝인 `samples/3-10월_교육_통합_2022.hwpx`를 별도로 Print했다.
  job `7cb694c3-cfee-4de3-9e30-50126d9f83bc`, Native 18쪽 최저 35.33407%다.
  HWP의 91.24244%만 보고 paired 회귀의 count/hash 기대값을 변경하지 않는다.

## 텍스트 보정과 추가 줄 높이 확인

- CENTER의 정렬점·문자 폭 보정 후 동일 영역은 Native 100%다. Native 추가 반례 12개·
  24쪽은 최저 99.99937%, 정상 대조군 57쪽도 전쪽 90% 이상이다. 같은 코드를 다시
  fresh WASM으로 빌드했으며 실행 결과는 `picture-text-align-wasm/results.json`에 보존한다.
- 관계 검사 3건의 ignored 초안은 최초 그림 누락 코드에서 3건 FAIL, 첫 그림 보정에서
  CENTER 1건 FAIL, 텍스트 보정 후 3건 PASS다. 최종 정식 회귀 추가를 대신하지 않으며
  현재 원본 24개 중 미달 문서의 기대값은 유지한다.
- 글자 표시 크기 106%와 작은 그림을 섞은 추가 반례를 한컴 정상 저장 → LineSeg 제거 →
  Print로 확인했다. job `1e865bf3-388b-4b51-b834-b0910c972b98`이며 정상 저장본은 선언
  크기 1000HU, relative_sizes[0]=106, 줄 높이 1000HU·기준선 500HU·간격 600HU다.
  보정 전 Native는 줄 높이를 표시 크기로 계산해 14.1px로 늘리고 후속 문단을 밀었다.
  같은 영역이 99.92206%여도 이 차이를 면제하지 않는다. 선언 크기의 줄 상자와 표시
  크기를 분리하고, 최종 Native/fresh WASM과 독립 저장 메트릭의 관계 검사를 재실행한다.
- 선언 크기 보정 후 Native 상대 글자 크기 반례는 100%이며, 실제 글줄 높이는 13.3px로
  정상 저장본의 1000HU를 소비한다. 106% 표시 크기는 유지한다. 최종 재조판 반례 13개·
  26쪽은 최저 99.99937%이고 쪽수가 일치한다. 확대 PNG를 직접 표시해 글자 모양·위치와
  그림·후속 문단을 확인했다. 기존 입력 6개·57쪽의 Native 재검증도 완료했다.
  최종 fresh WASM은 15분 1초, exit 0으로 완료했고 Studio와 루트 pkg의 SHA-256이 같다.


## 최종 시각 증적과 관계 회귀

최종 생산 source 8개의 SHA-256은 `picture-logical-font-provenance.json`으로 고정했다.
Native 실행 파일은 `3b0e6aad53a6429baa8b19e4e7e3638cd737499e82af7bfff6443d0730b3583d`,
fresh WASM은 `209e7bd5e0fa066e6114ab6f59c48ba9279a67c06044a50634197e9d2ea10f72`다.

| 범위 | Native 쪽수/최저값 | fresh WASM 쪽수/최저값 | 판정 |
| --- | --- | --- | --- |
| 저장 줄 없는 독립 Print 반례 13개 | 26쪽 / 99.99937% | 26쪽 / 99.99937% | 쪽수 일치·90% 수치 기준 충족 |
| 원본 p122·상대 크기 정상 저장·영어·국어 대조군 6개 | 57쪽 / 90.61634% | 57쪽 / 90.61634% | 쪽수 일치·90% 수치 기준 충족 |

전쪽 TSV·명령·기준 SHA는 `picture-logical-font-native/results.json`,
`picture-logical-font-native-controls/results.json`, `picture-logical-font-wasm/results.json`에
연결했다. 합계 19개 입력·83쪽은 두 경로 모두 90% 미만·누락 쪽이 없다.
최종 CENTER와 상대 표시 크기의 WASM review를 직접 표시하고, 같은 배율로 그림·글자
정렬과 후속 문단을 확인했다. 글자 픽셀의 안티앨리어싱 차이는 실루엣 점수와 구분했다.

대표 증적:

- [CENTER review](../pr/assets/issue7642_inline_picture_flow/center-final-wasm-p002-review.png)
- [CENTER overlay](../pr/assets/issue7642_inline_picture_flow/center-final-wasm-p002-overlay.png)
- [상대 표시 크기 review](../pr/assets/issue7642_inline_picture_flow/relative-font-final-wasm-p002-review.png)
- [상대 표시 크기 overlay](../pr/assets/issue7642_inline_picture_flow/relative-font-final-wasm-p002-overlay.png)

시각 검증 뒤 [HWP·Print PDF 13쌍](../../tests/fixtures/issue_7642/no-ls/README.md)을 보존하고
`tests/cases/issue_7642_missing_lineseg_picture_flow.rs`에 관계 검사 4건을 추가했다.
실제 글줄·문단·쪽 소유, 그림/글자 순서와 포함·겹침, 후속 본문 누락·중복,
CENTER 정렬점 및 독립 저장 메트릭의 무차원 비율을 검사한다. 절대 px 영역·좌표·해시는
고정하지 않는다. ignored 초안의 단계별 FAIL/PASS는 `proposed-four-relations-results.json`에
있으며 정식 실행 결과는 아래에 별도로 기록한다.

### 남은 범위

원본 24개 중 p122 외 미달 문서와 전체 회귀 실패는 해결 완료가 아니다. 특히 2022년
10월 HWP는 18쪽 최저 91.24244%지만 같은 HWPX는 35.33407%이므로 짝 문서의 폰트
추적 count/hash 검사를 아직 변경하지 않았다. 합성 대조군의 83쪽 통과를 원본 전체의
통과로 보고하지 않는다. 캡션과 단보다 큰 그림 문단의 기존 분할 경로도 이번 새 소유자의
검증 범위에 포함되지 않는다. 전체 회귀·PR·CI·merge·후속 처리는 계속 진행한다.


### 정식 회귀 실행

`cargo nextest run --cargo-profile release-test --target-dir target/pr-review
--test regression_suite_001 --test regression_suite_018
-E 'test(issue_7642_missing_lineseg_picture_flow) | test(p122_stored_vpos_page_break)'
--test-threads 12 --no-fail-fast`는 컴파일 4분 27초, 8건 PASS, exit 0으로 끝났다.
새 관계 4건과 기존 p122 4건이며 전체 회귀 완료를 의미하지 않는다.
로그는 `stage4-formal-nextest.log`에 있다.

동일 정식 suite 실행 파일의 CLI 입력만 해시 고정된 이전 단계로 바꿔 원인별 검출을
확인했다. `stage4-formal-before-after-results.json`에 실행 파일·CLI·로그·종료값을 연결했다.

| CLI 단계 | 관계 검사 4건 결과 | 검출 의미 |
| --- | --- | --- |
| 그림 흐름 보정 전 | 1 PASS / 3 FAIL | 실제 줄 소유·CENTER·그림 간격 결함 |
| 첫 그림 흐름 보정 | 2 PASS / 2 FAIL | CENTER와 상대 표시 크기의 새 경로 결함 |
| 텍스트 정렬 보정 | 3 PASS / 1 FAIL | 상대 표시 크기가 선언 줄 상자를 늘린 결함 |
| 최종 선언 줄 상자 보정 | 4 PASS | 원인별 검사 통과 |

상대 표시 크기 검사는 최초 legacy 코드에서도 통과한다. 새 그림 흐름의 잘못된
글리프 크기 사용을 검출하는 대조군이며, 최초 그림 누락 검출 검사로 주장하지 않는다.


### 최종 lint·정책 확인

helper의 `option_env!().expect()` Clippy 오류를 기존 CLI 검사와 같은 `env!()` 방식으로
고치고, 포맷 후 파생 suite를 재준비했다. 최종 배정은 새 관계 검사가 suite 020,
p122가 suite 016이다. 같은 filter의 최종 실행은 컴파일 17.68초, 8건 PASS, exit 0이다
(`stage4-formal-nextest-final.log`). 생산 source 8개의 해시는 시각 검증 당시와 같다.
이 구현·fixture·관계 회귀·대표 PNG를 커밋 `305ab694b`로 고정했다.

- `cargo fmt --all -- --check`: PASS (`stage4-final-fmt-check.log`).
- Native Clippy: PASS, 1분 44초 (`stage4-clippy-native.log`).
- WASM lib Clippy: PASS, 1분 28초 (`stage4-clippy-wasm.log`).
- locked workspace build: PASS, 3분 11초 (`stage4-workspace-build.log`).
- 최종 파생 suite의 workspace/all-targets Clippy: PASS, 9.74초
  (`stage4-final-clippy-all-targets.log`).
- base `b3c3047db575d146dff9d39098e6de98c4630b3e` manifest 정책: PASS,
  1488 sources / 6401 static test attrs / 48 integration targets (`stage4-manifest-check.log`).
- 같은 base의 unit tier 정책: PASS, 4199 tests / 298 modules (`stage4-unit-tiers-check.log`).

파생 harness·manifest·unit inventory·TSV·로그는 stage하지 않는다.
이 단계는 검증된 그림 흐름만 고정하며, 90% 미만인 다른 원본의 px/hash 기대값을
바꾸거나 전체 회귀·PR 승인 완료로 판정하지 않는다.


전쪽 `--silhouette-only` TSV 산출의 `pr_review_gate.status`는 **not_evaluated**다.
위 표는 수치 기준 충족이며 자동 승인 gate 통과라는 뜻이 아니다. 대표 페이지의 review·
overlay·확대 판독은 별도로 실행했다. 전체 변경 범위에 미달 원본이 남아 있으므로 이
단계의 검사 통과나 새 관계 회귀를 PR 승인·통합 가능 판정으로 승격하지 않는다.
