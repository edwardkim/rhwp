---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase18-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase18-goal-checkpoint.md` · 원문 SHA-256: `b59bfe353d48b1478113262a798293bc33b9f356161278ac25254727723d3c5e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 18차 체크포인트

목표는 ACTIVE이다. 전수 조사·이슈 업데이트·부적합 검사 실제 보정·issue_8232 새 명명·helper/module/manifest/생성/스크립트/E2E 연결·필수 재검증이 모두 완료 조건이다. PR/push는 아직 하지 않는다. subagent/다른 task 메시지·예약·메모리 쓰기도 하지 않는다.

직전 질문 응답 턴은 문구 설명만 했으므로 no progress로 분류한다. 이번 턴은 실제 검사 반례 실행·289함수 직접 판정·증적 오류 정정·현재27개 재실행·원격 이슈 업데이트를 했으므로 progress이다. 진짜 blocker는 없다. goal을 완료/일시정지/차단으로 바꾸지 않았다.

## 작업장

- R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025.
- HEAD=238de6de41a208648f4ade6b494fe433a81c1fce, 마지막 git status clean.
- 제품·테스트 수정/개명은 아직0건이다. 전수 조사 뒤 실제 보정한다는 순서를 유지했다. 새 자료는 모두 ignored output/issue8232-audit 아래다.
- 기존 7445 worktree는 제거된 상태라 사용하지 않는다. 공유 target/pr-review·입력·독립 PDF·다른 작업장은 보존한다.
- 전역 memory 검색8232/audit는 hit0이라 관련 memory를 근거로 쓰지 않았다. 저장소 authority는 현재 파일을 확인했다.

## 직접 판정

batch32: 7파일 검사96/helper48=144함수. agent_codex_skill_contract(407줄), ontology_contract(460), root_binary_contract(60), agent_strategist_skill_contract(432), agent_visual_regression_skill_contract(284), render_diff_json_contract(651), redact_sanitize_contract(772)를 전체 직접 읽은 이전 phase18 이어서 현재 source SHA를 검증해 기록했다.

batch33: 9파일 검사100/helper45=145함수. eps_artwork_to_svg_contract(234), issue_3695_structure_auto_policy(206), nextcall_cli_contract(228), issue_7218_page_break_at_paragraph_start(427), cli_exit_codes(321), cli_exit_code_support(46), fields_json_contract(383), explain_contract(234), search_json_contract(287)를 전체 직접 읽었다.

합계16파일/196검사/93helper=289함수. record_integration_batch32/33.py, integration-semantics-batch-32/33.json, non-numeric-helper-semantics-23/24.json. batch33 초기 합계를110tests로 잘못 쓴 assertion은 실제 목록100으로 정정했으며 source에는 영향이 없다.

작성 EPS BoundingBox/행렬/font-size/색상, 저장 break bit/문자 오프셋, 구조·필드·검색 논리 주소, CLI API threshold와 실물 페이지 px 핀을 구분했다. render-diff 기본1px를 PDF ±2px 정책으로 바꾸지 않는다. 제품 전체 호출 폐쇄/독립 시각 수용은 별도이며 full_conformance false다.

## 실제 기존 검사 반례 3개

1. agent-codex-runtime-failure: 현재 suite008 실제binary에서 capabilities 검색 정상1PASS, 실제 호출한 fakeCLI exit1에도1PASS. 실패 return이 검사 생략. 관측은 이전 phase18에서 완료했고 중복 실행하지 않았다.
2. visual-skill-runtime-exit-code: 현재 metadata suite009에서 정상1PASS, fakeCLI exit2에도1PASS. runtime1/usage2를 혼용한다. observe_visual_skill_exit_code.py.
3. search-cell-address-owner: 현재 metadata suite003에서 정상1PASS, 실제 응답의 cell 주소6개를 전부 제거한 proxy 응답에도1PASS. 필수 주소 선택이 optional if-let이다. observe_search_cell_owner.py.

세 정상+세 반례의 기존 검사 총6회가 실제PASS다. 제품은 정상이며 새 검사 FAIL→PASS가 아니다. 각 observation.json에 source/input/binary/inventory/log SHA와 실제호출 근거를 고정했다. 원본은 수정하지 않았다.

