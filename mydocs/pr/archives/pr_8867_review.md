# PR #8867 리뷰 — PDF 진단과 qcms 테스트 포인터 경계

## 최종 판정

**승인.** code head `bde7035ab6ae288c68da65b582a835bf95a7681d`의 승인 범위 보정·필수 로컬 검증·GitHub CI와 최종 정책 감사가 모두 통과했다. 문서-only trailing head의 실제 CI·fast-pass·충돌 없음과 exact head를 재확인한 뒤 병합한다. main의 8개 OPEN 경고 전체 해소나 #597의 추가 분류는 이 판정에 포함하지 않는다.

## 접수와 승인

- PR: https://github.com/edwardkim/rhwp/pull/8867
- 작성자: `edwardkim`, 본인 PR의 self-review. reviewer 요청 없음.
- base: `devel`, 검증 기준 `edbd51067730098e830eb5d03aed625eac47b10f`.
- 검증 code head: `bde7035ab6ae288c68da65b582a835bf95a7681d`.
- 작업지시: 2026-11-09 “PR 로 머지 처리합니다.” PR 제출·CI·self-review·devel merge·종료 정리를 진행한다.
- route: `collaborator_self_merge`; 보조 경로는 intake, local_validation, review_only_fast_pass, post_merge, rework_and_exceptions의 대형 증적 범위다. 실행 정책 변경은 없으므로 동일 PR의 문서-only trailing 경로를 적용한다.
- 제출 전 본인 OPEN PR 0건, source/입력 hash 일치, 후보의 최신 base 병합 시뮬레이션 종료 0을 확인했다.

## 변경과 선행 조사

완료된 #8745·#8750은 인증·query·Debug 경계의 선행 보정이며 이번에 재오픈하지 않는다. 당시 남긴 vendor·테스트 경고를 개별 검토했고, 114건의 운영 분류를 별도 승인·GET 재확인으로 완료했다. 기존 dismissed 69건·fixed 10건은 보존했다.

이번 코드는 PDF의 두 진단에서 전체 객체·복호화 본문을 제거하고 종류·길이·쪽 번호를 유지한다. qcms C bindings 테스트의 일곱 nullable transform을 checked reference로 바꾼다. 선택 기능을 실제 실행하는 데 필요했던 기존 LUT helper `Option` 반환형도 C adapter에 연결해 예산 실패 시 호출자 배열을 보존한다.

## 로컬 검증과 재사용 경계

- 최신 base 전체 release-test: **13,130 PASS, 0 FAIL, 45 skipped**.
- 집중: PDF 로그·원문/캐시 보존 2건, 기존 인증 경계 6건, 새 base 글꼴 신원 4건, 번호 들여쓰기 2건 PASS.
- format, native root Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy, suite 정책 PASS.
- 최소 글꼴 격리에서 공개 기본 글꼴 5개만 노출했으며 실제 목록을 기록했다.
- qcms 선택 기능 30 PASS는 동일 standalone source·codec 자원 dependency·toolchain·manifest의 기존 실제 실행 증거를 재사용한다. 기존 null 경로의 SIGSEGV와 보정 후 두 null 검사, 예산 실패·정상 LUT를 구분했다.
- 추가 standalone vendor Clippy는 기존 스타일 125건으로 실패하며 기준 대조군과 같고 새 진단은 0건이다. 정식 root lint의 `-D warnings` PASS와 구분한다.

근거: [검증 원장](../../tech/investigations/code-scanning-20261109/README.md), [최신 base 검증](../../tech/investigations/code-scanning-20261109/pr-local-validation.json), [런타임 전후 대조](../../tech/investigations/code-scanning-20261109/runtime-validation.json).

원래 기준 `e00f9b7e26` 위 source `612b4053db`는 최신 devel의 #8857을 반영해 `913713dbfd`로 리베이스됐다. 충돌·보정 파일 중복 변경은 없었고 여섯 보정 source hash가 보존됐다. 새 base의 실제 두 계약과 전체 회귀를 실행했으며 source·test·CI 정책을 추가로 수정하지 않았다. 새 base의 조판·글꼴 source는 후보와 같고 이번 변경은 그 계산 경로를 바꾸지 않는다.

## 검증 입력 커밋 확인

