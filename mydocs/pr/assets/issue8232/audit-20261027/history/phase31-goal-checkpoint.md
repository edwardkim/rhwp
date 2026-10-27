---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase31-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase31-goal-checkpoint.md` · 원문 SHA-256: `7ab53aa72aafc4553486ebfaf5333ee599a38eac270037c1e46382689725155e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 31차

## 목표와 권한

전수 조사와 독립 판정 → #8232 반영 → 부적합한 기존 검사 및 필요한 공통 구현 보정 → 실제 보정 파일·함수를 `issue_8232`로 명명하고 helper·module·suite manifest·생성 경로·스크립트/E2E 호출 갱신 → 필수 재검증까지 완료한다. 이전 이름의 wrapper나 중복 등록은 만들지 않는다.

잘못된 기존 판정을 유지할 의무는 없다. 독립 근거가 확인된 내용·순서·소속·저장 목적은 유지하거나 재구성한다. 정상 검사까지 일괄 삭제·개명하지 않는다. PR은 나중이며 현재 push/PR 승인은 없다. #8232 수정과 필요한 MCP Print는 승인됐다. agent/task 메시지·예약·메모리 쓰기는 금지돼 있다.

직전 문구 답변 자체는 상태를 변경하지 않은 설명이었다. 이번에는 원장 70함수 추가, 독립 CFB/Unicode 근거, Native 41개 실행, 이슈 갱신으로 실제 진전이 있었다. 전체 목표는 ACTIVE이며 완료·중단·차단 상태가 아니다.

## 현재 작업장

- R: `/Users/tsjang/rhwp`
- 브랜치: `audit/8232-px-regressions-20261025`
- HEAD/base: `238de6de41a208648f4ade6b494fe433a81c1fce`
- Git clean, 제품·테스트 수정 0건, 개명 0건.
- 모든 임시 자료는 ignored `output/issue8232-audit/`에 있다.
- jobs 8, threads 8, 캡처 총 8, Cargo 순차 실행을 유지한다. 원본·PDF·공유 target·다른 작업장을 보존한다.
- 이번에 새 PDF·시각 캡처·WASM/browser·MCP Print는 실행하지 않았다. 기존 Native 실행은 terminal exit 0으로 끝났으며 중복 시작하지 않는다.

## 직접 판정과 실제 실행

네 파일 전체를 읽었다.

| 파일 | 줄 수 | 검사 | helper |
| --- | ---: | ---: | ---: |
| tests/cases/issue_3587_block_import.rs | 554 | 13 | 7 |
| tests/cases/issue_3587_template_fill.rs | 350 | 9 | 4 |
| tests/run_plan_contract.rs | 385 | 8 | 8 |
| tests/unicode_deception_contract.rs | 576 | 13 | 8 |

검사 43개와 helper 27개, 합계 70함수를 중복 없이 추가했다. `record_integration_batch46.py`, `integration-semantics-batch-46.json`, `non-numeric-helper-semantics-37.json`에 함수별 직접 단언과 검출 범위를 연결했다.

현재 generated suite 005/013/027과 nextest metadata에서 43개 등록을 확인했다. 수동 materialize 생성기 2개는 기존 ignored 상태로 실행하지 않았고 나머지 41개만 실행했다. `run_phase31_existing_tests.py`와 `phase31-existing-tests-producer.json/log`가 정본이다. 결과는 41 PASS / 0 FAIL, Summary 1.977초, 선택 target 751 skipped다. 준비 빌드는 10.71초, wrapper 경과 시간은 3.00755초다. 실제 바이너리와 필수 원본 8개의 전후 SHA가 같고 TMPDIR는 ignored output 아래에서 실행 후 비어 있다. 새 검사 FAIL→PASS, 전체 회귀, 독립 시각 수용 결과가 아니다.

## 독립 근거와 검출 한계

