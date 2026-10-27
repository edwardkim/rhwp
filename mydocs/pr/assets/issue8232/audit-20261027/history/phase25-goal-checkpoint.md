---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase25-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase25-goal-checkpoint.md` · 원문 SHA-256: `011dbed2d29ef3a65619da185d16f6216128bbdea57aa02ac006eb25d79be014`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 24~25차 체크포인트

목표 ACTIVE, 완료 아님. 직전22~23차는 원장·실행·반례·원격 이슈를 갱신한 progress였고, 이번24~25차도 직접 판정122개 추가·실제 실행·새 source-cell 검출 반례·현재 전쪽 시각 출력·원격 이슈 갱신을 수행한 progress다. blocker나 현재 기다릴 실행은 없다. 전수 조사/독립 판정 이후 실제 부적합 검사 재작성·issue_8232 새 명명·helper/module/suite/manifest/생성/스크립트/E2E 호출 갱신·필요한 공통 출력 보정과 필수 재검증을 마쳐야 목표 완료다. 기존 단언/검출 방식 자체의 무조건 보존은 요구하지 않는다.

## 작업장/권한

R=/Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/최종 Git clean, diff --check0. 제품/검사 수정·개명0건. 새 자료는 전부 ignored output/issue8232-audit 아래이며 원본/PDF/다른 작업장/공유 target을 보존했다. PR/push는 아직 하지 않는다. subagent/다른 task 메시지/예약/메모리 쓰기 금지 유지. memory 8232/audit quick 검색은0건이며 관련 memory를 사용하지 않았다.

## 24차 직접 판정

세 파일 전체를 직접 읽었다.
- tests/cases/issue_7911_pdf_frontend.rs617줄:15검사/3helper.
- tests/cases/issue_7998_pdf_border_structure.rs371줄:10검사/8helper.
- tests/cases/issue_7998_pdf_table_composition.rs1064줄:8검사/24helper.

batch39/helper30은33검사/35helper=68함수. 이전4개 기록을 보완한 것을 제외하면 고유64개 증가했다. 당시 직접6057개, 원장10606그룹/2107파일, unreviewed4219/pending487 등. phase24-semantic-ledger-validation.json은 당시 snapshot으로 이후25차 원장과 숫자가 다르다.

PDFfrontend의200x200pt/12pt/30,140pt는 작성 입력의 수치이며 화면 기대 위치가 아니다. 1x1 RGB/CMY image bytes/저장 참조·HWP wire format·같은 core의HWPX비영향·source page선택/162page·byte/연산 budget·오류후재시도·원본/provenance/publicAPI/실제SVGtext순서를 검사한다. schema버전/64MiB/순서metadata와 실제pagepx를 구분했다.

Border structure는 삽입/삭제/inverse/undo-redo/양저장 후 공통BorderFill·child비영향·새셀원문복제금지·실제cell선구간coverage를 확인한다. 실제cellbbox 기준0.5 여유/틈 관계와 절대PDF핀은 다르다. 원본stroke색0.2/0.4/0.6→BGR0x996633을 독립 확인했다. JSON child 모델 동등·동일시점HWPX snapshot은 historical SVG layout golden과 다르다. 테두리계획오류/비정상crop·mask에서 원자적비변경을 확인한다.

Composition은 actual nested owner·source chars/그림데이터·cell 포함·BEFORE/AFTER·원본 crop 분할의합집합/비중첩을 구분한다. model/paint 문자를 sort한 multiset은 셀별소속/읽기순서를 보호하지 않는다. inline 그림 baseline85%의 독립기준과 전체 caller 소유/순서는 여전히 pending이다. source 선 길이8건을 입증해도 전체호출폐쇄를 완료로 표시하지 않았다.

## 24차 독립 PDF 원본

inspect_phase24_pdf_sources.py / phase24-independent-pdf-contracts.json은 production parser 없이 pypdf로29개 PDF 원본operator·CTM·stroke·width·text Tm·image payload를 읽었다. 8periodic source의 축별stroke구간합집합길이는 기존기대2480/2250/2150/2830/2955pt와 정확히 같고 모두0.5pt폭이다. source A=(48,265)pt,B=(118,265)pt이다. blue source는 RG0.2/0.4/0.6이다. 원본 길이×0.5는 centerline extent proxy이며 native0.8CSS pen의실제면적과 다르다. 문서화된HWP0.2mm의600dpi5tick→0.8CSS 반올림과 원본0.5pt를 구분한다.

composition-sources.md·stage4 일부·stage5 관련부분·native border renderer의단위근거를 읽었다. 원/통합PR 전체provenance완료나현재모든PDF전쪽시각자격을 주장하지 않는다. 실제검사변경전에원해결/실제통합/후속변경을연결해야한다. 원본/기대값/문턱을바꾸지않았다.

## 24차 실제 source-cell 반례

probe_phase24_cell_ownership.py exec session94789 terminal0. 현재composition검사source를 output probe에만복사하고 debug rlib SHA5768866158c10fc9e1336cc286e2daa3a8d548baaaf602a89359a2e8de217c27에연결했다. 실제입력 nested-periodic-child.pdf를읽어 기존 verify(core,2,expected,2480)를 먼저실행해PASS, clone IR에서자식row0col0의A→B/row0col1의B→A를교환한뒤기존helper를다시실행해PASS했다. 두 actualpaint/기하검사가 모두끝까지실행됐다.

기존verify는모델/paint문자multiset·tablecount·source선extent·cell포함·BEFORE/AFTER관계까지확인하지만 sourcecell별원문소속이틀려도통과한다. 보정대상이다. 현재제품이원본문서를잘못import했다는증거/정식root전체mutation/새formalFAIL→PASS로표현하지않는다. 별도compound leaf-owner helper는각cell별정확한A/B/C/D를검사하므로같은약점으로일괄판정하지않는다. provenance/run/sourceSHA는 phase24-cell-owner-probe/result.json 및run.log에있다.

run_phase24_existing_tests.py session46165 terminal0. 현재metadata native-skia suites017/020/025 build14.36초, 기존정식33PASS/0FAIL Summary0.542초, 선택target775skipped, elapsed1.673310042초. testbinary전후SHA동일. 전체회귀/새검사FAIL→PASS/전체시각수용이아니다.

## 25차 직접 판정과 실행

다음세파일전체를직접읽었다.
- tests/agent_toolkit_contract.rs698줄:13검사/9helper.
- tests/capabilities_schema_contract.rs500줄:17검사/5helper.
- tests/scan_contract.rs293줄:10검사/4helper.

batch40/helper31은40검사/18helper=58함수이며기존direct기록중복0,고유58개추가다. 명령집합/dispatch·JSON봉투·버전/참조/field배선·상태코드·파일byte수·임시corpus/정렬/symlink/limit·같은현재호출fingerprint·chunk계획커버를페이지px와구분했다. no_orphan 정의참조합집합을root전체도달성증명으로확대하지않았다. 그별도일반스키마문제를픽셀감사범위밖에서임의보정하지않는다. scan의field-01 3쪽은독립현재원본근거를아래와같이별도연결했다.

run_phase25_existing_tests.py session19020 terminal0. Native/native-skia suites015/022/023 freshmetadata build14.33초. 기존정식40PASS/0FAIL Summary2.718초, 선택target774skipped, elapsed4.152736458초. Unix symlink조건은현재Mac에서수집됐다. testbinary전후SHA동일. nextestmeta의non-test rhwp/rhwp-agent 실제경로와실행후SHA도 qualification에기록했다. CLI바이너리의실행전SHA를새로확인했다고주장하지않는다.

## field-01 현재 전쪽/쪽수

run_phase25_field01.py session68489 terminal exit0/34.459574167초. fcntl capture.lock, 총capture8. 원본518cb939079e6e0640a5f813597f744e2528a17ca52ee418929f1c8f4b5380c0와 oracle9d18eb5a5fd1d9db28a331ec85727762a2ecec11의원본바이트가같다. PDFec488080ac777c355444bf98e0d0fb34ef02600870313bcaa212aff4500d8d0b,129044B/3쪽. CreatorHwp2022이며파일명2020으로실제MCP engine/job/PrintMethod를추정하지않았다. 새MCP변환을시작하지않았다.

같은auditHEAD의 frozenNative5c9aed5b...와freshWASM0a81d25f.../JSba97ed3f... package를hash확인후재사용했고PNG/TSV는새로생성했다. Mac lockedwrapperweb/pkg/no-opt 로컬대체빌드재사용이며새Docker최적화빌드가아니다. canonicalvisual_sweep.py/PDFium5.13/96CSSdpi/compat2022/2pxraw/NativeSkiaPNG·WASMPrintCanvas Canvas2D이다. PNG와전쪽보조TSV양쪽exit0/capturegatepassed,각3/3쪽/누락0/미달0/raw100,99.99969,100%. 두TSVSHA d33e6448e06d4aace2b4acccd939adb9160cccf8499c99b4f262389c59467c6d가같다. 출력PNG는Native/WASM0/3byteequal이므로판독을backend간재사용하지않았다.

Native1/2/3·WASM1/2/3의canonicalreview와standaloneoverlay총12개를직접열었다. 표지마케팅전략기획서/회사·작성자·부서/Tel/E-Mail,목차01..05,마지막쪽01.과하단장식의해당쪽소속을확인했다. 새blank/주요내용누락/중복쪽을관측하지않았다. 사진·글리프윤곽/색상raster잔차는남고엄격RGB동일성/폰트완전일치를주장하지않는다. analysisPNG를변형하거나점수기준을바꾸지않았다.

qualify_phase25_field01.py는실제nextest CLI의readonly scan --probe도3쪽임을확인하고원본/PDF/SHA/metadata/PNG/SHA/직접판독에연결했다. 새절대px핀추가/다른caller전체호출폐쇄완료가아니다. 기존scan metadata의3쪽을임의로완화하거나일괄삭제하지않는다. px핀을새로검사하지않아독립좌표/확대핀자격을확인했다고하지않는다.

## 현재 원장/원격

direct6115개/709소스파일이builder와같은sorted우선순위로원장에모두연결·현재sourceSHA일치·누락0이다. 원장10630그룹/2107파일: unreviewed4185, directmeaning5628, evidencepending487, semanticnotes118, filecontext212. 모든full_conformance/callclosure는아직완료아니다. 기록숫자를결함수/PASS수로표현하지않는다. pairedcatalog는116조건/각backend2591쪽/입력PDF228파일이며기존미달도포함한다. phase25field01을추가하고compat2022및실제capture/TSVgate·source/artifacthash를확인했다.

#8232 body를갱신했다. OPEN/updated_at2026-10-25T23:15:44Z, phase25-final-issue-body.md와API재조회body를말미개행까지실제byte동일확인했다. SHA dd806992bf2dadbbaab42a54cb04b429d4ec31534c8a37d3627158124c3021e2. phase25-remote-verification.json이다. 이슈완료체크/close하지않았다. 실검사수정/개명0건,PR없음. 최종실프로세스에driver/Cargo/nextest/capture/rustc가없음을확인했다.

## 다음 작업

미분류4185/pending487·공용/동적/조건부호출폐쇄를계속확인한다. 우선남은pixel/glyph연결파일예시는 tests/cases/issue_4969_shaping_shadow_context.rs994줄,issue_4969_shaping_atomic_activation.rs1832줄,split_caret_axis.rs711줄등이다. 아직이번에전체읽기/새판정/실행하지않았으므로완료로처리하지않는다. 이전root/캡처/반례실행을중복시작하지말고현재HEAD/dirty/process/log부터확인한다.

전수조사뒤부적합검사의실제재작성·issue_8232개명과동일호출/등록수집갱신·필수검증까지완료해야한다. 현행px90/98/±2기준을완화하거나정상원본HU/관계/구문계약을이름변경을위해훼손하지않는다. PR은나중에사용자지시를따른다. 목표ACTIVE유지.
