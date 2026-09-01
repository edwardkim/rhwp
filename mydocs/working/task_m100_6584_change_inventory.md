# Task M100 #6584 Stage R2 — v0.8.6 변경·호환성 인벤토리

- **범위**: `v0.8.4` `496333b27d21ddb9114ba9ae340bcb895870c9a7` .. release base
  `063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- **계측일**: 2026-09-02 KST
- **입력**: 2,214커밋, 17,373파일, 기여 PR provenance 262개
- **기여 계보 정본**:
  `mydocs/tech/investigations/issue-6584/release_contributor_ledger.json`

## 1. 결론

v0.8.6은 두 항목짜리 소규모 patch가 아니다. v0.8.4 이후 parser/save, 조판·렌더링, 편집·Studio,
CLI·agent, 보안, 성능과 릴리스 운영이 함께 누적된 큰 patch release다. 현재 `CHANGELOG.md`의
`Unreleased` 두 항목은 실제 포함 변경이며 그대로 v0.8.6 후보가 되지만, 전체 사용자 영향을 대표하지 못한다.
`CHANGELOG_EN.md`의 `Unreleased`는 비어 있어 Stage R3에서 한·영을 동시에 보강해야 한다.

이번 단계에서 release blocker로 확정된 사용자 회귀는 발견하지 않았다. 다만 이는 계보·변경 목록 판정이지 실행
검증 결과가 아니다. Rust release-test, Docker WASM, Studio/CDP E2E, package·archive 검사와 성능 확인은
Stage R4 게이트로 남아 있으므로 지금 단계에서 “회귀 없음”이나 “성능 영향 없음”을 선언하지 않는다.

## 2. 범위와 분류 방법

### 2.1 계보 규모

| 항목 | 결과 |
|---|---:|
| 커밋 | 2,214 |
| merge commit | 236 |
| 기여 PR provenance | 262 |
| 변경 파일 | 17,373 |
| `src/` | 774 |
| `rhwp-studio/` | 322 |
| `tests/` | 1,555 |
| `scripts/` | 264 |
| `.github/` | 20 |
| `mydocs/` | 4,460 |

commit subject의 큰 묶음은 `docs` 550, `fix` 548, `feat` 276, merge 227, `test` 206,
`refactor` 76, `ci` 24, `perf` 16이다. 이 수치는 commit 수이지 사용자 기능 수가 아니다. 통합 PR 하나가
여러 외부 PR을 provenance-preserving cherry-pick한 경우가 많으므로 제목 수를 기능 수로 해석하지 않는다.

### 2.2 다중 축 분류

262개 PR의 제목·label을 1차 분류하고 통합 PR의 commit·변경 경로를 교차 확인했다. 한 PR이 여러 기능군을
포함할 수 있어 아래 수치는 중복된다. 수치의 목적은 릴리스 노트 누락 탐지이며 기능량 비교가 아니다.

| 기능군 | 관련 PR | 대표 근거 |
|---|---:|---|
| parser/save | 41 | #4684, #4687, #4776, #4970, #5851 |
| layout/render | 66 | #4773, #5525, #5770, #6214, #6270, #6493, #6578 |
| editing/studio | 41 | #4699, #4717, #5719, #6290, #6461, #6558, #6562, #6564 |
| CLI/agent | 34 | #5185, #5192, #5569, #5672, #5917, #6540 |
| security | 8 | #4738, #4743, #4767, #6379, #6386, #6466 |
| performance | 25 | #5170, #5455, #5746, #6207, #6253, #6477 |
| packaging/operations | 182 | review·docs·test·CI 통합을 포함, #4824, #6297, #6573 |

모든 262개 PR은 적어도 하나의 축으로 disposition했다. `packaging/operations`가 큰 이유는 외부 PR을 받는
통합 PR, 검토 archive, 파생 test suite와 trusted-controller 계보를 독립 커밋·PR로 보존했기 때문이다.

## 3. 사용자 영향 변경 후보

### 3.1 Parser·개방·저장·왕복 보존

- HWPX curve를 `hp:seg` 체인으로 저장해 한글에서 열 때 발생하던 크래시를 막았다
  (#4684, merge `55eb2860ba`).
- HWP3 구역 정의 제어문자 위치를 한글 호환 순서로 고치고, HWP3 문자·OWPML 열거·숨은 설명·누름틀 범위
  때문에 저장 후 본문이 사라지던 경로를 보정했다 (#4687 `f7a98ce044`, #4776 `618794f9c7`).
- HWPX BinData storage id에 구멍이 있는 문서의 그림을 순번 축으로 복구한다
  (#4970 `b9efb7c203`).
- DOCTYPE이 붙은 HWPML과 Version 2.1 입력을 연다 (#5851 `65f71270f7`).
- `hp:ole` shape-component, `id`/`instid`, 0 크기 센티널과 음수 offset 보존을 추가했다. 현재
  `Unreleased` 항목의 포함 근거는 `d9f04c6eec1fee82ab3e574615b5cf5c77a55570`과
  `bef280b001b290768874ff164b8314fced6e25ea`다 (#4669, #5450).
- 중첩 표 검색·치환은 깊이 2 이상에서 `cellPath`를 내고 깊이 1의 기존 `cellContext` 봉투는 유지한다
  (`e948cb618df083c72a0d29bc784a46f4d235d64b`).

### 3.2 조판·렌더링·폰트

- 저장 RowBreak 표와 물리 frame, LineSeg, 자리차지 개체의 페이지 소유권·배치를 연속 보정했다
  (#4755, #4773 `b5d828af4b`, #5264, #5736, #5770 `f26c2e7ca4`).
- `--compat 2024`를 opt-in으로 추가해 한글 2024의 자리차지 표 앵커 줄 계상을 선택할 수 있게 했다
  (#5525 `9d352d56d3`). 기본 경로를 버전 번호로 자동 전환하지 않는다.
- exact font 커닝, bounded common shaping replay, variable instance와 vertical layout을 단계적으로 연결했다
  (#6214 `6415047a4d`, #6270 `1a43a507c9`, #6493 `3afbb066fe`, #6519).
- `HwpDocument.setExactFontInstance`·`clearExactFontInstance`의 정본 포함 SHA는
  `3005302519f97a884912750f0de9168d92a8bd2f`다. 등록된 exact slot에 대한 명시적 요청만 허용하며
  parser나 font 이름에서 axis를 자동 추론하지 않는다 (#4969).
- 빈 줄 TAC 그림, Square 겹침 그림, 표 폭·글상자 vpos·위첨자 advance와 투명선 표시 등 실제 오라클
  회귀를 보정했다 (#4774, #5770, #5772, #6302, #6578 `59bde68e66`).

### 3.3 편집·Studio

- 문서 전체 HTML과 Word `.doc` 내보내기, 차트 숫자 데이터 편집 UI를 추가했다
  (#4699 `e219134ae0`, #4717 `c653f7dcc5`).
- 한글 IME에서 `Ctrl+A`, 다른 이름 저장 뒤 문서명·최근 문서 갱신, 머리말·꼬리말 선택·편집 API를
  보정했다 (#4805, #6004, #6394 `d3b40a3d7c`, #6461 `cfa4ccacab`).
- 스킨과 첫 실행 선택 안내, 검증된 사용자 배율 원자 적용, 표준 인쇄 판형 스냅을 추가했다
  (#5719 `73939045e3`, #6290 `b1485e0a14`, #6562 `b33752594e`).
- 병합 셀과 일반 표 경계의 마우스 리사이즈 hit 경로를 고쳤다
  (#6558 `d776fdb287`, #6564 `07e1dd7ef6`).
- Chrome 다운로드 판정에서 `.xlsx`가 HWP viewer로 열리는 경로를 차단했다
  (#6547 `336c4526e9`).

### 3.4 CLI·agent·MCP

- 편집 명령 13건·29건과 조회 CLI 50여 개를 통합하고 `rhwp-q-pack`으로 조회 surface를 정리했다
  (#5185 `ba097d6bf9`, #5192 `e0851908bb`, #5672 `f9616a95fd`).
- 문서 에이전트 공개 명령 bridge와 HWP 2024 원격 MCP client를 추가했다
  (#5569 `b0c0a08394`, #5917 `2078a2629c`).
- CLI 도움말 index와 명령별 안내를 정비했다 (#6540 `a46ad1073c`).
- 명령 추가는 additive지만 JSON 봉투·조회 순서·중첩 경로는 Stage R4의 CLI 회귀 계약으로 다시 확인한다.

### 3.5 보안·입력 경계

- HWPX container와 parser 재귀 깊이에 상한을 두고 입력 경계·WMF 초기화를 보강했다
  (#4738 `d871bb8ce1`, #4743 `9a11240ced`, #4767 `b37c0a79ae`).
- 신규 sample 보안 sweep 범위를 실제 신규 입력으로 좁히고, HWP5 무인 스크립트 입력을 사전 차단하도록
  운영 안내를 보강했다 (#6379, #6386).
- Oracle PDF 자동 선택은 형식·생성 엔진·저장 제품이 불명확할 때 fail-closed하도록 바꿨다
  (#6466 `bd78a53122`, #6474).
- 공개 credit이 필요한 별도 보안 제보자는 이번 release range에서 확인되지 않았다.

### 3.6 성능·CI·패키징

- exact font source 준비·variable shaping cache를 bounded reuse하도록 구현했지만 최종 성능 주장은 R4에서
  비교 계측 전까지 보류한다 (`2465bb9775acb354101684354b62aba078f1c4fa`,
  `e7f814330dbc8c19cc8ef00462ea6b993aae74fb`).
- nextest archive를 단일 build 후 lib/integration과 A/B/C/D로 분할하고, CodeQL·문서-only·review-only
  재사용 경로를 보강했다 (#5170, #5455, #5500, #5576, #5746, #6207, #6253, #6477).
- trusted controller가 동일 merge tree와 review tail 증적을 재사용하도록 보정했다
  (#4824 `627c8c49ae`, #6297 `96da78a9c3`). #6243의 실제 post-release canary는 아직 남아 있다.
- Docker의 기존 GID 충돌을 처리하고 Linux AArch64 release binary target을 추가했다
  (#5758 `cd761884cc`, #6573 `a4d15ad6f7`). Linux AArch64 이슈 #5949는 실제 v0.8.6 asset 검증 전까지
  닫지 않는다.

## 4. `Unreleased` 포함 여부 대사

| 정본 항목 | release base 포함 근거 | disposition |
|---|---|---|
| exact font instance API | `3005302519f97a884912750f0de9168d92a8bd2f`, #4969 CLOSED | v0.8.6 이동 |
| HWPX `hp:ole` shape-component·id 보존 | `d9f04c6eec1fee82ab3e574615b5cf5c77a55570`, `bef280b001b290768874ff164b8314fced6e25ea`, #4669/#5450 CLOSED | v0.8.6 이동 |
| `CHANGELOG_EN.md` `Unreleased` | 항목 없음 | 위 두 항목과 나머지 사용자 영향 후보를 영문으로 신규 작성 |

현재 항목 중 다음 사이클로 이월할 것은 없다. 다만 두 항목만 이동하고 끝내면 parser/save, Studio, CLI,
보안, Linux AArch64와 주요 조판 회귀가 누락되므로 Stage R3에서 이 보고서의 대표 변경을 함께 편집한다.

## 5. 호환성·알려진 문제·비범위

### 5.1 호환성 판단

| 변화 | 호환성 판단 |
|---|---|
| exact font instance | 명시적 opt-in additive API, 기본 자동 추론 없음 |
| `--compat 2024` | 명시적 opt-in, 기본 조판 정책 유지 |
| HWPX/HWP3/HML 저장·개방 | 손상·유실 복구 목적. 저장 bytes는 달라질 수 있으나 의미 보존 방향 |
| 중첩 표 `cellPath` | 깊이 2 이상에만 추가, 깊이 1 `cellContext` 유지 |
| CLI·agent 명령 | additive surface. 기존 봉투 major 변경 선언 없음 |
| Chrome `.xlsx` 차단 | 잘못된 자동 열기 범위를 좁히는 의도된 동작 변화 |

현재 인벤토리에서 공개 major envelope 변경이나 명시적 breaking change는 확인되지 않았다. 이 판단은 R4의
빌드·계약 시험이 통과해야 확정할 수 있다.

### 5.2 열린 후속과 release gate

- #5949: OPEN. v0.8.6 GitHub Release에 실제 Linux AArch64 asset이 올라가고 ELF·실행권한·버전을 확인한 뒤
  닫는다.
- #6243: OPEN. `main` trusted controller와 실제 post-release Render Diff canary 성공 뒤에만 닫는다.
- #6584: OPEN. 공식 채널 정합과 릴리스 후속 정산이 끝날 때까지 유지한다.
- #4960·#4969: 2026-08-31 CLOSED. 이번 릴리스에는 구현 결과와 bounded guard를 포함한다.
- Draft #5953, #6458과 stacked Draft #6467은 release base에 포함하지 않는다. 특히 #6458/#6467의 Studio
  zoom topology·budget 동작을 v0.8.6 기능으로 광고하지 않는다.
- R4가 아직 실행되지 않았으므로 현재 알려진 실행 위험은 “미검증 release candidate”다. 실패가 나오면 이
  inventory의 알려진 문제 또는 이월 disposition으로 갱신하고 R3 기록을 다시 맞춘다.

## 6. Stage R3 편집 골격

한·영 CHANGELOG와 GitHub 릴리스 노트는 다음 순서를 공통으로 사용한다.

1. 조판·렌더링·exact font와 bounded shaping
2. HWP/HWPX/HML 개방·저장·왕복 보존
3. Studio 편집·내보내기·표·머리말/꼬리말 UX
4. CLI·agent·HWP 2024 MCP
5. 보안·입력 경계
6. 성능·CI 신뢰성
7. Linux AArch64와 배포
8. 호환성·알려진 문제
9. 기여자 20명

세 공개 기록은 기능군의 의미와 기여자 credit-key 집합을 일치시키되, 내부 review archive나 모든 CI 정정
커밋을 사용자 변경처럼 나열하지 않는다.

## 7. Stage R2 판정

- 262개 PR provenance가 적어도 한 분류 축에 disposition됐다.
- 기존 `Unreleased` 두 항목 모두 포함 SHA와 CLOSED 이슈가 확인됐다.
- 대표 사용자 영향, 호환성, Draft 비범위, #5949/#6243 후속 경계가 분리됐다.
- R4 전에는 회귀·성능 성공을 주장하지 않는 보호 불변식을 유지했다.

따라서 Stage R2 변경 인벤토리는 완료 조건을 충족하며 Stage R3 버전·CHANGELOG·릴리스 노트 편집의 입력으로
사용할 수 있다.