Codex/visual 원 이슈5318/5312 전체 본문/댓글, 원PR5328/5320, 통합5444/e79f113, 현재 git log/도입 함수 본문을 확인했다. phase18-agent-original-history.json. 두 잘못된 조건은 통합 시점부터 있었다. 검색 원PR 계보 나머지는 실제 수정 전에 계속 확인한다.

## 본문 helper: 무효 시도와 최종 제한 근거

sanitize-body-response-schema의 최초 actual proxy 시도는 무효다. run helper가 compile-time env!를 써 runtime env override를 소비하지 않아 proxy호출0이었다. 두 기존 검사PASS를 변형 응답 수용 증거로 쓰지 않는다. INVALID_ACTUAL_PROXY_ATTEMPT.json/invalid-proxy-driver.py.txt를 보존했다. 해당 observer exec29340은terminal exit1. 다른 정상 문서 검증 실패가 아니다.

observe_sanitize_body_schema.py를 실제 read-only 응답/source predicate 대조로 고쳤다. frozen Native field-01 export-text 실제3쪽/156글자, missing/null 응답 둘은 다르지만 기존 unwrap_or_default helper가 같은 빈 값으로 바꾼다. 최종 observation.json은 이 predicate 반례만 주장한다. 제품 본문 손실·실제정식검사 변형주입·새FAIL→PASS가 아니다. 실행은terminal exit0이다. 원본은변경없다.

20개 보정 후보 기록은15개 missing-fixture 함수·helper/caller2개 text누락·검색실패/종료코드/소유주소 각1개다. 원인별 독립결함20건으로 세지 않는다.

## 바이너리 provenance 자체 오류 보정

15/17차 producer가 nextest 실행 후 캐시된 이전 binary 경로에서 SHA를 읽었다. 실제 실행 요약은 유효하나 그 SHA는 실행 증거가 아니다.

- phase17-existing-tests-producer-before-provenance-correction.json에 원본보존. current metadata phase18-current-tests-list.json의5개 target경로를 조회했다. Finished0.38초/추가compile없음, currentbinarymtime<=원producer를 대조한 연결근거를 정정producer에 명시했다. phase18-binary-provenance-comparison.json. 기존33PASS/0FAIL요약불변이다.
- phase17 qualification도 원본을 existing-pins-qualification-before-phase18-provenance-correction.json에 보존한 뒤 qualify_phase17_existing_pins.py로 producerSHA 재연결했다. 옛phase17 receipt는 정정 전 historicalsnapshot이며 새producer와 일치한다고 주장하지 않는다.
- phase15producer 원본도백업. cachedbinarySHA를 실행증거에서철회했다. 현재 metadata suite016 actualpath/SHA/sourcehash를 고정하여 기존27개를 직접binary/threads8로 다시실행했다. exec35226terminal exit0,27PASS/0FAIL,1.216743초. phase18-para-table-existing-tests.log/producer.json. 예전27개PASS와 이번27개PASS를 중복distinct검사로 세지 않는다.

## 원장/이슈

- 최신10,346그룹/2,105파일. unreviewed4443, directmeaning5118, evidencepending446,semanticnotes127,filecontext212.
- 직접판정5564개가 원장에 모두연결,누락0/현재sourceSHA확인. phase18-semantic-ledger-validation.json.
- paired visual catalog96조건/각backend2505쪽은변경없음. 이번턴 새캡처 없음. 이전미달/보류/자격제한을 없애지않는다.
- #8232본문과댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6403774851 갱신후 remote/local본문·댓글정확일치 확인. phase18-final-issue-body.md/phase18-survey-report.md/phase18-published-receipt.json(26증적hash).
- 모든observer/metadata/current27개 실행은종료했다. 마지막ps에서Cargo/rustc/nextest/observer실행없음. 다음작업전재확인.

## 다음 작업

4443미분류와pending독립근거/동적helper·fixture·production소비연결을 계속한다. 기존 source미수정finding은 phase17/16/15 이하checkpoint에서 이어받는다. 기존이름/단언을 무조건보존하는 조건은적용하지않는다. 새issue_8232명명과 실제등록검증도 전체완료요건이다. 이번16파일/기존3개반례/현재27개를 처음부터 중복실행하지않는다. 다음직접후보는 현재원장에 남은 rowbreak_short_row_whole_carry(842줄),issue_6812_square_picture_tac_table(787),layout_anomaly_contract(592),issue_536_native_accessibility(518),issue_6639_cell_para_format_vpos(508)등이다. 아직읽지않았으므로 판정완료로세지않는다.
