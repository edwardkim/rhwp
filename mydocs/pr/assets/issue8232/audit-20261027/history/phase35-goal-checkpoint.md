---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase35-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase35-goal-checkpoint.md` · 원문 SHA-256: `2304307cdb15a91a303f28468d404e76a6e7f88764b02b5741774d415dbb8c1b`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 35차

사용자 목표는 전수 조사·독립 판정 → 이슈 반영 → 현행 지침에 맞지 않는 검사와 필요한 구현 보정 → 실제 보정 파일·함수를 issue_8232로 명명하고 호출·module·suite manifest·생성기·스크립트/E2E 경로 갱신 → 실제 검증이다. 잘못된 기존 판정은 보존하지 않고 독립 근거가 있는 검증 목적을 유지/재구성한다. 이전 이름 wrapper·중복 등록을 만들지 않는다. 정상 검사를 이름만 바꾸려고 수정하지 않는다. PR은 나중이며 현재 push/PR 승인은 없다. 이슈 본문 수정과 필요한 독립 Print는 승인됐다. agent·task 메시지·예약·메모리 쓰기는 없다.

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. 작업장 clean이며 제품/검사 변경·개명은 0건이다. 새 자료는 ignored output/issue8232-audit 안에만 작성했다. 공유 target/pr-review·원본·PDF·다른 작업장을 보존했다. Cargo 순차/jobs8/threads8/캡처 총8을 유지했다. 이번 실제 검사와 캡처는 모두 terminal이다. 전체 목표는 ACTIVE이며 완료/차단/중지가 아니다.

## 직접 조사와 실제 실행

tests/cases/stored_partial_frame_alignment.rs 전체529줄과 tests/cases/issue_4962_font_metric_coverage.rs 전체를 읽었다. 두 파일의38함수 중 신규33개/기존5개 재판정이다. 기존15개 Native를 현재 generated suite006/017에서 도출해 실행했고 15PASS/0FAIL, Summary4.200초, 선택 target의 나머지502개 skipped다. wrapper5.056365초이며 원본·fixture·바이너리 전후 SHA가 같다. phase35-existing-tests-producer.json/log를 따른다. 이 결과는 전체 회귀나 새 정식 검사 FAIL→PASS가 아니다.

실물 Center 정렬의 원본 앞 띠·빈 줄 상자·실제 상하 여유 관계와 절대 page 원점 pin을 구분했다. 0.05×6 또는0.05×4는 관계 계산의 JSON 반올림 bound이며 기계적으로±2로 바꿀 수치가 아니다. 별도 CFB 직접 읽기에서 Center14셀, 557번6셀의 저장 leading=spacing_before/2, 650번 닫는 빈 문단14의600HU 높이/360HU 뒤 간격을 확인했다. 이는 현재 구현의 계산값을 정답으로 복사한 것이 아니다. section1 중첩118/119쪽의 전체 raw ancestry·독립 소유 대조는 별도 pending이다. 글꼴 집계의 count/schema·현재 호출 결정성·자원 예산·Unicode scalar와 source em은 실제 문서 page px 고정과 다르다.

## 네 쪽 실제 Native/fresh WASM 및 독립 PDF 좌표

같은 가맹사업 원본 SHA a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55와 독립 Print SHA4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009를 사용했다. 기본 환경57/71쪽과 기존 명시 Human/Haansoft 환경68/70쪽을 분리했다. 정본 sweep/96CSSdpi/2px raw/Native Skia/fresh WASM PrintCanvas이며 hash 고정된 감사 producer를 재사용했다. Mac WASM은 locked wrapper web/pkg/no-opt 로컬 대체 빌드로 새 Docker 최적화 통과가 아니다.

| 쪽 | 새 Native raw | 새 fresh WASM raw | 독립 PDF 첫 대상 기준선 CSSpx |
| --- | ---: | ---: | ---: |
| 57 | 100.00000 | 100.00000 | 840.16 |
| 68 | 100.00000 | 100.00000 | 796.96 |
| 70 | 100.00000 | 100.00000 | 155.84 |
| 71 | 99.93799 | 99.93797 | 527.52 |

phase35-franchise-pages/state.json의 네 실행은 capture/quant 모두 exit0이며 전체66.634089초다. 각 요청 페이지 집합과 TSV가 정확히 같다. 원본/PDF는 바꾸지 않았다. 두 backend의 review·독립 overlay16장을 직접 읽고 각 기준선8개를 nearest3배 확대해 다시 읽었다. 자형·획 농도·밑줄과 점선의 미세 차이는 남으며 raw100을 RGBA 동일성으로 해석하지 않는다. 실제 Native/WASM RGBA도 각 쪽에서 다르다. 확대 사본의 source SHA·crop/배율은 baseline-crops/provenance.json에 있다.

PDF CTM과 Tm을 합성해 source 단어가 있는 첫 줄을 선택했다. 기존 기대 y를 선택창으로 사용하지 않는다. PDF의 네 기준선과 기존 기대값의 최대 차이는0.000039CSSpx다. 초기 탐색에서 CTM을 적용하지 않은 임시 출력은 잘못된 좌표였고 최종 근거로 쓰지 않는다. phase35-independent-pdf-baselines.json과 직접 판독을 따른다. 기본/명시 환경은 검사 caller와 같이 유지했다. 따라서 네 쪽의±2CSSpx 검사는 이번 조사에서 유지 근거가 있으며 baseline helper 두 개의 pending을 해당 caller 범위에서 해소했다.

기존 전142쪽 TSV는 별도로 source673c1d5f와 현재 src/crates/rhwp-studio diff0, 정확한 입력/PDF hash와 커밋된 원본 TSV142행을 대조했다. 최저 Native93.81069/freshWASM93.80829%다. 이번 새 실행은 네 쪽이며 전142쪽을 새로 실행했다고 쓰지 않는다. 과거 README의 최저93.16816%는 이전 producer의 이력이다. 실제 #8191·후속#8228 merge가 현재 조상임도 확인했다.

## 원장·이슈와 다음 작업

integration-semantics-batch-48.json/non-numeric-helper-semantics-41.json의38기록은 신규33/갱신5로 분리했다. 직접 판정 누계7,114개/756소스 파일, 전체 원장11,352그룹/2,127파일이다. 미분류3,908+메모만330=4,238개, 독립 근거 pending507개는 직접 기록 안에 포함된다. 직접 판정·좁은 PASS를 전체 적합/호출 폐쇄로 확대하지 않는다. 새로운 결함을 이번에 추가 확정한 것은0개다. 모든 기존 미해결 문제는 계속 유효하다.

validate_phase35_ledger.py가 현재 source SHA·원장 연결·15PASS·두 환경 네 쪽/두 backend·독립 PDF 좌표·raw CFB·16장 판독·원본과 바이너리 보존·실제 merge ancestry를 검증했다. 이슈 #8232에 사용자 표현 정정과 이번 결과를 반영하고, 앞의 현재 집계도 최신 원장으로 정정했다. API 재조회 본문이 말미 개행까지 byte 동일했다. phase35-remote-verification.json의 updated_at=2026-10-26T01:45:52Z, bodySHA16bb354b14fa34b2d39a7148d78547282c3ae505aa9aa600f7b642838f750c12다. 이슈 OPEN/완료 checkbox 미표시/PR 생성 없음이다.

다음에는 아직 미분류인 실제 문서 검사와 공용·중첩 helper/동적·생성/E2E 경로를 계속 직접 판정한다. 현재 미분류 후보 issue_5961_hwpx_lineseg_axis_projection, issue_6624_chart_default_layout, issue_6778_square_beside_lane_render_flow, issue_6981_split_straddle_row_height 등과 중첩 source ancestry pending을 원장으로 선택한다. 이번15개 및 끝난 네 쪽 캡처를 자동 중복 실행하지 않는다. 전수 조사·실제 부적합 보정·새 명명·모든 소비/등록 경로 갱신과 필수 검증은 아직 남는다.
