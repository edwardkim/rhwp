---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase40-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase40-goal-checkpoint.md` · 원문 SHA-256: `dad2f03f79fa5c488926d30dba75b2214c29c3645686de1bf34ad1ba979e3f53`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 phase40 감사 체크포인트

R=/Users/tsjang/rhwp, D=R/output/issue8232-audit. branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. tracked dirty 없음. 이전phase39는실제새소실반례/source단위/전4쪽/원장/이슈갱신을완료한progress. 이번phase40은source39재판정/실제39실행/원문단언검출력반례/원본XML·CFB·산술/출처보정/이슈갱신을완료한progress다. 전체목표ACTIVE/미완료. 제품/정식검사보정0/개명0. 승인없는PR/push/새issue/comment/schedule/agent/메모리수정없음. 한국어,클릭가능로그,독립review+overlay규칙유지.

## 누계

직접7432/775파일과원장11569그룹/2134파일은같음. source HeightCursor이미pending직접기록39를재판정한것이며신규39로더하지않음. 정상source단위/작성상태관계26,검출누락보정대상1,독립근거pending12. 전체pending529→502,source내pending65→38. unreviewed3812+의미메모113+파일맥락212=4137미확정. adjudicated6930. pending은직접기록안에포함. 전체호출폐쇄/fullconformance는미완료다.

## 실제 실행

run_phase40_height_cursor_tests.py, session98644 terminal0. 현재native-skia --lib nextest등록에서정확히39선택/39PASS/0FAIL summary0.065초,필터3831skip. wrapper1.550577초이며선행listcompile2m13s를테스트시간으로혼동하지않음. source/binary 전후SHA같음. RHWP_VPOS_DEBUG=1/--success-output immediate로실제단위진단로그보존. phase40-existing-tests-producer.json/log. 완료39재실행금지. root별trace상중간result와최종return을항상같다고확대하지않는다.

## 새 검출누락 원인1

HeightCursor test compact_endnote_deep_backtrack_allows_safe_new_note_title(line3198)은assert!(got<980)하나뿐. 작성입력COL_Y100/순차980/prevLS6000HU의내용바닥900에비해잘못된899/99후보를원문단언그대로컴파일한replay가허용.950정상후보허용/980상한후보exit101거부로단언실행확인. phase40-predicate-counterexample.json/phase40-predicate-replay/*.log. 이것은마지막단언검출력재생이며전체원unit test나생산cursor를그잘못된값으로실행한것아님. 현재실제39unit은PASS,해당진단result964로하한침범제품결함을주장하지않는다. source현재검사하한/이전내용점유/단포함보완대상으로기록했다. 정식FAIL→PASS/CI등록은아직없음.

## 26 단위/관계 대 12 pending

39함수전체3103~3953선별본문과작성para/styles/cursor/mark_note_first/multilinehelper를읽었다. source3103/3117등명시page-base와sourcevpos/직전점유끝관계,간격한번예약과활성좌표base보존은실물절대pagepxgolden과다르다. 1e-6만으로일괄삭제/±2변경하지않음. 대상별basis는source-semantics-batch-36.json이다. 전체생산분기정확성이나모든실물Print수용으로확대하지않음.

미확정PENDING lines={3145,3216,3298,3345,3457,3487,3509,3557,3583,3612,3636,3698}.11개는4/10/16/18/30/40/41px생산완충/cap과동일기대의독립선언/점유/Print근거미확정. 3145는base6800HU기대의anchor소유가미확정. 최초independent math가fullend78800과순차980을대응시켜12800HU로계산했으나생산은후행6000HU별도차감→6800HU다. 이는점짝선택의근거문제라생산식을독립정답으로복사하거나기대12800으로수정하지않음. 초기실패로그phase40-independent-units-initial-point-pair.log보존,정본JSON에unresolved_anchor_pair로남김. 해당2개lazybase/target산술을독립증명10건에넣지않음.

inspect_phase40_units.py는원본samples/3-09월_교육_통합_2022.hwpx의endNotePr/noteSpacing betweenNotes1984를직접ZIP/XML로읽고,2022HWP와2024미주사이20HWP의tag74offset20에서1984/5669를독립CFB로확인. 선언HU/동등좌표/점유바닥의유리수산술10건PASS. 그산술은40/18/10/4px의근거/실물98자격이아니다.

## 출처 정정과 실제 소비

inspect_phase40_origins.py는39함수16도입commit/10최초조상merge를구했다. 최초조상merge는기능branch에devel을가져오는commit일수있으므로실제PR통합이라고자동취급하지않게정정했다. 실제GitHub metadata조회에서1178/1232/1259/1264/1292는merged=false/merged_atnull이다. 옛commit제목Merge PR과API실제상태를분리했다. 각도입commit을포함하고함수가실제있는1194/1247/1294/1743의확인된merge를연결했다. 현재함수body와처음확인된병합body가같은것11개,나머지는후속변경있음을기록. predicate3198의약한단언은당시병합과현재양쪽에존재한다. phase40-origin-chains.json/phase40-pr-*.json. 과거1189 stage1/2는현재working경로없어commit c8b81708에서ignored output으로원문확보했으며currentdocs로위장하지않는다.

현재typeset3442/3522의측정소비와layout8596의배치y_offset소비를확인. helper/분기문맥은height_cursor현재원점·간격보정소스이고모든후속paint의호출폐쇄완료를이3위치로대신하지않음.

## 게시·남은 일

validate_phase40_ledger.py가현재source39원장/SHA,실제39실행,원문단언4후보재생,XML/CFB/10산술,실제병합/도입조상을PASS. phase40-semantic-ledger-validation.json/log. 새시각캡처0/기존catalog158조건·각backend2647쪽그대로(실패/재사용포함목록).

#8232는fresh본문이phase39snapshot과byte같은지확인후phase40을누적,API재조회EOF포함byte같음. updated_at/SHA는phase40-remote-verification.json. 본문현재52778문자라65000한도접근시자기이전조사서술을근거보존하며압축할수있지만사용자/다른사람수정은보존할것. 전수조사완료로체크하거나PR생성하지않음.

다음은남은integration기하/helper/동적/Studio및sourcepending38(이번12포함)의독립근거판정과호출경로를계속추적한다.7902의20pending은과거98/현재NativePASS만으로수용금지. 완료한39Native/단언replay4후보/원시단위읽기를자동중복시작하지않음. 실제보정/issue_8232새파일·함수명/공용helper,module,manifest,생성,script/E2E참조갱신/필수재검증은전수조사뒤에진행하는전체목표를유지한다.
