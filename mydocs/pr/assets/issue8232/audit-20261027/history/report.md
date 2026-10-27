---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/report.md` · 원문 SHA-256: `64d73f14ff5ecac489da6243b968e684cca9569cc8bd43bf8b240b93532a9475`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 조사 기록 — 정적 전수 목록과 현재 증거 대조

사용자 요청에 따라 **조사와 이슈 업데이트만 수행**했습니다. 제품·회귀 검사·fixture·baseline을 수정하지 않았고 PR·push를 하지 않았습니다. 이후 개선은 새 PR에서 처리합니다.

### 조사 기준

- 브랜치 `audit/8232-px-regressions-20261025`, base/head `238de6de41a208648f4ade6b494fe433a81c1fce`. 착수 시 upstream/devel fetch 후 동일 SHA를 확인했습니다.
- [현행 98%·±2 CSS px 기준](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/manual/pr_review/visual_fixture_evidence.md#L305)을 적용했습니다. 양쪽 raw98%와 확대 판독은 별도 요건입니다. 원본HU·font em·소속·정렬·겹침 관계를 단순 숫자 때문에 삭제 대상으로 잡지 않습니다.
- 이 기록은 **전체 후보의 최종 적합성 심사가 끝났다는 뜻이 아닙니다.** 정적 목록은 전 범위를 훑었지만, 아래 증거 미연결·동적 경로·의미 판정 잔여는 OPEN입니다. 위반 총수로 추측하지 않습니다.

### 범위와 실제 수집

| 구분 | 조사 결과 | 의미 |
| --- | ---: | --- |
| 추적 코드 | 6,253파일 | Rust/JS/TS/TSX/Python. vendor848파일 별도 집계 |
| 제품/도구 source `.rs` | 1,595파일 / `#[test]`4,248개 | root/crate 외 도구·앱도 포함한 정적 구문 수 |
| root `src/` + `crates/*/src` | `#[test]`4,072개 | 외부 모듈·복합cfg를 AST에서 직접 확인한 수 |
| `/tests/` 경로의 Rust | 1,761파일 / `#[test]`7,679개 | root·crate·하위 helper 포함, 실행 건수와 다름 |
| 테스트 스크립트 | 919파일 | Studio/Node/E2E·테스트 지원 경로 정적 검사 |
| `/tests/` 전체 파일 경로 | 3,966개 | fixture·비코드 자료를 포함한 경로 원장 |
| root suite | 28개 / 원본case1,703파일 + 예외target20개 | manifest 검증 오류0, 총 원본경로1,723개 모두 AST 목록에 존재 |
| 물리 경로 추가 확인 | 생성suite28개 | 물리 `src/`·`tests/`·`crates/` 아래에서 추적 목록 밖의 `.rs`는 이28개뿐. 원본case에 전부 연결, 직접 테스트 정의0 |
| 실제 Native 등록 | **11,639건 / 72개 바이너리 / ignored45건** | `cargo nextest list --workspace --cargo-profile release-test` exit0. **테스트 실행·PASS 수가 아님** |
| workspace | 10packages / Cargo target87개 | Cargo metadata와 대조 |
| 테스트 생성macro | 8개 | partition/corpus 검사는 전개된 실제 Native 목록과 원본 템플릿을 구분 |

`CARGO_TARGET_DIR=target/pr-review`, jobs8로 실제 등록 목록을 취득했습니다. 새 full-nextest·WASM·Visual Sweep 실행은 하지 않았습니다. source/fixture가 바뀌지 않은 기존 증적의 동일성을 확인하여 조사에 사용했습니다.

**source unit registry의4,100개와 AST4,072개의 차이:** 외부 모듈3개에서 중복 집계51개, registry 별도 항목으로 잡히지 않는5파일의23개로 `4100 - 51 + 23 = 4072`입니다. 테스트 누락 판정과 같지 않습니다. 이23개 중17개는 실제 Native 목록에 있으며, 나머지6개는 `native-skia` 조건부라 기본 feature 목록에서 제외됐습니다. 후속 적합성 심사에서 해당 feature/target 조건을 별도 추적해야 합니다. 외부 별도 테스트 모듈과 `cfg(all(test,...))`를 이번 조사에서 제외하지 않았습니다.

### 정적 후보 원장

숫자와 기하 assertion/helper를 보수적으로 추출한 결과 **2,707함수·helper 그룹 / 1,071파일 / 4,756구문 지점**입니다. source내부495그룹, `/tests`1,909그룹, 테스트스크립트291그룹, 기타스크립트12그룹입니다. helper중복·합성계약·운영코드 오탐이 포함되어 **부적합 검사2,707개라는 뜻이 아닙니다.**

