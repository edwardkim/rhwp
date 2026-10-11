---
kind: working
status: completed
last_verified: 2026-10-11
---

# #7820 3단계 — 전체 회귀와 Rust lint

메인테이너가 원격 CLI 직접 출력 성공을 확인한 뒤 전체 회귀·Native Skia 검사·Rust lint
실행을 승인했다. [2단계](task_m100_7820_stage2.md)의 Docker WASM·전쪽 TSV·터미널 판정을
이어받는다. 최신 `upstream/devel`을 fetch했으며 비교 base는
`790a6f8f388445e7bff2bcb165aa7b384f9ece80`, 구현 source는
`dc72a324923c404fb38a46667fc841304f3669e9`이다.

기본 작업공간의 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 사용하며
같은 target의 Cargo 작업을 순차 실행한다. 실행 원문·정책 base·종료 코드는 ignored
`output/pr-review/issue7820-20261011/full-validation/`에 보존한다.

## 실행 범위

파생 integration suite 준비, fmt, 기본·WASM·workspace all-target Clippy와 workspace build,
고정 base의 manifest 검사, Native Skia feature Clippy, release-test 전체 nextest,
Native Skia lib·missing-picture·direct-PDF 검사와 새 Sixel Native 계약 검사를 실행한다.
아래 최종 결과는 실행 원문과 연결한다. generated suite·manifest는 stage하지 않는다.

## 최초 lint 결과와 보정

fmt·기본 Clippy·WASM Clippy·workspace build·workspace all-target Clippy·manifest 검사는
통과했다. 추가 Native Skia all-target Clippy에서 `items_after_test_module`을 검출했다.
`renderer/skia/renderer.rs`의 `draw_arrow_heads`가 테스트 모듈 뒤에 있는 기존 배치이며
고정 base에도 함수 내용과 배치가 같다. allow 속성을 추가하지 않고 함수 전체를 테스트
모듈 앞에 옮겼다. `helper-move.json`으로 base와 함수 본문 동일성을 확인했으며
조판·그리기 동작이나 테스트 기대값은 바꾸지 않았다. 보정 source에서 lint·회귀를 다시
실행했고 최초 실패 원문은 보존했다.

helper 이동 뒤 필수 lint와 추가 Native Skia lint는 모두 통과했다. 전체 nextest 최초 결과는
10,570건 중 10,569건 통과·1건 실패·50건 skipped다. 실패한
`agent_codex_contract::every_capability_command_has_a_codex_chapter`는 새 명령의
대전 장 누락을 정확히 검출했다. 생성기의 변환·렌더 가족에 `export-sixel`을 등록하고
같은 `command_chapter`·`write_if_changed`로 해당 장만 바이너리 자기서술·명령별
상세 도움말에서 생성했다. 기존 장은 그대로 보존했다. 생성 절차는
`agent-codex-regeneration.json`, 최초 실패는 `helper-move/nextest-all.log`에 있다.
Python 생성기 계약 14건과 보완 후 agent Codex 계약 2건이 통과했다.
테스트 조건을 완화하지 않고 누락 문서를 보완했으며 전체 회귀를 다시 실행했다.

## 최종 검증 결과

최종 검증 head는 `c95269728f4d06e27b503d3d01d906e02e0ea07e`다.
Rust 보정 commit `11076741a` 이후 변경은 생성기·생성 문서·작업 기록뿐이며
`src`·`crates`·`tests`·Cargo manifest/lock·workflow는 동일하다. 따라서
`helper-move/results.json`의 lint 결과와 `codex-fixed/results.json`의 최종 회귀 결과를
이 head에 연결한다. 검증 base는 위의 `790a6f8f...`로 고정했다.

| 검사 | 결과 | full-validation/ 증적 |
| --- | --- | --- |
| 파생 suite 준비·fmt/check·기본 Clippy·WASM Clippy·workspace build·all-target Clippy | 모두 통과 | `helper-move/results.json` 및 각 로그 |
| manifest·source unit tier 정책의 고정 base 비교 | 통과 | `helper-move/manifest-check.log`, `helper-move/unit-tiers-check.log` |
| 추가 Native Skia all-target Clippy | 통과 | `helper-move/clippy-native-skia.log` |
| 전체 release-test nextest | 10,570건 통과·50건 skipped | `codex-fixed/nextest-all.log` |
| Native Skia lib (workspace 기본 member 포함) | 4,109건 통과·13건 ignored | `codex-fixed/native-skia-lib.log` |
| missing-picture placeholder | 2건 통과 | `codex-fixed/native-skia-placeholder.log` |
| direct-PDF export | 4건 통과 | `codex-fixed/native-skia-direct-pdf.log` |
| 새 Native Sixel 계약 | 7건 통과 | `codex-fixed/native-sixel-contract.log` |
| 에이전트 대전 생성기 Python 계약 / Rust 대전 계약 | 14건 / 2건 통과 | `agent-codex-python.log`, `agent-codex-fixed.log` |
| 전체 CLI·명령별 도움말 | Sixel 항목과 세부 옵션 표시 확인 | `cli-help-verification.json`, `cli-global-help.txt`, `cli-sixel-help.txt` |

