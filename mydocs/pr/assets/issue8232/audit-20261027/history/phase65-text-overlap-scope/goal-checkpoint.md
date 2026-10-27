---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase65-text-overlap-scope/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase65-text-overlap-scope/goal-checkpoint.md` · 원문 SHA-256: `b0d39767ee5f69cb362b5b7fcbb5da117b0045942be86281eaae1fcf49864c7a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 65차 감사 체크포인트

- 현재 브랜치 `audit/8232-px-regressions-20261025`, head `01ced3d42724285c8f9765a453393ab6af326617`. 삭제검사 및 신규95개Rust/신규JS2case는 감사·PASS에서 제외했다. 같은동기화를반복하거나새검사를재편입하지않는다.
- active-audit.json은65차원장을가리킨다. 누적직접8,425함수·Rust5,888검사/756파일, 원장12,107그룹/2,139파일. 의미판정7,925·별도file-scope4·pending500·미분류/문맥3,678. 전체감사미완료.
- 64차37함수/20검사,65차18함수/2검사로이번재개에기존55함수·Native22 PASS를확인했다. 이22PASS를누적전체검사의현재head PASS로확대하지않는다.
- 65차공식Native사진2PASS/0FAIL·0.034초·빌드/등록13.843초/307필터제외. 기존text_overlap_baseline.rs의18함수와macro16등록/0..15dispatch를확인했다. macro16wrapper는함수계수에중복합산하지않고file-scope1개로분리했다.
- 실제기존collector를그대로실행해활성1250입력·16분할union동일/중복0을확인했다. sample16/편람두형식은활성목록에없다. 원장92행모두파일존재/중복0,90행만활성이다. 제외2행은issue1949 giantperf 두형식으로deferred_visual_regression_inputs.tsv text_overlap축에이관됐다. 원본/기준값은보존했다.
- 실제count_doc를원서4쪽과단축키7쪽두원본에만실행해각각0을확인했다. 전체1250입력16분할렌더PASS는실행/주장하지않는다. 독립Print4/7쪽의내용/metadata/해시와원본SHA를연결했다. 과거98.77240/96.13등을현재head의전쪽통과로재인용하지않는다.
- 변경하지않은공식suite바이너리의16개macro를격리한상대경로corpus에서검증했다. 정상16PASS/orphan행16PASS/선택된파싱실패15PASS1FAIL. 실제원장에누락이관측된것이아니다. 각분할에속하지않은행을제외하는기존계약이며,전역orphan감사의한계를새px결함/삭제검사재강제/보정범위확대로오인하지않는다. scope-interpretation.json에기록했다.
- 사진helper는중심소속/내용/원형개체와교차pair를검사한다. 전체bbox포함·모든paint가시성을검증하는것으로확대하지않는다. 단축키helper도master-page소유와자동번호표시를검사하며bool만으로paint z순서전체를검증하지않는다. 포함된printed_photo_seals 모듈은현재파일에서호출되지않으므로실행근거로세지않았다.
- #6316→#6328, #7505/#7832/#8191/#8228을추적했다. #8266도live로읽고snapshot보존했다. #8266은LineSeg없는inline-control-move두HWPX의기존배치결함이며현재px감사에서엔진보정범위로확대하지않고이슈본문/완료기준을수정하지않았다.
- 64/65 validation 모두PASS는증적/분류/계수의일관성확인이다. 새px보정대상0·제품/검사수정0·개명0·PR/push없음. 이전63차의Native90미달4쪽/98미달14쪽및기존pending500은그대로남는다.
- #8232의자기현재상태만갱신하고EOF까지exactreadback확인했다. body UTF16길이65,528로65,536한도에가깝다. 다음갱신은우선자기정상진행요약만대체/압축하며과거실제실패·외부본문·#8266·완료조건은보존한다. 본문수정전전체before/fullstatus와역복원proof를보존하고가용길이를먼저계산한다. 실패하지않은과거정상실행세부1단락만요약에서제거했고원문은archive로보존했다.
- 별도dirty `rhwp-studio/public/rhwp.js` SHA `c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6`는보존한다. 작성자추정/복원/커밋금지. sourceNative/입력은불변이다. 공유target/원본/PDF/다른작업장보존.
- 현재Cargo/캡처모두종료. 다음작업전ps/로그확인하고기존검사의독립판정을계속한다. 직전turn은증거와원장이진전된progress이며blocked가아니다. 전체목표미완료로ACTIVE유지,새agent/task메시지/예약/메모리쓰기금지. 빌드jobs8/testthreads8/capture총8/Cargo순차.
