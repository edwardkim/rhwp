---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase27-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase27-goal-checkpoint.md` · 원문 SHA-256: `67ec7612230319cbdec3ffdd55501db8362847586b534c1e0a3b724841b056dc`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 27차 중단점

## 목표와 현재 권한

전수 조사·독립 판정 → #8232 업데이트 → 실제 부적합 검사 보정/필요한 구현 수정 → 보정 파일·함수를 issue_8232로 새 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 소비 갱신 → 실제 재검증 순서다. 전수 조사 완료 전 실제 테스트 재작성/개명은 시작하지 않았다. PR은 나중이며 push/PR 승인 없음. #8232 본문 업데이트와 필요한 MCP Print는 승인돼 있다. 다른 task/agent/예약/메모리 쓰기 금지.

사용자의 모순 지적을 반영했다. 기존 기대값·좌표·판정 방식 자체가 잘못됐다면 수정하거나 제거하며, 기존의 잘못된 판정을 보존할 의무는 두지 않는다. 독립 근거로 확인한 내용·순서·소속·저장 목적은 적절한 검사로 유지/대체한다. 실제 보정 항목만 issue_8232로 개명하며 이전 이름 wrapper로 중복 등록하지 않는다. 정상 검사까지 대량 개명/삭제하지 않는다.

## 현재 작업장

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. Git clean. 제품/테스트 수정0·개명0. 기존7445작업장은 제거돼 사용하지 않는다. 공유 target/pr-review·원본·PDF·다른 작업장은 보존한다. jobs8/threads8/capture합계8/Cargo순차를 유지한다. 현재 프로세스에 Cargo/nextest/rustc/Python driver가 없다.

AGENTS/CLAUDE/현재 visual_fixture_evidence 의미 계약을 따른다. PDFium5.13/96CSSdpi/2px raw이며 실물 절대 좌표는 같은 쪽 Native+freshWASM 각각98% 이상·독립 좌표·직접 확대 판독·±2CSSpx·의미 관계가 필요하다. source HU·bytes·CU·선언 크기/상대 간격·동일 호출 cache/serialization 계약은 화면 절대 px 핀과 구분한다. 모든 관련 쪽90%/내용/쪽수와 수정 전후 근거는 검사 변경 전에 필요하다. 현재 권위 문서의 사양이나 문턱은 변경하지 않았다.

## 이번 직접 조사

네 파일 전체를 읽었다: tests/issue_1058_textbox_list_header.rs(389줄), tests/issue_554.rs(521줄), tests/issue_1143.rs(524줄), tests/cases/issue_2792_search_nested_tables.rs(255줄). functions/assertions의39tests/23helpers=62functions를 integration-semantics-batch-42.json / non-numeric-helper-semantics-33.json에 source SHA·직접 단언으로 연결했다. prior overlap0, 새 고유62개다.

독립 raw/XML 5입력: 두 HWPX header/section0의 bullet4개(❏/※/soft-hyphen/❍), style14개의langID1042/각주style11, 실제MSB id, rect/drawText1개·각주2개를 읽었다. CFB3개에서 fn/gso control ancestry·LIST_HEADER16/33·후행13byte0·각주PARA_HEADER char_count18/controlmask262144/style11을 읽었다. source bytes/논리offset은 실물 px 핀이 아니다. LIST_HEADER size가 있다는 것과 해당 owner 전체의 보존은 다르다. source 배열 전체 동일성을 확인하지 않는 predicate의 범위를 명시했다.

#554의10입력에 대해 지정 독립PDF의36/64/64/16/16/16/64/35/74/30쪽이 기존 기대값과 모두 같다. phase27-independent-contracts.json에 exact input/PDF SHA가 있다. 현재 paired catalog에 같은 input hash가 있는 것은 HWP5 sample/sample5 두 건뿐이며 다른 형식·다른PDF의 whole증거를 재사용하지 않는다. 그2건 최소치는 각각99.14995/99.45903과93.37483/93.37577이지만 현재 선택PDF의 출처/내용소속/직접판독 자격을 자동 승격하지 않았다. 나머지8건 및 fixed page owner 자격은 pending이다. 그림gap sourceoffset+margin+LS의HU/75 환산과0.2px 직렬화예산은 absolute page pin이 아니므로 무조건±2로 바꾸지 않는다.

중첩 표 검색은 명시 HML의고유token4개·깊이0/0/1/2·outer cell logical paragraph1·중첩page생략·DTO필드·limit2·치환1회 계약이다. source크기HU는 입력제작이다. depth정렬 root 단독은 중간토큰누락을전부 검출하지 않지만 별도 root가각4개1회를검사한다. 이 두 범위를 구분했다. 실제 glyph/page 배치 근거로 사용하지 않는다.

## 실제 실행과 반례

run_phase27_existing_tests.py는 현재 generated suite003/014/028에서4모듈39개를 actual metadata로 고정해 native-skia로 실행했다. phase27-existing-tests.log Summary6.022초:39PASS/0FAIL/758선택target skipped. 준비빌드14.90초, wrapperelapsed7.435초, 실제3바이너리 전후SHA동일. 이름에wasm/CanvasKit인검사도 Native API/plan이며 실제WASM/browser재생으로 주장하지 않는다. template에남은40개scope문구만39개로정정했고 producer에정정근거를남겼다. 명령/선택목록/summary는 처음부터39였다.

probe_phase27_contract_predicates.py / phase27-predicate-probes/results.json: 공유debugrlib SHA5768866158c10fc9e1336cc286e2daa3a8d548baaaf602a89359a2e8de217c27을 연결한 분석용 복사본이다. external read_sample에 한정해 rdb02.gif/s1.jpg 원본 대신 oracle.gif를 공급했다. 원본3파일SHA/format/크기는서로다르다. 기존9root 모두PASS였지만 실제영향은 across3formats/key2/3basenamecache의3root다. 다른6개는변형영향없음/원래oracle입력이다. 주입metadata/계획/payload존재수는실제imageidentity를검출하지못하는범위로기록했다. native directory root는read_sample미사용으로변형영향없다.

별도1058복사본의 load에서 line_spacing_v2를모두1, 첫bullet을A로바꿔도 기존synced/bullet2root가PASS였다. 독립XML PERCENT130/150/160/170과❏에맞지않지만 기존단언은nonzero만검사한다. 단위계약자체를삭제하지않고 source value identity 검출범위를pending으로기록했다. 실제제품그림/각주실패·새formalFAIL→PASS·실제전체WASM/CanvasKit시각수용으로확대하지않는다.

첫probe는 load_doc/read_sample에같은read표현2개가있어선택assert에서중단됐다. 아무harness컴파일/실행증거가없던실행이며 probe_phase27_contract_predicates-first-selector.py.txt에보존했다. read_sample함수안으로선택을한정한최종실제실행만반례증거다. 분석용컴파일의unused warning은정식lint PASS가아니다. 정식source/test수정은없다.

## 현재 원장/외부 상태

audit-ledger-summary.json:10699groups/2107files, unreviewed4117 / function_px_policy_adjudicated5746 / function_evidence_pending506 / semantic_notes118 / file_context212. 직접6252records/714sourcefiles, 모두원장연결/현재sourceSHA동일. phase27-semantic-ledger-validation.json에확인했다. 숫자분류나focusedPASS를전체적합완료로세지않는다. 원장전체call_closure_review는여전히pending이다.

#8232 OPEN, updated_at2026-10-25T23:44:47Z. phase27-final-issue-body.md와API재조회body는말미개행포함byte동일, SHAefa5ba2067ca142273dca61569cc849cf5fcba4e58285b4235cf9d7d6feeb17e. phase27-remote-verification.json. 정책문구정정+이번62함수/독립근거/39PASS/반례/미완료범위를본문에반영했다. close/PR/스케줄생성없음. goal ACTIVE 유지.

## 다음 조사

현재 미분류에는 tests/injection_scan_contract.rs11그룹, tests/cases/cli_catalog_contract.rs10그룹, bookmark_delete_preservation.rs/issue_536_native_clipboard.rs/issue_6874_hwp3_objtype3_is_a_table.rs/issue_1251_ole_chart_contents.rs각9그룹등이있다. 파일단위숫자는전체함수수와다를수있다. 현재HEAD/dirty/실프로세스·이전producer부터확인하고같은실행을중복시작하지않는다. page픽셀과logical주소/원본형식/작성치수구분을이어가며모든미분류·pending·소비경로를조사한다. 기존 26차 bbox origin/size-receipt pending 등은해소하지않았고그대로남아있다. 이전phase27/26집중검사나캡처를자동재시작하지않는다. 전수조사/독립판정후실제부적합기대값보정·issue_8232개명·호출/등록갱신·필수검증까지완료해야전체목표완료다.
