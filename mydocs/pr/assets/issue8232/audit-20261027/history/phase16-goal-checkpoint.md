---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase16-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase16-goal-checkpoint.md` · 원문 SHA-256: `dd0f2c5e0258a28dc4de1c4e9b875488fde43b972196d57e7a97b5f13bd1cd85`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 16차 체크포인트

목표는 ACTIVE이다. 직전 질문 응답은 설명만 한 턴이었다. 이번에는 실제 작업장/프로세스를 재확인하고, 직접 분류·독립 원본 확인·검사 미검출 반례·원장 연결 보정·이슈 갱신을 수행했으므로 progress이다. 완료/차단/일시 정지로 바꾸지 않는다.

## 범위와 상태

- R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025.
- HEAD238de6de41a208648f4ade6b494fe433a81c1fce, tracked clean. 제품·테스트 수정/개명은 여전히0건이다.
- 전수 조사→이슈 정리→실제 보정/issue_8232 새 명명/호출·등록 갱신/재검증까지가 전체 목표다. 감사-only로 축소하지 않는다.
- 독립 근거로 유효한 결함 검출을 보호한다. 잘못된 기존 기대값·선택 대상·단언까지 보존하지 않는다. 이름 보존 wrapper/중복 등록은 만들지 않는다.
- PR/push는 나중이며 현재 승인 없음. 이슈8232 본문/댓글 갱신과 필요한 MCP는 승인돼 있다. subagent/다른 task 메시지·예약·메모리 쓰기는 하지 않는다.
- 빌드jobs8/tests8/capture총8/Cargo순차·원본/PDF/공유target 보존.

## 직접 분류

record_integration_batch25~29.py → integration-semantics-batch-25~29.json 및 non-numeric-helper-semantics-16~20.json.

- batch25: PrEP646줄, Native commands706줄, Native editor655줄. test57/helper35.
- batch26: hyperlink6963의798줄, mixed char format6788의508줄. test37/helper17.
- batch27: rowbreak_chart_overlap633줄, clickhere_form_mode745줄. test29/helper17.
- batch28: Native region cache486줄, Native view399줄. test25/helper8.
- batch29: TAC5788의78줄, ghost vrules5877의191줄, source_frame_relations5877의239줄, nested_host5885의239줄, continuation_body5885의90줄. test10/helper24.
- 합계14파일/test158/helper101=259함수. 전체 파일의 functions.json 목록과 판정 레코드 집합·source SHA·원장 연결을 대조해 누락0임을 확인했다. phase16-audit-record-validation.json. 실제 정식 test 실행 PASS나 전체 production 호출 폐쇄 완료가 아니다.
- 본문 범위 helper all_text는 본문·표·Shape text_box만 순회한다. 각주/머리말/제어 자체 전체 보존으로 확대하지 않는다.
- Native view의1e-9는 작성한 호스트 산술, region cache는 같은 호출의 전체 raster/crop 동등성이다. 과거 golden/PDF 시각 정답으로 취급하지 않는다. region cache는 native-skia 조건부다.

## 독립 원본 단위

inspect_batch27_source_units.py → batch27-source-units.json.
rowbreak HWP/HWPX 독립 PDF는 각각18쪽이다. 원본 HWPX section1/para13/cell0의 직접 p/linesegarray에서 caption1000HU, textbox host11564HU, LH1100HU를 읽었다. 비율9.6036363636, 허용2/1100은 원본HU 양자화 관계다. 첫 진단은 하위 글상자 lineseg까지 섞었으므로 직접 host array로 고쳤다. 현재 paired catalog에는 두 원본의 전쪽 시각 자격이 없으며 과거 stage27 raw를 현재 head 수용으로 인용하지 않는다. 고정18쪽 검사319는 pending이다.
batch29-independent-pdf-counts.json은 TAC두판본8/8쪽, gas31쪽, design7쪽과 SHA를 기록한다. 원 이슈의 거대 원본9쪽과 축소 fixture8쪽을 혼동하지 않는다.

## 새 확인: #5788 고정 영역·다음 글줄 선택·본문 누락 미검출

tests/cases/issue_5788_tac_table_anchor_line_spacing.rs:20.
기존 y380..500/x폭300 선택 뒤 raw y attribute만 읽는다. 현재 SVG에서 표 하단464.826667, 실제 다음 (주) translate baseline484.893333이지만 장평 transform 글자를 건너뛰고 뒤 표 본문610.346667을 선택해145.52px로PASS한다. 모든 text를 제거한 반례도 next_baseline=f64::MAX로PASS한다.

- observe_tac_anchor_gap_counterexample.py와 tac-anchor-gap-counterexample/observation.json.
- 현재 frozenCLI export-SVG index0, Screen 기본/compat2022. 첫 -p1은0기반 때문에 실제 두 번째 쪽이었다. invalid-page-index/INVALID.txt에 보존했으며 -p0의 최종 입력만 사용한다. 잘못된 실행을 성공 근거로 쓰지 않는다.
- export exec17930 terminal exit0, 앞선 잘못된 index exec32918도terminal0. 모두 종료했다. 새 캡처/서버 없음.
- frozen CLI SHA5c9aed5b7779a9fffc102ebc662ad1f66faabc020012011dc1444a12c8222c9f. SVG/input/test SHA는 observation에 있다.
- 현재 SVG에 predicate를 재현한 반례이며 제품 mutation/과거binary/새 정식 test FAIL→PASS가 아니다. 전쪽 Native/fresh WASM/PDF 수용도 아니다.
- 원 이슈5788 전체본문/댓글, 원PR5829 body/live metadata 및 archive review, 실제통합5844/1b8c39b6(current ancestor)를 확인했다. 원PR5829는CLOSED 미병합이며 실제누적통합은5844다. 도입/현재 검사SHA2472b97f95b6b3c8fdb7f67e065db0af9d8e336a9e368e15bdf7a19aea6448bb가 동일하다. origin-metadata.json.
- 실제 보정 시 독립 전쪽 자격과 원본 소유/다음 문단·간격을 보호하고 단순 하한/허용폭 조정으로 대신하지 않는다.

## 원장 builder 누락 보정

직접 판정 기록을 초기 numeric/render/helper 후보와 연결하는 검사에서78개가 원장에 없었다. MCP wrong response finding도 빠져 있었다. build_audit_ledger.py가 직접 판정한 함수 전부를 records에 연결하도록 보완했다. 이는 ignored 감사 도구 변경이며 제품/테스트 수정이 아니다.

- 보정 전 목록/수치: phase16-ledger-backstop-before.json.
- 전체 직접 판정5197개가 현재 원장에 모두 연결됐고 누락0이다. 새로운 결함/PASS78개라는 뜻이 아니다.
- 최신원장10194그룹/2103파일. unreviewed4655; direct4755; evidence_pending442; semantic_notes130; file_context212.
- full_conformance false/call_closure pending 유지. 위 숫자를 적합/완료 수로 보고하지 않는다.
- 초기 validation의 set(keys)<=ledger assertion이 실제 누락을 검출했다. builder 보정 후 같은 연결검사를 통과했다. 산출물검사이지 제품회귀PASS가 아니다.

## 게시와 다음 작업

이슈8232 본문과 댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6403331817 갱신·원격/로컬 본문 정확히 일치 검증 완료.
phase16-final-issue-body.md/phase16-survey-report.md/phase16-published-receipt.json이 최종이다. 원장78개 연결 전 receipt는 before-ledger-backstop 파일로 보존한다.

4655미분류 및 독립 pending/동적 helper·fixture 소비를 계속 조사한다. 이전 phase15-checkpoint의 PUA잘못된셀19/실제22, MCP wrong response, Hyperlink드롭, 상대크기, Recipe, page98자격 보정 등은 아직 source미수정이며 잊지 않는다. source/지침 변동 없이 이번 읽은14파일을 다시 처음부터 읽지 않는다. 필요한 전체 감사와 실제 보정/개명/등록/필수검증을 끝내야 완료다.

관련 전역memory검색 hits0이라 memory기반 판단을 쓰지 않았다. 마지막 프로세스 확인은 도구 자신의 ps/rg만 있었고 Cargo/캡처/observer는 없었다. 다음 긴 실행 전에는 다시 실제프로세스를 확인한다.