- 동일 파일의 상수와 helper 호출을 추적해 입력 후보를 연결했습니다. 단순히 파일에 등장하는 모든 원본을 모든 검사에 배정하지 않았습니다.
- 실물 입력 경로와 절대값으로 보이는 literal이 함께 있는 **88그룹을 우선 심사 원장**에 기록했습니다. 별도 상수/상한값으로 숨어 있는 아래 확인2건은 이88개 밖에서 수동으로 발견했습니다. 따라서88개가 모든 절대핀의 확정 총수도 아닙니다.
- 기하 단어가 없는 `upper/lower` 같은 별칭의 누락을 점검하기 위해0.3수치도 별도 역검색했습니다. **30구문/14파일**을 확인했으며 메시지의0.3, 비율0.3%, font em 및 운영코드 IIFE 오탐을 구분했습니다. `issue_7902...`의 위아래 여백 차이≤0.3은 절대 페이지 위치가 아니라 가운데 정렬 관계입니다.

### 현재 증거로 확인한 98% 미만 절대 고정 — 2개 테스트

| 검사 | 실제 고정 | 동일 입력·해당 쪽 raw Native / fresh WASM | 판정 |
| --- | --- | --- | --- |
| [#6925 빈 문단 슬롯 검사](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/tests/cases/issue_6925_empty_cell_paragraph_keeps_stored_slot.rs#L55) | 1쪽3개 문장 y=`391.3/535.2/580.3`, 대칭±6px 및 아래방향6px미만 | **95.74283% / 95.76159%** | 절대 PDF 좌표 고정 자격 미충족. 빈 슬롯/뒤 문단의 원본 줄 점유·소속 관계 보호를 유지하면서 새 PR에서 교정 필요 |
| [#1440 6쪽 박스 줄 원점 검사](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/tests/issue_1440_onsamiro_picture_wrap.rs#L288) | `first_x <235`, `second_x <222` 절대 x상한 | HWP6쪽 **97.38819% / 97.38799%** | 절대 쪽 좌표 상한도 해당. 원본 column_start 이중 적용 검출 계약을 보존해 교정 필요. 같은 함수의 HWPX 입력은 현재 동일 조건 증거 미연결 |

두 함수 모두 실제 Native suite에 등록되어 있습니다. 각각 `regression_suite_017`, `regression_suite_007`에서 확인했습니다. 이번에 테스트를 실행하거나 고치지 않았습니다. 98% 자격 미충족은 원래 빈 슬롯/그림 어울림 기능이 쓸모없다는 판정이 아닙니다.

원 해결 경로도 확인했습니다.

- **#6925 → 원 PR[#7255](https://github.com/edwardkim/rhwp/pull/7255)** head `3380ada6d92ad2bd818d9427ddcaf793fa54cc0e` → 메인터너 입력중복 제거·기존 PDF재사용 → 통합 PR[#7264](https://github.com/edwardkim/rhwp/pull/7264), commit `62f1048f469f767a1c9f200209d23cf48b90016a`. [당시 보정·독립 PDF 기록](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/archives/pr_7255_review.md#L18)에는 공통 원점 약4px 잔여도 명시되어 있습니다. 현재 좌표 자격과 원래 기능 개선을 구분합니다.
- **#1440 → 원 PR[#1441](https://github.com/edwardkim/rhwp/pull/1441)**. 해당 절대 상한의 도입commit `7f026325d279b8a306412cb9beed8b440b753044`, 실제 merge `fc8cf550952792654de7b8fd88fe1c32233b61c2`. 원래 목적은 저장 LineSeg column_start의 이중 적용 방지입니다.

동일 입력·PDF 해시:

| 대상 | 입력 SHA-256 | Print PDF SHA-256 |
| --- | --- | --- |
| `samples/issue6924/148751598-briefing.hwp` / `pdf/148751598-briefing-2020.pdf` | `03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610` | `98ce52ec0a6ed25ba73070b22c743cb72d129113456ae3019f417d1b53ee9dd3` |
| `samples/[2027] 온새미로 1 본교재.hwp` / `pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf` | `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce` | `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7` |

TSV: [6925 Native](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/assets/integration-planet6897/final-validation-20261025/head/native/briefing1487.tsv) / [6925 WASM](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/assets/integration-planet6897/final-validation-20261025/head/wasm/briefing1487.tsv) / [1440 Native](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/assets/integration-planet6897/final-validation-20261025/head/native/onsaemiro-control.tsv) / [1440 WASM](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/assets/integration-planet6897/final-validation-20261025/head/wasm/onsaemiro-control.tsv).

### 오탐을 제거한 대조와 별도 증거 확인 대상

| 대상 | 조사 결론 |
| --- | --- |
| #1440 35쪽 그림-글줄 검사 | 35쪽 raw96.13711/96.13692지만 실제 그림bbox와 글줄의 겹침 관계 검사입니다. 폭150/높이120은 대상 그림 선택값. 미달 페이지라는 이유만으로 관계 검사를 삭제하면 안 됩니다. |
| #1440 6쪽 네 면 점선 연결 검사 | 본문 실제 범위와 괘선의 포함·네 모서리 연결·원본 dash계약입니다. 절대 쪽 원점 고정과 구분합니다. |
| [single_cell_opening_fragment...](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/tests/cases/regulatory_analysis_completed_pages.rs#L525) | 29쪽 y끝1024.3199869±2px는 **29쪽99.30979/99.30615**에 속합니다. 30쪽95.83355/95.83355에는 문장 소속·순서 관계만 적용합니다. 30쪽 점수로29쪽핀을 부적합 처리하지 않습니다. 이는 raw수치 자격 확인이며 전체 직접 판독 요건을 대신하지 않습니다. |
| [#6312 머리 표 높이±0.3](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/tests/cases/issue_6312_float_anchor_line_uses_source_vpos.rs#L98) | 실물1쪽 PDF높이46.1907px±0.3와 원본3476HU/75±0.1이 같은 함수에 섞여 있습니다. 현재 동일 입력의 양쪽raw연결이 없으므로 **미검증**. 후자는 독립 원본 치수 계약으로 따로 심사합니다. |
| [#7150 원점142.37/177.73 등](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/tests/cases/issue_7150_tac_line_owner_anchor.rs#L79) | 원본 줄·여백에서 유도했다는 설명과 Print실측이 함께 있습니다. 단지0.3이란 이유로 부적합이라 단정하지 않고, 원본독립계약인지 잠정출력literal인지와 현재 해당 쪽raw를 추가 확인해야 합니다. |
| #4068/#7063/#7336 | 원본 host·outer margin·HU환산의 관계입니다. 고정 PDF화면좌표로 일괄 취급하지 않습니다. |
| #6708/#7518/#7902 및 수식첨자 대조 | font em·앞뒤 개체 상대 이동·위아래 점유 균형은 절대 쪽좌표와 구분합니다. #7518 상대간격45.33의 독립 출처는 잔여 확인 대상으로 남겼습니다. |

### 증거 재사용과 미완료 범위

현재 제출 증거 **35조건, Native2,041쪽/fresh WASM2,041쪽**에서35개 입력/PDF쌍의SHA를 실제 파일과 대조했습니다. producer `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`와 조사head의 `src crates rhwp-studio` diff가0임을 확인했습니다. [현재 증거 provenance](https://github.com/edwardkim/rhwp/blob/238de6de41a208648f4ade6b494fe433a81c1fce/mydocs/pr/assets/integration-planet6897/final-validation-20261025/EVIDENCE.md)를 사용했으며 과거 전체최저값을 각 검사 쪽수의 값으로 대신하지 않았습니다.

2,707그룹의 자동 입력·페이지 연결은 다음 상태입니다. 이것은 적합/부적합 수가 아니라 **증거 검색 상태**입니다.

- 현재 원본쌍 미연결2,596그룹: 합성/unit/helper/동적경로/원본자료 부족이 섞여 있습니다. 전부 실물 위반·전부 증거누락으로 판정하지 않았습니다.
- 원본쌍 일치하지만 해당 페이지 미해석78그룹.
- 페이지별 쌍 증거 연결28그룹.
- 98%미만 쪽을 포함하여 수동 심사한5그룹: 위 확정2개와 관계검사·페이지소유 오탐3개로 분리했습니다.

**남은 조사:** 우선literal88그룹을 포함한 나머지 후보의 독립 기대값·현재 입력/글꼴/쪽/양쪽raw·확대 판독 연결, 타파일helper·동적fixture·feature/target 조건별 추가 연결입니다. AST는 Rust/JS/TS완전한 심볼·매크로전개 해석기가 아닙니다. 기본 feature Native목록만으로 조건부Skia/Windows/WASM경로까지 실제 실행했다고 주장하지 않습니다. 생산 TypeScript `rhwp-studio/src/core/wasm-bridge.ts`1파일은 파서오류가 있어 별도 문자열검색(직접assert/expect0)을 했고, 테스트코드 AST파서오류는0입니다. shell/PowerShell38파일(테스트·harness표식28)도 별도 원장에 남겼으며, `svg_regression_diff.sh`는 before/after SVG byte차이 보고도구로 독립PDF98%자격을 대신하지 않습니다.

전체 의미 판정·시각 증거 연결·수정·전후회귀·CI/통합은 **미완료**입니다. 이 이슈의 완료 체크박스를 완료로 바꾸거나 이슈를 닫지 않습니다.

### 로컬 조사 산출물

`/Users/tsjang/rhwp/output/issue8232-audit/`에만 보관했습니다. 저장소 changes에 임시자료를 섞지 않았습니다.

- `files.json`, `functions.json`, `assertions.json`, `indirect.json`, `macros.json`: 전체 구문/간접 참조 원장
- `candidate-functions.tsv/json`:2,707그룹 전체 후보
- `literal-pin-priority.tsv/json`:88그룹 우선 조사 목록
- `tolerance-0p3-reviewed.json`:30구문 수동 분류와 사유
- `canonical-suite-inventory.json`, `canonical-suite-check.json`, `generated-suite-reconciliation.json`:정본/생성suite 대조
- `conditional-unit-reconciliation.json`, `runtime-selected-tests.json`:조건부 및 확인된 검사의 실제 등록
- `nextest-list.json`, `nextest-list.log`:Native 실제 등록 원문/로그(exit0, 컴파일8분40초)
- `current-evidence-index.json`:현재35조건의 해시와 전쪽 raw

PR·개선 작업은 사용자 후속 지시 시 별도 단계로 진행합니다.