`inspect_phase31_independent_contracts.py`는 production 모델 없이 원본 CFB를 읽었다. labnote 문단 12의 표 control 3개, field-01의 %clk 자식 CTRL_DATA에 있는 회사명/작성자, Unicode 합성 대상 원문의 anchor 1회를 확인했다. Rust payload 문자열을 독립 해석해 제로폭 3개, RLO/PDF 각 1개, 태그 6개 → Ignore, 키릴 Тotal을 대조했다. 원😀형의 scalar 3개/UTF-16 4유닛, 8CU gap 이후 주소 [8,9,11], 두 8CU control과 본문·문단 종료를 합친 21, 비교 byte 4+4=8, u16 초과 65,536은 작성 형식·주소·예산이다. 실물 페이지 px 핀이 아니다.

1,700HU 서식, 1,000×1,000HU 작성 그림, storage/ordinal/instance identity, 같은 호출의 preview·현재 상태를 과거 배치 golden과 구분했다. cell별 본문·저장 재열기를 독립 Print 수용으로 확대하지 않는다.

일부 계획 검사의 ‘디스크 무변경’은 출력 부재만 단언하고 ‘한 번 저장’도 출력 존재만 확인한다. 입력 전체 byte 불변이나 write 호출 횟수를 입증하지 않는다. 재독 JSON 전체의 문자열 포함은 field 소속을 보장하지 않고 schema 배선은 모든 runtime 인자 실행의 증거가 아니다. 반면 Unicode 정상 corpus는 최종 checked==5 조건이 있어 누락·파싱 실패를 PASS로 숨기지 않는다.

로컬 Rust AST 식별자 근사로 70함수, 43개 root, ignored 2개, 실제 등록과 관련 참조를 연결했다. run_plan의 Drop은 명시적 이름 참조에 따른 정적 도달에서 빠지지만 RAII 자동 호출과 구분하며 죽은 helper로 판정하지 않는다. trait 동적 호출도 정적 이름만으로 실행 완료라고 세지 않는다. 저장소 전체 호출 경로 검증은 아직 미완료다.

## 원장과 원격 이슈

직접 판정은 6,567개/731소스 파일, 원장은 10,872그룹/2,108파일이다. 미분류 3,975개와 메모만 있는 330개를 합쳐 분류 미확정 4,305개가 남는다. 독립 근거 pending 508개는 이미 직접 읽은 기록 안에 있어 미분류와 단순 합산하지 않는다. 전체 직접 기록의 현재 source SHA와 원장 연결 누락 0을 `phase31-semantic-ledger-validation.json`에서 확인했다. 제품·검사 수정과 개명은 아직 0건이다.

#8232 본문에 이번 결과 4개 bullet을 추가했고, 말미 개행을 포함한 byte 동일성을 API 재조회로 확인했다. updated_at과 본문 SHA는 `phase31-remote-verification.json`에 있다. 기존 본문과 미완료 checkbox를 보존했고 close·완료 표시·PR 생성은 하지 않았다.

## 다음 작업

phase30에서 확인한 잘못된 vpos 4개(같은 원인의 함수 1개), 암호 HWP3의 Native 3쪽 raw 39.00717%, 배경 차이와 전 24쪽/WASM 자격 미완료는 계속 남는다. `phase30-goal-checkpoint.md`와 독립 JSON을 따른다. 이전 검증을 새 source 결과로 재인용하지 않는다.

남은 전수 조사에는 source cfg 검사, Studio E2E, 동적/생성/공용 helper, 분류 미확정 4,305개와 독립 근거 pending 508개가 포함된다. 현재 원장에서 render-backend.test.ts, ai-write-tools.test.ts, overwrite-mode.runner.mjs, nested_tables.rs, issue_4764_pdf_raster_fidelity.rs 등의 실제 미조사 함수를 먼저 확인한다. 끝난 이번 41개와 이전 37·60개를 자동 재실행하지 않는다. 전수 조사 이후 실제 보정·개명·등록 갱신과 필수 검증까지 끝나야 전체 목표가 완료된다.
