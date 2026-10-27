---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase38-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase38-goal-checkpoint.md` · 원문 SHA-256: `aad57c8ee857b69a578f41333bf08992f7f1f4e12ff40b84995e04c10648ba77`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 phase38 감사 체크포인트

브랜치 audit/8232-px-regressions-20261025, HEAD 238de6de41a208648f4ade6b494fe433a81c1fce. tracked dirty 없음. phase38 Native43/캡처29입력 실행은 모두 terminal이며 중복 시작하지 않는다. 사용자 질문에 대한 직전 설명은 실행 진척이 아니었고 이번에는 phase37 검증/게시 및 phase38 실제 감사·실행·독립 좌표·전쪽 시각·이슈 갱신을 완료했다.

## 새 근거

semanticist21_qualified_print_positions.rs 837줄/40함수/29검사와 issue_7902_completed_page_geometry.rs 455줄/30함수/14검사를 전체 읽었다.70직접그룹 중68신규/기존near2재판정이다. 현재 generated suite004/023의43검사를 등록해43PASS/0FAIL, summary4.348초/필터512skip,wrapper5.3303175초. source/실행binary 전후SHA같음. phase38-existing-tests-producer.json/log를 따른다.

semanticist21 README의29입력/PDF SHA를 실제 확인, canonical96CSSdpi/compat2022/2px/총capture8로 Native/fresh WASM 전42쪽씩 새 산출. 양쪽 raw최저100/미달0/쪽수 불일치 및 누락/추가0. source/producer 고정 및 script SHA는 phase38-semanticist-evidence/state.json.8분대 실행 완결(session31400 terminal). WASM은 Mac locked-wrapper web/pkg/no-opt 로컬 대체artifact이고 Docker최적화 아님.

PDFium 원문 문자 원점과 path/object matrix로94개 독립 좌표를 내용으로 선택해 재대조, 최대 차이0.0005604655CSSpx. pypdf exploratory visitor가둘째단의한 text origin을0,0으로보고해원문API로다시읽었으며잘못된0,0을제품결함으로취급하지않았다. phase38-independent-pdf-positions.json/log. 글자마다 source identity/본문/각주소유와좌표를구분한다.

정본review/overlay동일crop88영역(84backend/page),2배nearest확대12시트 모두 직접 읽었다. 본문/셀/머리말탭,각주/미주,그림뒤번호,폭/한두자리/TotalPage/PAGE,부분문단/양쪽정렬,좌우단/구분선 끝을 확인. 획/래스터의미세차이가남고raw100을RGBA동일로주장하지않는다. 선별판독원장은phase38-reading-sheets.json. 직접 열린fullsize정본은 Native단구분선p2review+별도overlay2장이고나머지는정본crop시트로읽었다. cropper최초실패는pdf_001/png이름가정(pdf-001실제)과고정30px라벨가정(실제32px)이었고,실제파일목록/정본label산식으로수정했다. 최종각crop의overlay가별도overlay원본과바이트같음검증. 이도구오류를제품/검사FAIL로세지않는다.

29개기존검사는현재양쪽98/독립기대±2/내용관계가유효하므로이름만바꾸지않는다.7902의14root+6helper20기록은현재선정쪽자격/독립좌표재대조pending. 과거835쪽/선정98과NativePASS만으로현재수용하지않는다. 기존catalog의HWP vpos5쪽만으로HWPX/다른문서를대체하지않는다.

원7663/7670/7671/7673는미병합CLOSED.실제7935/e4e6d0c8,다른파일은7941/84764fa2의현재조상과merge당시source바이트동일확인. metadata와phase38-origin-chains.json. merged_at없는APImerge_commit_sha는병합증거가아님.

## 누계·현재 한계

직접7397/771파일,원장11548그룹/2134파일.미분류3826+의미메모113+파일맥락212=4151.독립근거pending529는직접기록안에포함.시각catalog155조건/각backend2643쪽은실패/재사용포함목록이며전부PASS수가아니다. 새부적합확인0/제품검사수정0/개명0/전체감사미완료.

validate_phase38_ledger.py가sourceSHA·실행등록/PASS·29입력/PDF·전쪽42×2/gate·94좌표·88crop/12판독시트·실제merge ancestry를현재상태에서통과했다.#8232본문freshfetch기존snapshot바이트일치뒤갱신,재조회EOF포함일치. updated_at/bodySHA는phase38-remote-verification.json.

다음은남은4151미확정및529독립pending의내용/원본/페이지/호출/실행근거를확정한다.특히7902의현재선정쪽자격을과거자료로대체하지말것.완료43/29문서42쪽양쪽캡처는자동중복실행금지. 전수조사→이슈판정→실제부적합보정/issue_8232개명/간접등록갱신/재검증의전체목표ACTIVE. PR승인없음, 새schedule/agent/메모리수정없음.
