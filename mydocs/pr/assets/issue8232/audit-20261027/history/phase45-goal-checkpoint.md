---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase45-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase45-goal-checkpoint.md` · 원문 SHA-256: `4bca048e9dae6b05a70d7a3d1d98af2e19fbc223a68c4efacab47087149490fc`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# 45차 인계 — 목표 ACTIVE

승인된 순서/범위는 phase44-goal-checkpoint.md와 동일하다. 전수 조사→이슈 갱신→부적합 기대/방식 실제 보정 및 필요한 공통 구현→실제로 보정한 파일/함수 issue_8232 새 명명/관련 호출등록경로 갱신→필수 재검증. 정상 검사 일괄 개명·옛 이름wrapper·중복등록·무승인PR 금지. subagent/task메시지/예약/메모리쓰기 금지. 이번 턴은 실제Native·양backend캡처·독립PDF원점/원문XML·누락반례·직접판독·원장/이슈 갱신을 완료한 progress다.

현재R=/Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce, tracked clean. D=R/output/issue8232-audit. 목표ACTIVE이며 전수감사/실제제품검사수정/개명은 미완료/0/0이다. 공유target/pr-review·원본/PDF·다른작업장 보존. Cargo순차/jobs8/testthreads8/capture총8 유지. 45차nextest45113·capture51007·counter74202·association3837·ledger59078·catalog73946 전부exit0회수. 완료한실행을다시시작하지말것. 현재Cargo/nextest/capture 없음.

5파일39함수 직접판정: equation_completed_document_boundaries(12),issue157_stored_before_host_frame(3),issue1749_synthetic_cell_spacing(11),issue4138_reparsed_split_frame_cache(9),issue4628_dump_pages_production_height(4). 모두tests/cases/원본. 신규38/기존pendingnear91 1재판정, 초기원장16존재/검색밖23추가. record_phase45.py 첫added24가정이실제23과달라assert실패했으나원장쓰기전이었고수정후성공. 그사이에기존원장재생성만있었으며가짜진척아님. 최종integration57/helper50. 직접7699/796files, 전체원장11714/2138files. state미분류3690/adjud7199/의미메모113/독립pending500/파일맥락212. classification unresolved4015, pending500은직접기록안에포함. source-prefix override없음. sourcepending38/7902의20등다른독립pending은계속남는다. record45는최초증가가정이있어재실행해신규로세지않음. validate_phase45.py PASS.

Native12등록/12PASS/0FAIL summary1.985초,1303필터제외,elapsed2.960초. source/binary전후hash같음. phase45-existing-tests-producer.json/log. 기존derive정확한suite를현재manifest에서계산했고소스/수집개수는변경하지않음. 새Rust제품/검사변경없음. 실제CLI1749export-render-tree runtimeTMPDIR는D/phase45-runtime-temp,검사소유TempDir만자체drop. 이PASS가전체실물피델리티/감사완료는아님.

새검출누락1cause/1root: tests/cases/issue_4138_reparsed_split_frame_cache.rs:116 table_prefix_keeps_its_saved_anchor_and_all_nested_rows. 원문source모델의행열·셀내용·lineSeg앵커/제어슬롯보존을확인하지만실제placed표는행열/가로bbox포함만본다. 같은현재debugrlib SHA576886...로ignored분석원본1PASS,실제렌더트리중첩표의TextRun만제거한변형1PASS.25쪽owner322에서18개,156쪽owner2001에서19개제거했음을실제로그로확인. 원래단언·모델·bbox·행열그대로다. 제품이이누락을현재실제로발생시킨다거나정식보정FAIL→PASS완료주장금지. 정상앵커/행열·원문보호유지하고실제cell내용소유/누락검출을보완해야한다. probe_phase45_split_paint_loss.py,phase45-split-paint-counterexample.json/probes/driver.log. 기존sourcefirstroot58의749/1189글줄/줄비겹침·셀포함은관계검사로구분한다.

수식5기존pxroot정상유지자격/near91기존pending해소:
rm/signs Native/WASM100/100,Greek99.97856/99.97862,fraction99.57489/99.53928,state99.64866/99.67399. 모두동일입력독립Print전1쪽·compat2022·96CSSdpi. 원시PDFium26글리프origin을PDF문자순서/내용소유로선정(기대좌표근접검색아님),72ptbottom→96CSSpxtop환산. 기존기대값과max0.0000390625CSS차이. check_phase45_independent_origins.py/phase45-independent-pdf-glyphs.json/independent-origins.json. 이helper의미래임의caller까지자격자동확장금지.5기존test실제PASS. 기호/수식소유·순서·선택글리프단일출현·Σ/y첨자·Ππ·분수/후속항·정체공백·첫/마지막줄예약관계와±2px유지. glyphshelper는지원typedtree를순회하지만멀티문자Text/FontGlyph원점전체를일반oracle로확대금지. with_document는좌측/세로포함이고가로오른끝까지체크한다고과장금지. ±합성은Print에도주변본문겹침있음. raw100을strictRGB같음이라하지않음.

새capture: run_phase45_frame_equation_evidence.py/O=phase45-frame-equation-evidence. frozenNative5c9a.../freshWASM0a81.../JSba97.../현재canonicalscript hash고정,기존Maclockedwrapperno-opt산출물재사용이며Docker/새WASM빌드주장금지. capturelock총8/main guard.8조건각backend16쪽=7조건전12쪽(수식5+HWPissue157전2+1749X전5)+split의검사지정25/60/89/156 4쪽. 전체197새완료아님. source/PDFfullpagecount197과캡처선택4를구분. 전16선택쪽미달/누락/중복/쪽수불일치0,220.26초. split4최저99.99156양backend,1749전5최저99.99836/99.99835,157HWP전2각100.현재엔진/source다른옛97%관측과비교해원인규명없는개선주장금지.

직접판독: phase45-reading.json,make_phase45_reading.py.수식5+p157둘째+1749넷째+split4=대표11쪽/backend preview를모두도구로읽음. 수식독립PDF26원점의9영역각backend를정본review3패널에서같은좌표crop→nearest2x확대하여도구로직접읽음. standaloneoverlay와review3번째패널같음assert. 미세획/분수선/anti-alias차이남음. figure정합/분수첫마지막·정체m/s/A/B/b/c확인. 참석장·위임장·끝안내,1749별줄상하/간격,split25/156표와60/89본문소속preview확인. 전체197쪽/전16쪽모두확대판독했다고하지않음.

1749원본XMLparaPr11 caseprev500HWPUNIT/fallback1000을독립확인. sourceIR두배간격계약과줄높이1/10인관계오차는abs pagepx아님.주석1024/1017.29/962.15/993.15는assert된핀아님. p157호스트의1e-8상대비겹침·source주소/표내용선택은그대로유지.4628현API total/production/구성합일치1e-9·fallback분기0.01은역사적pagepin아님. 다만두번째root는실제diverged발생수를요구하지않으므로항상옛fallback결함검출/독립한컴레이아웃정답으로과장하지않는다. 추가counter를만들거나이root를새결함으로세지않았다.

기원 actualmerged8148/2e087(수식),7882/86a(host),8228/3e02(1749),8191/609725(split),6480/19b89(4628),각조상/currentbyte일치. phase45-origin-chains.json/PRmetadata/commitassociations.8148/8064과거PR자동래칫을다시연결하지않음. 단순히원출처를조사한것이다. CanonicalPDFium현재프로젝트권위를사용하며PDFskill의Poppler선호가현행프로젝트정본을바꾸지않았다. PDF생성/교체없음.

paired-evidence-catalogbuilder에phase45의whole/selected스코프명시지원추가. 기존같은task1749x5쪽재실행은replace해서중복조건으로추가안함. 따라서166→173조건,2675→2686쪽/backend(새캡처16이지만기존5대체로순증11). 실패/재사용포함이며모두PASS라는목록아님. splitrecord.whole_document_visual_complete=false,selected4목록/fullpdf197분리. 수치자격외독립/직접판독자격은semantic원장/reading에별도연결. build_paired_evidence_catalog.py는ignored도구이며제품visual_sweep나자동품질gate변경아님.

44차producer scope문구가3파일22검사로복사되어있던설명오류만바로잡음(실제57등록·명령·결과/hash는변경안함). 이전영수증의원문을phase44-existing-tests-producer-before-scope-correction.json으로복구해SHA가이전receipt와같음을검증했다. phase45receipt는수정본과원본을별도로연결. 이전receipt를현수정본과같다고주장하지않음.

#8232는fresh44본문byte일치→45누적→직전재조회→게시후EOF포함byte동일확인. updated2026-10-26T04:36:43Z,최종63927Python문자/JS63932. bodySHA는phase45-remote-verification.json.남은약1k문자로다음게시전자기옛조사만로컬원문/영수증보존후압축이필요할수있다. 외부/사용자편집보존. 승인된작업계속하며전수완료/실제보정/개명완료또는PR생성으로표시하지않음.

다음은미분류3690·메모325·독립pending500에대한source cfg/integration/sharedhelper/dynamicfixture/script/E2E호출/독립근거판정을계속한다. 특히새반례목록보정/개명은전수확정뒤전체목표범위에서진행. 이번39/12/16쪽/26origin/1+1probe/11preview9detail은완료했으므로반복실행하지않음.
