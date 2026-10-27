---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase43-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase43-goal-checkpoint.md` · 원문 SHA-256: `4d31345e3b694ef69333be9beaf53ce5167c2e303e2c361b07d9cf6e5bb5b9d7`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 43차 조사 체크포인트

R=/Users/tsjang/rhwp, D=R/output/issue8232-audit. branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce, clean. 실제43차22실행/반례/독립CFB·좌표·전15쪽 dual/판독/원장/이슈 갱신을 완료한 progress다. 전체 목표 ACTIVE/미완료. 제품·정식 검사 보정0/개명0, PR/push/새 issue/comment/예약/agent/메모리 수정 없음.

누계 직접7537/781파일(독립 근거 pending501 포함). 원장11616그룹/2134파일, adjudicated7036/pending501/unreviewed3754/의미메모113/파일맥락212. 분류 미확정4079. 43차는 세 파일1263줄/46함수(22검사24helper), 신규45/기존pending1 재판정, 새 원장22그룹. integration55/helper48. source-prefix override 없음. record_phase43.py는신규카운트가정이있어다시실행해새진척으로세지말것.

실제 Native suite012/022/026 22등록/22PASS/0FAIL summary1.093s,754필터제외. producer/source/binary 전후SHA동일. 모든 관련 Cargo/nextest/capture terminal. 별도 source분석에서 TAC2 fixture를silent continue대신읽기/파싱/삽입오류를드러내고실제두case1PASS(trace1쪽/5쪽)확인. phase43-tac-observed.log. 정식추가등록test1개라고합산하지않는다. 완료22/전15쪽을중복시작하지않음.

새 검출누락1cause/1root: tests/table_extract_json_contract.rs:102 export_tables_preserves_merged_spans. 원시CFB table00119×9/131앵커셀/171격자칸은모두1회소유. 실제CLI JSON의(0,4)1×1셀을(0,0)복제로교체하면셀수131/면적171유지지만한칸중복+한칸누락. 기존병합/봉투/범위검사정상3PASS/변형3PASS. 면적합만으로중복금지의충분조건삼은약점. 제품중복출력/정식FAIL→PASS완료가아님. 원래병합·범위검사유지하고source격자소유/유일성보완대상. 다른중첩/컨테이너rawTABLE레코드2/3개도대조. phase43-cell-duplicate-counterexample.json/probes,independent-raw-contracts.json.

기존px유지근거1: tests/issue_1892.rs:245. 같은현재공개export_hwp_with_adapter와curved_visibility_variant본문을ignoredRusthelper로실행했다. public-saved d067d6b0..., none-curved73aa5162...,visible-curved8644cdff...는PR8191 README의정확한독립Print입력과byte같음. 원본ab6bb4e...와4조건각2쪽을현재Native/freshWASM으로재캡처. p1 raw N/W original99.71888/99.71890, saved/invisible99.73096/99.73097, visible99.68124/99.68126. 빈p2는모두100이며실제PNG/PDF이미지16개잉크없음확인. 기존원문소유/순서/누락/중복·가시Line1/비표시Path1·끝점순서와±2좌표는정상유지대상이다.

독립PDFiumpath에서기대좌표없이최장filled세로치수선을고르고동일geometry의PDF중복paint2개를구분했다. shaft가아닌y극점arrowtip. 원본PDF (96.668,671.146)/(96.668,127.301)pt의841pt상단좌표와기존기대차<0.000006CSS. 같은공개저장Printtip은670.906/127.660pt라기존기대와0.319992/0.478672CSS차이,기존±2이내다. 초기감사checker는두독립PDFtip완전같음(0.002내)을잘못가정하여FAIL했고초기로그보존후그가정을제거했다. 제품허용폭변경아님. current qualifier는기존±2와각입력의독립tip을대조한다. phase43-independent-arrow-endpoints.json/log/initial-check.log. 이혼동으로기존검사를먼저바꾸지않았다.

추가source shortcut원본7쪽 N/W min96.75856/96.74946(p3),90미달/쪽수차이/누락0. 5조건총15쪽/backend,141.9337s완료. p3는98자격없음. 원시CFB문단16first0HU확인,현재280문단/firstzero73은별도실제HwpDocument관측과동일(phase43-shortcut-model-count.log). 옛PR/comment76과현재관측을구분하며73기대핀을새로만들지않음. 편집8조건전부의Print/paint를원본7쪽으로대신하지않는다. source HU·current-callgeometry≤1/용지비율1HU·affineEPSILON·탭null마커·Column인덱스는절대pagepxgolden과다름. 분류는수치의미이며모든본문/실제glyph/모든호출폐쇄의완료아님.

판독: phase43-reading-sheets.json의14preview/16영역으로도형4조건p1과shortcutp1/p7각backend를개관했다. 최저가p3임을확인한뒤추가phase43-minimum-reading.json의두backend review/standaloneoverlay4이미지도직접읽음. 최종대표7쪽/backend+핵심9crop2x직접판독. phase43-reading-details.json. group양끝·비표시동반개체·본문과표,shortcut양단/p3입력/p7본문·큰숫자대조. 미세획/색상/글리프차이남음. cubic명령보존을임의곡선전체paint정확성으로확대하지않음. 모든15쪽2x읽었다고쓰지않음. 현재pairedcatalog166조건·각backend2675쪽(실패/재사용포함).

출처: #1892원PR1905/merge23aea574,후속8191/60972534; #2299실제PR2314/467ca361; #3278실제통합3304/f68aa8be. metadatamerged/조상/sourcebyte대조. 현재세파일은각최신merge와byte같음. 원1892보고서와group-source-semantics README/3278보고서,원PR2314·3304본문읽음. 잘못열었던task_m100_7642_stage2314.md는2299근거로쓰지않음. independent CFBscanner는생산parser를호출하지않고표/셀원시주소와lineSegHU를읽음.

validate_phase43.py PASS. #8232fresh42본문일치→43누적→갱신직전재조회→게시후EOF포함byte일치. updated2026-10-26T04:00:38Z/bodySHAf5747d099be62c708ba301a6fc0cf5c4abe0cbe6e66c1e5ed75cd1c9ef62efa3/59594문자. phase43-final-issue-body.md/remote-verification.json/semantic-ledger-validation.json. 65000문자한도다가오므로후속누적전에자기이전조사서술만로컬원문·근거보존하며압축필요할수있음. 사용자/외부편집보존. 전수조사/보정/개명완료체크금지.

다음: 남은integration/source cfg/helper/dynamic/Studio호출·독립pending501을계속판정. sourcepending38(phase40의12포함),7902의20pending등과현재source연결필요. 이번46/22/15쪽/반례3+3/관측1완료는반복하지않는다. 실제보정/issue_8232파일·함수개명/공용helper,module,manifest,생성·script/E2E참조갱신/필수집중·전체검증은전수조사뒤전체목표대로계속한다. user PR later/무승인PR금지 유지.