**충족.** 공개 PDF control/empty-password, 기존 암호/평문 HWPX, 새 base의 HWP와 명시적으로 공급한 font fixture 등 6개를 실제 working bytes와 Git commit bytes로 대조했다. 경로·바이트 수·SHA-256은 `pr-input-fixtures.json`에 있다. 파일 없이 합성·소비한 PDF는 회귀 코드에 보존돼 있다. qcms ICC는 고정 upstream commit·Git blob·SHA-256 원장을 따른 비문서 테스트 입력이며 배포 파일로 추가하지 않았다.

## 조판 원칙 준수 검토

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬 검증 | 충족 — 위 최신 base 필수 실행과 source/input hash 증거 |
| 기존 이슈·해결 PR의 선행 분석 | 충족 — #8745·#8750 완료 범위, 원 source/sink 도입과 최초/현재 SARIF를 구분 |
| 구현 근거·반례 | 충족 — 실제 로그의 원문·캐시 보존/비노출과 nullable 실패 경계의 수정 전 FAIL·수정 후 PASS |
| 리베이스 반영 기능 보존 | 충족 — source 경계 동일성, 새 base 글꼴 신원·들여쓰기 6건 및 전체 회귀 |
| 측정·배치·분할·줄 소속 | 비해당 — 그 source/계산과 baseline을 변경하지 않음 |
| 최종 출력 소비 | 충족 — PDF decoded bytes·캐시 반환값, C LUT 예산 실패·정상 결과의 실제 경로 확인. 시각 출력 일치를 주장하지 않음 |
| 렌더링 회귀 선행 조건·확대 판독 | 비해당 — 새 검사는 로그/포인터 계약이며 렌더링·시각 fixture 또는 golden을 추가/갱신하지 않음 |

## CodeQL과 잔여 범위

초기 base의 전체 경로·동일 경고 규칙 17개 분석에서 원문 SARIF는 144→137건이고 포인터 경고는 7→0건이다. 보정 이외 주요 finding은 유지됐으며 같은 소스의 RNG 경로/메시지 순서 차이는 별도 원장으로 보존했다. 리베이스 뒤 vendor·codec 자원 dependency와 보정 source hash는 같다. 이 정량을 새 base의 전체 경로 분석으로 표시하지 않는다.

현행 GitHub PR Rust CodeQL은 `rust-pr.yml`의 src/crates/rhwp-desk/build.rs 범위다. vendor 전체 분석을 대신하지 않는다. #597의 잔여 4개 flow는 `page_index: usize` 수치 진단으로 이어지며 실제 본문 로그는 제거됐다. 추가 분류·main 승격·protected ref 전체 분석은 이번 PR merge와 구분하고, 그 전에는 원격 경고 8건이 모두 닫혔다고 주장하지 않는다. 탐지 query/path/language/severity·sanitizer·CI 실행 정책은 변경하지 않았다.

## Code head CI

2026-11-10 00:50:00 KST에 exact head의 전체 check 완료를 확인했다. 실패·취소·대기 없음, Build & Test와 적용 제품 검사 모두 success다. Native·Archive A–E·Frontend·lint의 실제 실행을 확인했고 수동 재실행이나 우회는 하지 않았다.

| Workflow | Run | 결과 |
| --- | --- | --- |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/42642910113) | `42642910113` | success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42642910181) | `42642910181` | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42642910151) | `42642910151` | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42642909992) | `42642909992` | success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42642910008) | `42642910008` | success |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/42642910059) | `42642910059` | success |

- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/42648501012): **success**.

[head별 check·실행 URL·검증 시각](../../tech/investigations/code-scanning-20261109/pr-code-ci.json)을 보존했다. 정책상 skip된 비적용 단계와 실제 제품 검사 성공은 구분했다.

## Merge 전 조건

code head CI 성공을 확인했으며 이 review·오늘할일을 동일 PR의 문서-only trailing commit에 포함한다. 최신 base merge tree·링크·오늘할일 보존을 push 전에 확인하고, trailing head의 실제 CI·fast-pass·mergeability와 exact head를 다시 확인해 일반 merge한다. GitHub merge SHA는 PR과 devel Git 이력에서 확인하며 보호 설정을 변경하지 않는다.