로컬 nextest 0.9.137은 프로젝트 최소 0.9.91을 충족한다. 권장 0.9.140과 다르며
CI 전용 JUnit의 미사용 설정 키 경고는 남아 있지만 default profile 검사의 실패는 없다.
skipped·ignored 항목을 실행 완료 건수에 포함하지 않았다.

## 최종 산출물과 시각 대조

최종 Native 개발 CLI는 `candidate/rhwp`이고 SHA-256은
`2318d7e6b718ccde00cf0e80a1fa5212dfd24a471626f4e1230f9294f6bde512`다.
`final-binary-provenance.json`에 source와 이전 바이너리를 연결했다.
4문서 전체 11쪽의 출력 폴더·쪽 선택·stdout/파일 바이트 일치와 잘못된 경로·쪽 번호의
오류 계약을 재확인했다(`final-file-output-results.json`).
11쪽을 너비 360·720으로 내보낸 22개 Sixel 스트림을 libsixel로 독립 디코딩했다.
모든 크기·framing 검사와 양자화 예산을 통과했고 RGB 평균 절대 오차 최댓값은
0.31817/255, 채널 오차 p99 최댓값은 5/255다
(`final-independent-pages-results.json`). 기존 PNG의 decoded 픽셀은 고정 base와 같다.

최종 head에서 Docker WASM wrapper를 다시 실행하고 pkg/Studio public 동기화를 확인했다
(`docker-wasm-final.log`, `docker-wasm-final-provenance.json`). WASM 대상에 영향 없는
Native 함수 이동과 문서 변경이므로 Rust 컴파일 캐시를 재사용했으며,
wasm-bindgen·wasm-opt·패키지 생성은 완료했다(전체 3분 6초).
JS·WASM 해시는 2단계 빌드와 동일하다.

최종 Native PNG를 다시 생성해 이전 출력과 전체 decoded 픽셀이 동일함을 확인하고
새 전쪽 TSV를 산출했다(`final-native-png-results.json`, `final-native-scores/`).
Docker fresh WASM도 같은 원문·독립 한컴 Print PDF로 새 전쪽 TSV를 산출했다
(`fresh-wasm-final-sweep/summary.json`). 전체 11쪽에 누락·쪽수 불일치·90% 미만은 없다.

| 문서 | 쪽수 | 최종 Native 최저값 | 최종 Docker WASM 최저값 |
| --- | ---: | ---: | ---: |
| table-core | 3 | 96.18681% | 97.41234% |
| footnote | 6 | 96.77648% | 98.15558% |
| image-crop | 1 | 99.83013% | 99.83033% |
| compound | 1 | 99.87079% | 99.87578% |

`final-inspection/`의 최종 PDF·Native·WASM 비교 PNG와 표·각주 확대 PNG를 직접 판독했다.
표의 셀 경계·본문 순서·그림 경계·화살표의 주요 위치는 보존됐다. 2단계에 기록한
글꼴 형태·굵기, 회색 괘선·각주선, compound 하단 `id` 표시 차이는 그대로 남아 있다.
이번 변경이 해당 조판 차이를 해결했다고 주장하지 않는다. 실루엣 전용 실행의
`not_evaluated`도 자동 승인으로 취급하지 않는다. 새 실물 조판 golden은 추가하지 않았다.

원격에 설치한 이전 개발 CLI의 전체 `--help`에도 `export-sixel`이 표시된다.
이번 후속 보정은 동일 함수의 이동과 문서 추가이며 원격 바이너리를 교체하지 않았다.
WezTerm·Windows Terminal의 사용자 직접 표시 판정은 2단계에 연결한다.
Foot 실행과 터미널 버전 확인은 미검증으로 남긴다.

이번 승인은 로컬 검증 범위다. 원격 push·PR 생성·merge·이슈 close는 수행하지 않는다.
