---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase2-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase2-report.md` · 원문 SHA-256: `e2ec65df5b167e43a4b273ffd7c02bfdc5cab820e84a2b4d0d25ca7832b92c10`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 추가 조사 — 실행 경로 전건 연결·88그룹 의미 분류·새 MCP Print 5조건

아직 전수 의미 심사가 완료된 것은 아닙니다. 숫자 목록만으로 조사를 완료 처리하지 않고 실제 모듈·매크로·독립 PDF 연결을 보완했습니다. 제품 코드와 테스트는 아직 수정하지 않았습니다.

### 최신 작업지시 반영

- 전수 조사 결과를 이슈에 업데이트한 뒤 지침과 다르게 절대 px로 고정한 기존 검사를 해결합니다.
- **보정하는 테스트 파일·함수는 `issue_8232` 기준의 새 이름**으로 변경합니다. 이전 이슈 번호를 이름에 남기지 않습니다. GitHub의 기존 이슈를 일괄 개명하는 뜻으로 처리하지 않습니다.
- PR은 나중에 진행합니다. 현재 조사용 스크립트·로그·PDF·PNG·TSV는 모두 ignored `output/issue8232-audit/` 아래에만 있습니다.

### 1. Native 등록 11,639건의 정의 경로 연결 완료

Cargo metadata의 workspace target에서 인라인/외부 `mod`, `#[path]`, `include!`, 파일 수준 `#![cfg]`, `cfg_attr(..., test)`와 `proptest!`를 추적했습니다.

- 파싱한 모듈 파일3,327개, 모듈 연결3,291개, 미해석 모듈 연결0개입니다.
- 현재 기본 feature Native 등록 **11,639건 전부**를 원본 정의에 연결했습니다: 직접 정의/조건부 attribute11,507건 + partition/proptest 매크로 생성132건. **등록 경로 미연결0건**입니다.
- 이 수치는 PASS가 아닙니다. 이번에 전체 회귀를 다시 실행하지 않았습니다.
- 기본 구성에 등록되지 않는 조건부 정의223건도 원장에 남겼습니다. 208건은 `native-skia` 조건을 포함합니다. 나머지는 WASM 전용, subsecond-dev, PDF feature-off/추가 codec, debug_assertions, macOS 제외 unix 조건입니다. 등록되지 않은 조건을 삭제나 유효 검사 누락으로 추정하지 않았습니다.
- 초기의 단순 `#[cfg(test)]` registry 차이만으로 실제 누락을 판단하지 않습니다. 예를 들어 CanvasKit7건은 `include!`로 실제 호출되고, kerning11건은 `cfg_attr`로 Native 테스트가 됩니다.

근거: `module-graph-tests.json`, `module-graph-gaps.json`, `runtime-macro-expansions.json`, `runtime-source-reconciliation-summary.json`.

### 2. 우선 후보88그룹의 기하 assertion 의미 분류

| 분류 | 그룹 수 | 의미 |
| --- | ---: | --- |
| 절대 페이지/PDF 수치 고정 후보 | 55 | 동일 원본·쪽의 현재 raw·Print 기대값 확인 필요 |
| 기존 현재 TSV에서 raw98% 수치 요건 확인 | 10 | 직접 판독·기대값 출처·허용폭 확인까지 완료했다는 뜻은 아님 |
| 원본 단위/상대 관계 | 13 | HU 치수·실제 객체 간 관계·편집 전후 상대 이동 등 |
| 원본 기하라고 설명한 literal, 추가 근거 필요 | 7 | 단순 과거 출력 상수인지 독립 원본 단위의 환산인지 구분 필요 |
| PDF 원본/편집·저장 계약 | 3 | authored PDF 좌표/변환 또는 전후 관계이며 한컴 실물 좌표와 구분 |

이88개는 최초 literal 후보에 대한 의미 분류이고, 전수 조사 전체의 최종 부적합 건수가 아닙니다. 상수·튜플 변수 뒤에 숨은 좌표처럼 이 목록 밖에서 확인한 대상도 함께 조사합니다.

소스 내부에서 실물 입력이 연결된22그룹도 확인했습니다. `src/renderer/layout/integration_tests.rs`의 #3257 그림x129.8±2와 TAC 합성 문서의 CELL baseline122.1±2는 독립 좌표 확인 대상입니다. 같은 TAC 검사의 NEXT161.3은 원본4160HU와 저장 baseline에서 유도한 축이어서 분리했습니다. 배경 그림의 page(0,0)·출력 bitmap 크기·소속/순서·HU 저장 계약을 동일하게 취급하지 않습니다.

근거: `priority-semantic-reviewed.tsv/json`, `source-input-semantic-reviewed.json`.

### 3. MCP 새 독립 Print와 현재 Native/fresh WASM 전쪽 비교

5개 입력 모두 `rhwp info --json`의 저장 제품이2022 이하임을 확인하고 engine2020·timeout1800으로 출력했습니다. 실제 한컴 **11.0.0.9136 / hwp-managed-direct-dll-host / Print method0 / hancom2020_pdf_driver_one_up / preprocess none**을 status와 download에서 확인했습니다. 기존 PDF는 덮어쓰지 않았습니다.

