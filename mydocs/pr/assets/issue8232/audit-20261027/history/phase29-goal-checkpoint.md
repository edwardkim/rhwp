---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase29-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase29-goal-checkpoint.md` · 원문 SHA-256: `4d9ab830fb06db403f11cce315e08b6af0c950cb3ef4b5afc48a8608df100f8b`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 29차

전체 목표·권한·이전 근거는 phase28-goal-checkpoint.md와 그 이전 checkpoint를 따른다. 전수 조사/독립 판정 → 이슈 업데이트 → 실제 부적합 검사/필요한 구현 보정 → 실제 보정 파일·함수를 issue_8232로 새 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 호출 갱신 → 필수 재검증까지가 목표다. 기존의 잘못된 판정은 보존할 의무가 없으며, 독립 근거가 있는 내용·순서·소속·저장 계약을 적절한 검사로 유지/대체한다. 정상 숫자/정상 검사까지 일괄 개명/삭제하거나 옛 wrapper로 중복 등록하지 않는다. PR은 나중이고 push/PR 승인 없음. #8232 본문 업데이트·필요한 MCP Print는 승인됨. subagent/task 메시지/예약/메모리 쓰기 금지.

## 현재 작업장·완료 여부

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. Git clean. 제품/테스트 수정0·개명0. 모든 이번 스크립트와 증거는 ignored output/issue8232-audit 아래다. jobs8/threads8/capture합계8/Cargo순차·공유 target/pr-review 보존. 이번에는 Native metadata/CLI 논리 계약을 확인했으며 캡처/MCP/실제WASM/browser/OSclipboard/원격CI 실행은 없다. 모든 Cargo/Native/observer 프로세스는 종료했다. 목표 ACTIVE, 전체 완료 아님.

## 새 직접 판정 102함수

네 test source는 전체를 읽었다.

| 파일 | 줄 | 검사 | helper |
| --- | --- | --- | --- |
| tests/injection_scan_contract.rs | 828 | 14 | 14 |
| tests/cases/cli_catalog_contract.rs | 691 | 20 | 5 |
| tests/cases/agent_form_fill_skill_contract.rs | 306 | 14 | 9 |
| tests/cases/agent_recipes_skill_contract.rs | 215 | 12 | 7 |

추가로 src/cli/catalog.rs의7함수 body와 COMMANDS 전체의 AST 선언을 읽었다. 검사60/helper42=102, prior overlap0. integration-semantics-batch-44.json / non-numeric-helper-semantics-35.json / record_integration_batch44.py에 source SHA·직접 단언을 연결했다. source catalog는 test의 #[path]로 포함되지만 새 source-side test7개가 아니다. 실제 catalog:: 하위 등록test는0개다.

발췌200scalar, 참조문서16개/UTF8 byte분량, 사례40/80개, 명령·fixture·schema/이슈id/0기반 필드 순번은 실물 페이지px 핀과 구분했다. glyph/layout 출력의 절대 좌표는 이번 파일들에 없다. source 파일/함수명 존재·위임 문자열 guard는 모듈 소유 구조의 guard이지 모든 handler의 실제 동작 증거가 아니다. peer SKILL파일 존재 guard는 그 내용의 비변경을 검증하지 않는다. fixture stale:false는 현재 날짜와의 신선도 계산이 아니다. 이런 범위를 넘겨 적합/실행 완료로 보고하지 않는다. SKILL 문서는 검사 데이터로 읽었으며 이번 작업 지침·승인으로 적용한 것이 아니다.

## 실제 실행·조건부 본문 증거

run_phase29_existing_tests.py는 현재 generated suite005/008/014/024에서 네 모듈60개를 cargo nextest list metadata로 준비·고정했다. 빌드15.67초. 실제 실행은 그 test 바이너리를 libtest module prefix/threads8/--nocapture로 직접 실행했다. nextest run summary로 보고하지 않는다. 각각14/20/14/12 PASS, FAIL0, ignored0, filtered239/238/256/253(합계986), summary6.19/.08/.11/.02초, wrapperelapsed6.41898초. 실제 바이너리4개/realCLI 전후SHA동일. phase29-existing-tests.log / phase29-existing-tests-producer.json / phase29-current-tests-list.json.

CLI 관찰은 phase29-cli-observer.py가 기존 CARGO_BIN_EXE 경로를 받는 test helper의 선택을 통해 개입하며, 실제 metadata의 release-test/rhwp를 argv 그대로 실행하고 stdout/stderr bytes/exit를 그대로 반환한다. realCLI와 observer 각각의 SHA를 기록했다. 입력데이터를 별도 명령으로 실행하지 않았고 외부MCP/네트워크 전송을 한 것도 아니다. TMPDIR를 phase29-cli-observations/tmp로 고정해 기존 std::env::temp_dir 산출도 output 아래로 모았다.

관찰 event108개: injection101/form-fill4/catalog3/recipes0. replace-text 합성24개가 전부 exit0/출력생성됐고, actual output SHA와 후속 scan 입력 SHA가 연결됐다. POSITIVE9개(6kind)와 NEGATIVE10개 모두 실제 생성/스캔/기대kind 또는clean 판정이 있고, nosanitize/envelope/minconf/registry/registry-neg 추가5개도 있다. 원래 검사코드는 ran>0만 요구해 일부 합성 실패를 허용하므로 formal PASS만으로9/10전부 실행을 단정하지 않았다. 이번에는 관찰과 로그에 합성 실패/건너뜀이 없음을 함께 확인했다. 모든 관찰 입력 파일의 before/after SHA가 같았다. 원본 test의 sha helper는 SHA256이 아니라 BLAKE3이며, observer의SHA256과 구분한다.

각 임시 출력이 검사에 의해 삭제되기 전에 동일 bytes를 phase29-cli-observations/synthesized-inputs에24개 보존했고 SHA를 대조했다. 각 event JSON은 events에108개 있다. 신호 문자열은 검사 입력 데이터다. 실제 보안 완전성/모든 corpus의 오탐0을 주장하지 않는다.

new_normal_sample_documents_are_clean은 환경 JSON이 비면 return으로 본문을 건너뛰는 검사다. 이번 명시 RHWP_SECURITY_SWEEP_SAMPLES_JSON=[samples/hwp3-sample.hwp]를 설정해 checked>0 guard를 실제1입력으로 실행했다. 현재PR신규문서 전체/전체samples수용이 아니다. form01/HOST/empty_header_only.csv가 존재하고 실제 form fields2/dry-run/emptyCSV오류4CLI가 관찰됐다. dry-run exit0/dryRuntrue/filledCount1/출력미생성을 확인했다.

## 독립 metadata/CI/공용 입력

inspect_phase29_metadata_contracts.py / phase29-independent-metadata-contracts.json/log:

- Rust AST로 src/cli/catalog.rs COMMANDS의 모든 spec 항목이 string·enum·feature·bool7필드임을 읽고 실제 main 최상위 match pattern만 수집했다. 명령103개가 중복 없이 정확히 같다. source 주석의옛98개는 현재수치가 아니다. 주석/nestedSome을 이름으로 읽지 않은 별도 대조다. CLI 실제 전체handler 실행 증거는 아니다.
- form-fill7개/recipes10개(추가 transcript index 포함), fixture JSON17개와 reference파일의SHA/UTF8bytes·배열개수를 읽었다. metadata 선언/파일 존재를 실제80여정·8recipe 편집 실행으로 확대하지 않는다. Recipe canonical 디렉터리의 동적07_/08_ 부재 검사도 파일명 범위다.
- 원본 samples/form-01.hwp를 독립 CFB record로 읽어 %clk CTRL_HEADER1개와 그 자식 CTRL_DATA26byte의UTF16 myMsg01(해당payload offset12)을 확인했다. 파일에는 다른 form UI control들도 있지만 그들을 click-here query의 동일 의미로 취급하지 않는다.
- 초기 diagnostic은 이름이 CTRL_HEADER 안에 있다고 잘못 가정해 assert에서 중단됐다. inspect_phase29_metadata_contracts-first-field-header-selector.py.txt / phase29-independent-metadata-contracts-first-selector-error.log를 보존했다. 실제source는level2 CTRL_DATA에이름을담으므로level계층으로%clk소유를연결해최종exit0/JSON으로정정했다. product/testFAIL이나새formalFAIL→PASS가아니다. 짧은ASTpreview의UTF8바이트중간잘림도unicode문자단위미리보기로바로잡았으며최종AST분석에는영향없다.

probe_phase29_ci_selection.cjs / phase29-ci-selection-probe.json은 현재 CI의 순수 securitySweepSamples와classifier의isSampleSecuritySweepPath를 source에서 추출해10입력으로 실행했다. added3개(.HWP/.hwpx/.hml)만 CI에서선택되고 modified/renamed/removed는제외되며, pathpredicate자체는status가아닌samples/접두와확장자만판정한다. CI의added선택→workflow환경JSON→Rustclean_sweep_documents연결을구분했다. 원격CI실행은없다.

phase29-call-and-data-dependencies.json에include_str/#[path]16개source입력의hash,동적CIcorpus와recipes디렉터리,포함catalog모듈을연결했다. catalog의commands→COMMANDS const→spec·in_help→help_group·in_capabilities·Category::as_str 의존을읽었다. catalog::find는이test에서import/call하지않으며일반Iterator::find와동명이란이유로실제실행했다고세지않는다. 전체repo호출폐쇄는아직false다.

## 원장·외부 상태

audit-ledger-summary:10808groups/2108files. unreviewed4042 / function_px_policy_adjudicated5929 / function_evidence_pending507 / semantic_notes118 / file_context212. 직접6436/723 모두현재sourceSHA와원장에연결됐고누락0. 총분류미확정은4042+118+212=4372이며,507독립근거pending은이미직접읽은기록안에있다. 이숫자를확인된위반수/해결수로쓰지않는다. phase29-semantic-ledger-validation.json. 제품/검사수정·개명0. 전체audit/fullconformance/전체callclosure미완료.

#8232 OPEN updated_at2026-10-26T00:11:00Z. phase29-final-issue-body.md와API재조회body는말미개행포함byte동일. SHA5139e5b329166a4954317457d333277d22c4eb4597ec84e7e6c626618682f252, phase29-remote-verification.json. 이번102함수·60Native/관찰108·CI10분류·실제conditional실행한계·근거와잔여를본문에반영했다. close/PR/완료체크하지않았고goal ACTIVE.

## 다음 조사

현재HEAD/dirty/실제프로세스와producer부터확인하고끝난이번60개/이전45·39개를자동중복시작하지않는다. 미분류상위는 tests/hwp3_password_fixture.rs9, tests/run_plan_contract.rs9, tests/unicode_deception_contract.rs9, tests/cases/agent_chief_skill_contract.rs8, agent_doc_triage_cli_ladder.rs8 등이다. 실제문서출력의검사가필요하면 issue_3587_*의블록/템플릿/클론·issue_4367_hwp3_convert_fourth_contract/issue_4764_pdf_raster_fidelity 등현재원장의미분류와함수별범위를확인한다. 파일숫자는원장그룹수이며전체함수/위반/PASS수가아니다. 이전pending507와공용/생성/조건부/E2E경로도남아있다. 전수조사완료후실제부적합보정·issue_8232개명·등록/호출갱신·검증을완료해야전체목표완료다.
