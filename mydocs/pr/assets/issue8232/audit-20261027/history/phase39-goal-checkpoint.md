---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase39-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase39-goal-checkpoint.md` · 원문 SHA-256: `888b7e67cf011baec0dc8ea6f840791d2d9edfcd4b72324630b8a90202f54971`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 phase39 감사 체크포인트

작업장 /Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD 238de6de41a208648f4ade6b494fe433a81c1fce. tracked dirty 없음. 이전 goal turn은 phase38 실제 실행/독립 좌표/전쪽 시각/이슈 갱신을 완료한 progress였다. 이번도 새원인 반례와 source/Print 단위의 독립 근거·원장·이슈 게시를 완료한 progress다. 전체 목표 ACTIVE, 전수 감사/실제 보정/개명 완료 아님. PR승인없음. 에이전트/예약/메모리수정 없음.

## 현재 원장

직접7432기록/775소스파일. 원장11569그룹/2134파일: unreviewed3812, adjudicated6903, 의미메모113, pending529, 파일맥락212. 미분류3812+메모325=4137. pending529는직접기록안에포함. full_conformance/callclosure 미완료. 신규35/기존close1재판정,36함수/5파일(자동번호자리표9/셀높이폭5/빈문단서식10/첫줄표여백7/새연속표5). 제품/정식검사수정0/개명0.

## 실제 실행

run_phase39_existing_tests.py는현manifest suite011/014/015/024에서12실등록/12PASS/0FAIL, summary0.062초, 필터1066skip. source/실행binary전후SHA같음. 최종session82934terminal0. 최초session17451은sourceSHA수집시saved/blank2010.hwp경로를tests/saved로잘못잡아FileNotFoundError,실제nextest run시작전종료했다. 최초inventory/log는phase39-first-inventory-*와note로보존했고경로수정후한번정식실행했다. 제품/검사FAIL로계수하지않음.

세fixed pr7468입력edited/deleted/stored-control과기존유효Print를현재Native/freshWASM각전4쪽새산출. canonical96CSSdpi/2px/compat2022/capture총8,양쪽rawmin100/미달0/누락추가/쪽수불일치0. session31701terminal0, state=phase39-empty-paragraph-evidence/state.json. MacWASM은lockedweb/pkg/no-opt로컬대체동일artifact, Docker통과아님. 이fixedfixture시각증거를모든live API편집결과바이트동일이나다른표문서전체수용으로확대하지않는다.

## 새 실제 검출 누락 1원인

column_top_table_reserves_outer_margins.rs: assert_text_clears_table38/root96. 기존root의정상사본1PASS,실제get_page_render_tree JSON에서source문단2셋째문단TextLine각1개를제거한사본도1PASS. 현재/HWP재열기/HWPX재열기/여백변형×빈/비빈입력총8관측모두정확히1개제거,기존상대기하단언은그대로. 남은줄이표바닥+원본여백보다아래인지/비어있지않은지만확인하므로셋째문단소실을놓친다. phase39-content-counterexample.json, phase39-content-probes/{baseline,successor-line-missing}-{compile,run}.log. root/helper2직접기록같은원인보정대상. 실제제품이본문을누락한다거나새정식FAIL→PASS/CI등록이라고주장하지않음. later correction에서는기존비겹침관계유지+독립입력문자/문단소유/내용누락·중복보호추가/issue_8232새이름/간접등록갱신필요.

## 원본 단위와 PDF 기준선 혼동 정정

빈문단close±0.05는13/20pt×160%source의줄상자전진과현재호출전후/저장동등성으로정상이다. 독립CFB에서C13pt/D20pt, vpos2080HU전진/빈줄3200HU삭제이동확인. 최초감사스크립트는D도13pt라고가정하고PDFglyph기준선D-C를줄상자전진과직접비교해AssertionError를냈다. 이는감사가정오류이며제품/검사결함아니다. 원본줄상자와glyph기준선은다르다. 원본삭제이동42.6666667CSS vs PDF기준선삭제이동42.5135091CSS를각관측값으로보존,타이트한px정답으로PDF값을덮어쓰지않음. 초기로그phase39-independent-source-and-print-initial-assumption.log, 정정driver/정본JSON/log에source raw/table와source크기,Print 원문소속을남겼다. 실제한컴stored raw문단41 vpos0경계,첫쪽001~041/다음쪽042~070 전70줄소유/순서독립확인. 원본source/PDF SHA는README/현재파일일치.

## 판독·생산물

정본review/standaloneoverlay같은crop10영역(양쪽8backend/page),NN2배6시트모두직접판독. TitleABCD/빈공간삭제,C13/D20,Line001~070모든내용/쪽소유확인. glyph획/래스터미세차이남음,raw100을RGBA동일로해석금지. Nativeeditedp1 fullreview와별도overlay도직접열었다. 최초cropcaption이WASM행에도Native/Print라고되어RHWP/Print로고침. phase39-reading-sheets-v2와현재manifest가정본. 앞서읽은6시트와caption행밖RGB전픽셀동일검증/전후SHA/producer근거로내용판독을재사용했고정본review/overlay/PNG/TSV는변경0. 이전captionPNG/manifest는보존. scripts오류/판독재사용을숨기지않는다.

원7468/7558/7613/7614/7615/7656는미병합CLOSED,실통합7686/7601/7882. 자동번호후속8191까지merge조상/current파일바이트동일확인. phase39-origin-chains.json/각PRmetadata. 단순merge_commit_sha존재만으로원PR을병합취급하지않음.

validate_phase39_ledger.py 현재PASS. 시각catalog158조건/각backend2647쪽(실패/재사용포함). #8232 fresh본문이phase38 snapshot과byte같음을확인후phase39추가,API재조회EOF포함byte일치. updated_at/SHA는phase39-remote-verification.json. 완료Native12/분석2실행/시각3조건4쪽양쪽을중복재시작하지않는다. 라이브Cargo/rustc/nextest/capture는최종ps로확인할것.

다음에는남은integration기하와helper/동적/Studio경로및sourcecfg독립pending65개(HeightCursor39포함)를계속판정한다. 현재unreviewed중source_rs가0이더라도기존sourcepending65·호출폐쇄/독립근거미완료라source감사완료로확대하지말것.7902의20pending도과거98/NativePASS만으로수용금지. 전수조사→이슈판정→부적합실제보정/issue_8232명명/간접수집호출갱신/재검증의전체목표를유지한다.
