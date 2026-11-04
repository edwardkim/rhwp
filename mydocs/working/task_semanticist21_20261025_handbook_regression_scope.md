# 이번 PR의 편람·sample16 회귀·검토 범위 제외

## 사용자 지시와 경계

2026-10-25 사용자는 다음 두 원본의 모든 기존 회귀 검사를 제거하고 다른 PR에서 새로
추가하도록 지정했다. 상수·helper·동적 파일 순회·fixture 생성기·Studio E2E도 추적하며,
두 문서는 이번 PR의 검토·시각 검증·회귀 검증 범위에서 제외한다.

- `samples/2025 행정업무운영 편람(최종).hwp`
- `samples/2025 행정업무운영 편람(최종).hwpx`
- `samples/hwp3-sample16*` 원본·변환본 계열: 앞서 제거한 회귀는 복원하지 않는다.

이는 해당 문서의 기존 기대값이 모두 틀렸다는 판정이나 출력 보정 완료가 아니다.
원본·독립 PDF·기존 증적과 렌더링 구현은 유지한다. 삭제한 검사 코드의 별도 사본은
만들지 않는다. 제거 전 코드는 `d3344f46b3`의 Git 이력에서 확인할 수 있다.
이번 PR의 검증 완료 문서·종료 이슈에는 포함하지 않는다.

## 실행 경로별 제거·차단

| 경로 | 처리 |
| --- | --- |
| 전용 Rust 회귀와 상수·helper 호출 | 전용 파일·함수를 제거하고, 혼합 검사는 편람 항목만 제거 |
| parser·serializer·CLI·LLM export의 실문서 사례 | 편람을 읽던 사례를 제거하며 다른 문서의 계약은 유지 |
| 공통 corpus·baseline·디렉터리 순회 | 공통 제외 목록을 세 Rust 수집 진입점에 적용하고 해당 원장 행 제거 |
| same-id 글자 모양 registry·JSONL | 편람 34행을 제거하며 나머지 행의 바이트를 유지 |
| char-shape fixture 추출·Rust table 생성기 | 공통 제외 목록을 읽어 재생성을 차단 |
| PDF residual fixture와 생성기 | 편람 두 corpus의 766쪽·6,128개 합성 사례 및 sample16 여섯 별칭 corpus의 96쪽·768개 합성 사례와 전용 catalog 제거 |
| 보안 inspect fixture와 생성기 | 편람 envelope·catalog 행·생성 입력 제거, 공통 제외 적용 |
| Studio E2E | 편람 전용 두 파일·npm 진입점 제거, #4741의 문서 부분만 제거하고 글꼴 probe 유지 |
| Studio 로딩 helper | 공통 제외 입력을 실제 로딩 전에 거부 |
| WASM/Canvas WMF crop 검사 | 편람 전용 검사와 browser suite import 제거 |
| 보안 Gym·기준풀이 | 편람 전용 SE13·SE20·SE21·SE32·SE41·SE42·SE53·SE66과 짝 기준풀이 제거 |
| 에이전트 벤치 | `t08_structure_report` 과제와 전용 기준풀이 제거 |
| agent bulk/recipe fixture와 생성기 | handbook 별칭·J20·T04/T18/T19·발췌 두 건과 실행 목록 제거, 생성 원본도 정리 |
| 기본 oracle 수집과 저장 manifest | 실제 문서 수집 및 manifest 읽기에서 공통 제외 적용 |
| HWP control 검증 probe | 편람을 여는 두 전용 probe 제거 |

`tests/fixtures/deferred_regression_documents.txt`가 두 편람 문서와 sample16 계열의 공통 제외 목록이다.
편람은 정확한 파일명, sample16은 basename 접두어로 판별한다.
Rust·Python·JavaScript helper가 이 목록을 소비한다. 시험지나 다른 2025년 문서로
범위를 확장하지 않는다. 정상 문서의 번호·크기·내용·주소·IR 계약과 일반 합성 계약은
유지한다. 실제 파일을 열지 않는 사양 설명·과거 보고서·일반 파일 분류용 mock은
문서 회귀 실행 경로와 구분한다. 보존한 PDF·oracle 메타데이터는 검증 완료를 뜻하지 않는다.

## 검증 계획