현재 source head `238de6de41a208648f4ade6b494fe433a81c1fce`에서 Native Skia 빌드 exit0, locked wrapper fresh WASM 빌드 exit0을 확인했습니다. Mac WASM은 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화로 주장하지 않습니다. root pkg와 감사용 고정 pkg의 실제 JS/WASM hash가 같습니다. 새 WASM hash는 `0a81d25f3e910d91d9b89fa5a06dbcb5cd6f38d126cb2caf3b1abb27bffe118d`입니다.

현재 canonical Visual Sweep의 **Native Skia PNG / fresh WASM Print Canvas PNG / PDFium / 96 CSSpx / 2px 관용 / page-workers8**로 전쪽 비교했습니다. 이 값은 새5조건의 결과이며 기존35조건 TSV의 일부를 다시 산출했다고 쓰지 않습니다. 비교 PNG에서 canonical TSV를 별도 산출했으며 입력 PNG hash를 기록했습니다.

| 입력 조건 | 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 검사 대상 쪽과 해석 |
| --- | ---: | ---: | ---: | --- |
| `samples/issue6312/fiscal_trend_float_table_anchor.hwpx` | 4 | 98.63756% | 98.63321% | ±0.3 높이 검사의1쪽은 **양쪽100%**. 단순히0.3이라는 이유로 미달 사례로 잡으면 잘못된 판정 |
| `samples/issue3257/webhangul_product_spec_v1.1.hwp` | 6 | 98.84999% | 98.84886% | source 내부 x129.8 검사의4쪽이 전쪽 최저. 양쪽98% 수치 요건 충족 |
| `samples/hwpx/form-01.hwpx` | 1 | 100.00000% | 100.00000% | combo x/baseline±0.4. 수치 요건과 지침상 허용폭 심사를 구분 |
| `samples/tac-host-spacing.hwpx` | 1 | 100.00000% | 100.00000% | source 내부CELL122.1/NEXT161.3 확인 대상 |
| `samples/issue2470/36382471_masked.hwpx` | 2 | **77.19425%** | 100.00000% | 1쪽 Native만 미달. 원본 데이터·실제 표 좌표 관계를 추가 구분해야 함 |

Native/fresh WASM 각각14쪽이며 누락·중복·쪽수 불일치가 없습니다. 마지막 입력의 Native 미달은 그대로 실패로 보존했습니다. Native 전체 통과 또는 양쪽98% 통과로 묶지 않았습니다.

#### #7150 입력의 미달 원인 구분

해당 HWPX ZIP에는 BinData가 없고, 그림2개의 `binaryItemIDRef`가 빈 문자열입니다. 직접 review에서 **Native만 회색 missing-image 표식2개를 인쇄하고, 한컴 Print와 WASM Print는 그 잉크를 표시하지 않는 것**을 확인했습니다. `src/renderer/skia/renderer.rs`의 Image 데이터 결측 fallback이 회색 표식을 그리는 소비 경로도 읽었습니다.

따라서77.19425를 곧바로 표 원점 오류나 글꼴 차이라고 설명하지 않습니다. 반대로 표 원점 검사가 PASS라는 이유로 이 페이지의 미달을 면제하지 않습니다. 이 입력의 고정px값은 원본 줄/여백·셀 선언·공유 기준선으로 독립 유도한 계약인지 별도 추적 중입니다. 구현 보정 필요 여부와 적절한 관계 검사를 조사 후 결정합니다.

현재 검사의 실제 동작도 확인했습니다. #6312의±0.3 검사1건과 #7150 모듈5건은 **6 PASS /0 FAIL,0.068초**입니다. 이는 전체 회귀나 시각 일치 통과를 의미하지 않으며, 기존 좌표 검사가 PASS여도 Native Print 미달이 남는 사례입니다. 필터로 제외한11,633건을 실행 결과에 합산하지 않습니다.

### 4. 남은 조사

- 전체2,707후보의 나머지 의미 심사, 상수/다른 파일 helper/동적 입력 연결을 계속합니다.
- 입력 경로 해석을 보강해1,136그룹에서 실제 명시 경로를 확인했습니다. basename 유일 후보102참조, basename 중복19참조, 동적 glob71참조, 미해석동적15참조 등은 런타임 대응으로 단정하지 않았습니다.
- fixture·baseline/golden의 구조 자료4,418파일 경로를 추가 수집했습니다. 선언 숫자와 실제 소비하는 px정답·전체 SVG/hash 고정을 구분하는 조사이며, 자료 목록을 의미 판정 완료로 세지 않습니다.
- 조건부Skia 등223정의는 조건을 보존하여 추가 심사합니다.
- 전수 조사 완료 후 지침 부적합 검사를 새 `issue_8232` 명명으로 보정·재검증합니다. 아직 제품·테스트 수정0, PR0이며 이슈는 OPEN입니다.
