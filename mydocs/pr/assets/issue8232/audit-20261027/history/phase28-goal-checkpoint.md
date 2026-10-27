---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase28-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase28-goal-checkpoint.md` · 원문 SHA-256: `69cb5a6af2c8123bdadec73106c439fe84ac14d5adb96de09a175da6c2aa8948`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 28차

이전 권한·전체 목표·정본 경로는 phase27-goal-checkpoint.md와 같다. 전수 조사/독립 판정 뒤 이슈 정리 → 실제 부적합 검사/필요한 구현 보정 → 실제 보정 파일·함수를 issue_8232로 새 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 소비 경로 갱신 → 필수 검증 순서다. 기존의 잘못된 판정을 보존할 의무는 두지 않는다. 독립 근거가 있는 내용·순서·소속·저장 목적은 적절한 검사로 유지/대체한다. 이전 이름 wrapper/중복 등록 금지. PR 나중/push·PR 승인 없음. #8232 본문 업데이트·필요한 MCP Print 승인. subagent/task 메시지/예약/메모리 쓰기 금지.

## 현재 상태

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. Git clean. 제품/테스트 수정0·개명0. 모든 이번 도구·증거는 ignored output/issue8232-audit 아래다. jobs8/threads8/capture합계8/Cargo순차를 유지하고 shared target/pr-review·원본·PDF·다른 작업장을 보존했다. 실제 Cargo/nextest/rustc/Python driver는 모두 종료했다. goal ACTIVE.

현재 사양은 PDFium5.13/96CSSdpi/2px raw, 실물 절대 좌표에는 해당쪽 양backend98%/독립PDF 좌표/확대판독/±2CSS/내용관계가 필요하다. source bytes/CU/HU·작성치수/상대단위·가역/비영향 snapshot과 구분한다. 현재 AGENTS/CLAUDE/visual_fixture_evidence 의미 계약의 내용·90% 선행 조건·원PR→실제통합·독립기대의 추적은 유지한다. 전체 조사 전 실제 보정/개명을 시작하지 않았다.

## 새 직접 조사 82함수

다음 네 파일을 전체 읽고 기존 functions/assertions/source SHA에 연결했다.

| 파일 | 전체 줄 | 검사 | helper |
| --- | --- | --- | --- |
| tests/cases/bookmark_delete_preservation.rs | 420 | 9 | 7 |
| tests/cases/issue_536_native_clipboard.rs | 553 | 19 | 6 |
| tests/cases/issue_6874_hwp3_objtype3_is_a_table.rs | 402 | 7 | 18 |
| tests/issue_1251_ole_chart_contents.rs | 387 | 10 | 6 |

integration-semantics-batch-43.json / non-numeric-helper-semantics-34.json / record_integration_batch43.py. 기존 직접 판정 중복0, 새 고유82개다. 직접6334records/718sourcefiles가 모두 현재source SHA·원장에 연결됐다.

책갈피8CU/임계27·37/ADD_AT4·14scalar 주소 배열은 페이지px가 아니다. 독립 산술은 secd/cold16 + emoji2 + tab8 + 명시 field gap8 at9/11 + shape gap8 at12를 적용해 [16,18,26,27,28,29,30,31,32,41,42,51,60,61]을 얻었다. 현재 before/after Debug/JSON 동등은 가역/비영향이며 historical SVG golden이 아니다. fields helper는 startPos/endPos를 제거하므로 raw 주소 전체 검증으로 과장하지 않는다. 별도 char_offsets/control/range/markpen 단언의 범위를 함께 기록했다.

clipboard는 실제 NativeEditor/Session의 scalar selection/revision/history/DTO/byte budget/owned capacity 계약이다. 43pt 빈 HTML·작성 표/도형/개체는 입력 제작이다. 5가지 entrypoint의 지원밖 객체·6HTML object 입력·4 runtime command 등 실제 호출 범위를 읽었다. 실제 OS clipboard·WASM/browser 재생·실물 glyph 시각 완료라고 주장하지 않는다.

HWP3 source hunit=1/1800인치. 100hunit 이동=100×96/1800=16/3CSSpx이고, 일반 ParaShape는 ×8 저장 후 flow에서2로나눠 ×4를 쓰는 계약을 소스에서 확인했다. 이 관계의1e-6은 절대원점 핀이 아니다. source250hunit=10pt의160%leading은600HU, 2000×4=8000HU/400×4=1600HU와baseline300/400=75%를 독립 산술로 대조했다. source Johab bytes는 Python johab codec으로 신명조/바탕으로 읽히며 표시이름/원래HFT종류를 구분한다. 그러나 equation기본1200HU/HYhwpEQ는 원본 상자 수치에서 유도되지 않으므로 root369는 independent_equation_default_font_policy_pending으로 남겼다. source unit 정상 단언까지 blanket삭제/±2완화하지 않는다.

## 독립 입력 확인

inspect_phase28_source_contracts.py / phase28-independent-source-contracts.json/log는 product parser 없이 OLE CFB, XML, Unicode/단위 산술을 읽었다.

- HWP samples/143E433F503322BD33.hwp와 HWPX samples/hwpx/143E433F503322BD33.hwpx는 각각 size-prefix포함11780B storage/Contents단일stream9876B. 전체 storage SHA와Contents SHA는 서로 다르다. HWP header00000100ec2e0000ec2e000060000000, HWPX는000001009e2800009e28000060000000이다. 같은 길이/라벨이라고 byte동일성을 추정하지 않는다.
- CP949 라벨8개(시리즈3/연도4/전각공백title)와 sourcef64 12개는 정확한 raw offset에서 양쪽이같다. 수치offset은326+category×117+series×26. 차트1702등은 data 값이지 page 좌표가 아니다. 다른 bytes의 차이를 metadata/geometry라고 추정하지 않는다.
- HWPX content.hpf의ole3/isEmbeded0/internalhref와extent7200, 원본 HWPgsoattr0x143a2610/OLErecord26B/직전component196B/7200HU·CTRL_DATA24B2개/책갈피16B1개를 대조했다. 형식·선언단위 계약이며 전체 차트style/배치/Print 수용이 아니다.
- tests/fixtures/issue_6874의 seizure-list-hancom-2020.hwpx와 seizure-list-candidate-hancom-2020.hwpx 각각tbl2/btn0/-581-13-를 확인했다. 원 한컴 기준과 후보 한컴 재저장을 구분하고 후자를 독립원본으로 쓰지 않는다. mydocs/pr/archives/pr_7088_review.md 전체를 읽었으며 원PR7088/head186953b.../cherry75099f5ce/보정568b210d9/재저장d3dfa17e0의 역사 근거와 현재Git objects를 구분했다. 해당3commit실제SHA/subject는읽었지만 이번에 원PRAPI/통합전체 ancestry를 재완료한 것은 아니다. 과거97.08982/pdftoppm 결과를 현재canonical 양backend시각 자격으로 인용하지 않는다.
- 기술문서 hwp_ole_chart_renderer_architecture_decision_1251.md의 현재 canonical renderer 결정과 investigations/issue-1251 시각 보고를 읽었다. 후자는 historical이며 차트 최소경로와 fullfidelity를 구분하는 기록이다. 당시 스타일 차이를 현재 실패나 현재 수용으로 재인용하지 않는다.

초기 진단의 오류도 보존했다. 첫 driver는 HWP/HWPX 전체storage동일을 잘못 가정해 중단됐고 inspect_phase28_source_contracts-first-storage-equality.py.txt에 남겼다. 실제 raw두Contents도 서로 달랐으며 지정 라벨/숫자만 독립 비교하도록 바로잡았다. 두번째 driver는 HWPX t.text만 읽어 fwSpace child의tail에있는581-13을 놓쳤다. inspect_phase28_source_contracts-second-text-selection.py.txt / phase28-independent-source-contracts-second-selector-error.log를 보존하고 t.itertext() 전체로 수정했다. 최종 driver 실제exit0와 JSON/log만 최종근거다. 이 둘은 product/test FAIL이 아니고 새로운 정식 FAIL→PASS도 아니다.

## 실제 Native/로컬 helper

run_phase28_existing_tests.py는 현재 generated suite012/023/027에서 위4module45개를 metadata로 고정해 native-skia로 실행했다. phase28-current-tests-list.json의 actual선택이 functions의45test이름과정확히같음을검증했다. 준비12.98초, Summary0.102초:45PASS/0FAIL/753선택target skipped. wrapper1.170초, 3실제바이너리 전후SHA동일. freshWASM/capture/MCP/전체회귀/새lint실행은이번에없다. source수정이없으므로중복전체gate를돌리지않았다.

inspect_phase28_reachability.py / phase28-local-helper-reachability.json는 Rust AST의 body identifier를수집하고함수포인터·동명함수를과대근사로연결한다. 45root→37localhelpers(7/6/18/6), 로컬미도달0. 전체82함수가이번직접판정에있다. generated suite source SHA/참조결과도기록했다. 이과대근사를모든runtime분기/전체repo동적호출폐쇄로승격하지않는다. 다른파일helper/module/corpus의전수조사는여전히필요하다.

## 원장·이슈·다음

audit-ledger-summary:10745groups/2107files, unreviewed4081 / function_px_policy_adjudicated5827 / function_evidence_pending507 / semantic_notes118 / file_context212. 새82개 중81직접수치의미분류·1독립기본값pending이다. 직접6334/718 모두원장/currentSHA연결,누락0. phase28-semantic-ledger-validation.json.full_conformance/전체call closure/전체audit는false이고제품/테스트수정·개명0이다.

#8232 OPEN updated_at2026-10-25T23:56:12Z. phase28-final-issue-body.md와API재조회body는끝개행포함byte동일, SHA5ab81d7aaf13aa3587c5898556fe12a0c6b5c32c239ab56bfccb30adc49bc27e. phase28-remote-verification.json에남겼다. 원장현황·82함수·raw단위·형식차이·45PASS·localhelper37/한계를본문에반영했다. close/PR/완료체크하지않았다. goal ACTIVE.

다음은 실제현재HEAD/dirty/process부터확인한다. 미분류상위에는 tests/injection_scan_contract.rs11, tests/cases/cli_catalog_contract.rs10, agent_form_fill_skill_contract.rs9, agent_recipes_skill_contract.rs9 등이있다. 숫자는원장의기존그룹수이지전체함수수/위반수/PASS수가아니다. scopes/functions/assertions와전체파일을읽고실제call/helper/상수·dynamic경로를연결한다. 이미끝난이번45개/이전39개를자동중복재시작하지않는다. 이전phase27의source값/그림identity pending, phase26bbox origin/size-receipt pending등도여전히남아있다. fullaudit후에실제부적합수정/issue_8232개명/호출·등록갱신/필수검증을완료해야전체목표완료다.