임시 증적 루트는 `output/pr-review/semanticist21-20261019/`다.

1. 제거 전후 파일·함수·혼합 입력과 Git blob을 대조한다. 입력 HWP/HWPX는 같은지 확인한다.
2. Rust 전체 integration source를 컴파일하고 남은 해당 모듈의 정상 검사를 실행한다.
3. 경로 표기별 공통 수집 경계, 실제 디렉터리 순회와 저장 manifest를 확인한다.
4. 생성기의 출력 경로를 ignored 임시 디렉터리로 바꾸어 제외 문서가 재생성되지 않는지 확인한다.
5. Studio helper·실행 목록·browser suite·에이전트 fixture·Gym 짝 기준풀이를 대조한다.
6. 코드·JSON 문법, Rust 형식과 자동 suite 원본·실행 목록을 확인한 뒤 함께 커밋한다.

아직 실행하지 않은 항목을 완료로 기록하지 않는다. 이번 문서 제외와 별개인 기존
`hf_cross_section_inherit.hwp`의 재저장 배치 실패는 유지하며 후속 PR 준비에서 판정한다.

## 제거 내역과 검증 결과

편람 제거 계획에서 Rust 검사 함수 59개와 혼합 배열 항목 7개를 제거했으며, 전용 Rust 파일
16개를 삭제했다. 정상 문서를 함께 검사하던 파일은 유지했다. 이전 sample16 제거 뒤 남았던
호출되지 않는 전용 helper 9개도 제거했다. `tests/issue_1105.rs`의 k-water 검사 3개와
글꼴 프로그램 소유권 검사는 유지한다. 원본 파일 10개를 `d3344f46b3`의 바이트와 대조했다.

아래 로그 경로는 모두 임시 증적 루트 기준이며 Git에 커밋하지 않는다.

| 검사 | 실제 결과 | 증적 |
| --- | --- | --- |
| Rust 전체 integration source 타입 검사 | PASS | `logs/scope-final-rust-3800.log` |
| 제거 후 영향 모듈의 정상 Rust 검사 | 107개 PASS, 실패 0개 | 같은 로그와 `scope-final-rust-3800-result.json` |
| sample16 전용 미사용 helper 제거 후 정상 검사 | 21개 PASS, 실패 0개 | `logs/scope-helper-rust-3807.log` |
| Rust 제외 경계 | 22개 경로 사례 PASS | 같은 로그의 standalone probe |
| Studio 공용 로딩 경계 | 제외 입력 10개 × 경로 표기 3종 거부, 정상 대조군 허용 | `logs/scope-js-3801.log` |
| Canvas 실제 브라우저 | 7개 PASS, 실패 0개 | `logs/scope-canvas-browser-3802.log` |
| Oracle 도구 unit | 120개 PASS | `logs/scope-oracle-unit-3803.log` |
| suite 자동 수집 unit | 23개 PASS | `logs/handbook-suite-unit-3799.log` |
| 변경 파일 문법·실제 char-shape 수집 main | 21개 문법 검사, 정상 입력 1,256개 선택, 제외 원본 10개 불변 | `scope-final-audit-3806-result.json` |
| fixture 생성기·수집 경로 | PASS; 임시 출력에서 편람·sample16 재생성 없음 | `handbook-generator-audit-3798-result.json` |

전체 raw char-shape dump는 현재 작업 트리에 없어 전체 원시 자료의 재생성을 실행한 것으로
보고하지 않는다. 생성기의 선택 경계와 최소 정상·제외 fixture, 실제 재귀 입력 목록을 각각
검사했다. 남은 문서만 실행하는 residual PDF 표본은 12개로 고정했으며 PDF 자체의 2쪽
구조 검사와 다른 문서의 page lock은 유지한다. 허용 오차나 시각 점수 기준은 바꾸지 않았다.

이번 제외와 관련 없는 renderer static contract 실패와 `hf_cross_section_inherit.hwp`의
재저장 배치 실패는 남아 있다. 이 제거 작업의 통과를 전체 PR 검증 완료로 간주하지 않는다.

helper 제거 후 suite 크기 변화로 파생 harness가 달라져 `--prepare`로 갱신했다.
파생 harness는 Git에 추가하지 않았으며 최종 `--check`와 정상 검사 21개를 통과했다.
