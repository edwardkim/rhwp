---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase23-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase23-goal-checkpoint.md` · 원문 SHA-256: `62dd872a337159912df0bff24e2ac1e964356ff812813c916411a73b6c94ce40`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 22~23차 체크포인트

목표 ACTIVE, 전수 조사·검사 보정·개명 미완료다. 직전 대화의 모순 설명만 한 턴은 authoritative state/evidence를 바꾸지 않은 no-progress였다. 이번에는 현재 작업장/이슈/실행을 재확인하고 직접 판정155개 추가·독립 원본·실제 집중 실행·반례·원격 이슈 갱신을 수행한 progress다. blocker나 현재 대기할 실행은 없다. subagent/다른 task 메시지/예약/메모리 쓰기를 하지 않았다. memory 검색 8232/audit는0건이고 관련 memory를 사용하지 않았다.

## 작업장과 범위

R=/Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/마지막 Git clean, diff --check0. 제품/검사 수정과 개명0건이다. 모든 새 스크립트·조사 기록·로그는 ignored output/issue8232-audit 아래다. 원본/PDF/공유 target/다른 작업장을 보존했다. PR/push 승인 없음, 지금 PR을 생성하지 않는다.

현행 지침과 독립 근거로 유효한 목표를 재구성한다. 옛 단언/검출 방식의 무조건 보존은 하지 않으며 정상 원본HU/관계/구문/저장 계약을 이름 변경만을 위해 훼손하지 않는다. 실제 보정한 파일/함수만 issue_8232로 새 명명하고 module/helper/manifest/generated/script/E2E를 갱신해야 한다. 원 이름의 wrapper/중복 등록은 남기지 않는다. 전수 판정 이후 실제 보정과 필수 검증까지 마쳐야 목표 완료다.

## 22차 — 대각선/각주 예약

앞선 context에서 두 파일 전체를 직접 읽었으며 이번 실행에서 핵심 source/원본/실제 등록을 재대조했다.

- tests/issue_1623_cellzone_diagonal.rs 987줄,19검사/5helper.
- tests/cases/issue_7379_rowbreak_table_footnote_reservation.rs 1868줄,14검사/47helper.
- batch37/helper28:85함수, 기존20개 보완을 제외하면 고유65개 증가. 직접5903개, 당시 원장10514그룹/2106파일, unreviewed4281/pending472 등. phase22-semantic-ledger-validation.json은 당시 snapshot이며 이후23차 글로벌 원장과 숫자가 다르다.

inspect_phase22_reachability.py는 tree-sitter Rust body 식별자로 함수 포인터/find_map(fn)/map(fn)/macro token tree까지 연결했다. 문자열/주석은 제외하고 동명 nested helper는 모두 연결하는 보수적 과대 근사다. 실제 generated suite023/026 attachment와33개 Native root를 대조했다. 각주14root에서 helper23개(최상위20/중첩3)가 미도달이다. 여러 절대 PDF좌표 helper가 여기에 속한다. 나머지24helper 모두의 실제 실행을 주장하지 않으며 전체 repository 동적 호출 폐쇄/안전한 삭제 완료도 아니다. dormant helper를 현재 active FAIL/PASS로 집계하지 않았다. tracked filename 검색에는 archive 기록이 많고 현재 generated는ignored라 별도로 직접 읽었다.

원본 대각선 표는3x4, sample3/4/5는10x10이다. 초기에 진단 script가 모든 표10x10을 가정해 실패했다. invalid-all10.py.txt/phase22-raw-source-first-invalid.json에 보존하고 전제를 수정해 다시 확인했다. 제품/test 실패로 세지 않는다. XML/CFB4쌍에서 row/col/cell/zone/BF와 flags를 직접 읽었다. 원본 note240(paragraph1865)의 XMLvpos[0,0,1172], 표728캡션1000HU도 독립 확인했다. 이는 실제 페이지 절대 px 핀이 아니다. 작성4195x1282HU와 선 길이 관계, y<350/x130..155/y140..150의 화면 선택은 구분한다. 활성73..84/70..85 범위 검색의 범위 밖 소유/중복은 여전히 pending이다.

run_phase22_existing_tests.py exec session38002 terminal exit0. fresh Native/native-skia metadata suites023/026, build12.91초, 실제33PASS/0FAIL, Summary10.150초, 선택 target478skipped. run elapsed11.151039833초. binary 전후SHA 동일. 전체 회귀/새FAIL→PASS/독립 전쪽 visual이 아니다. 새 capture를 시작하지 않았다.

#8232를22차 결과로 갱신한 상태는 updated_at2026-10-25T22:54:48Z였고 다음23차 직전 live fetch와 posted body가 동일함을 확인했다. phase22-final-issue-body.md는 당시 snapshot이다.

## 23차 — WMF/thumbnail/legacy chart/helper

이번에 다음 네 파일 전체를 직접 읽었다.

- tests/cases/issue_6865_wmf_monochrome_mask_rop.rs663줄:17검사/20helper.
- tests/issue_3366_thumbnail_contract.rs347줄:11검사/10helper.
- tests/issue_4098_legacy_chart_grid.rs451줄:10검사/3helper. 이 중 generator1개는 기존ignore.
- tests/support/issue_4055_chart_probe.rs407줄:공용 helper21개, 명시 path-module·동적 corpus·binary 변종 경로.

batch38/helper29:38검사/54helper=92함수. 기존2개 보완을 제외하면 고유90개 증가다. 직접5993개/704소스 파일이 현재 원장·source SHA에 모두 연결되고 누락0이다. 현 원장10573그룹/2107파일: unreviewed4250, 직접수치의미5509, evidencepending484, semanticnotes118, filecontext212. 전수 조사/전체 호출 폐쇄 완료가 아니다. source-side unreviewed0도 전체 적합 완료와 다르다.

### 독립 원본

inspect_phase23_source_contracts.py:

- CFB PrvImage17256B를 직접 읽고 Pillow 크기/format을 대조했다. frozen current CLI thumbnail --base64 --json을 decode한 바이트가 실제 CFB stream과 동일하다. CLI frozen SHA5c9aed5b... 고정. 이는 원본 내장 preview 전달이지 현 조판 출력 golden이 아니다.
- samples/chart를 재귀 순회한 HWPX28개/대응HWP28개 SHA를 고정했다. 각 ZIP Chart XML에서 c:val 또는c:yVal의 pt idx/v를 직접 읽어 계열값을 확보했다. 묶은세로막대형 4.3/2.5/3.5/4.5 등3계열은 독립XML과 같다. production OoxmlChart를 독립 oracle로 재인용하지 않았다.
- 월간 수출입 원본HWP의 BinData/BIN0003.wmf를 raw-deflate로 풀면 fixture92032B와 바이트 동일하다. 원본DIBBITBLT 레코드를 직접 스캔했다. D=0..255/M=0,255에 (D XOR217)AND M XOR217의512항등식은 brush/destination 선택을 독립 확인한다. 원본WMF source의 확인을 문서 전쪽 visual 완료로 확대하지 않는다.

### 실제 문자열 predicate 반례

probe_phase23_wmf_predicate.py exec session61005 terminal0. 현재 test source47..190의 작성 helper를 output generator에만 복사하고 current debug rlib SHA5768866158c10fc9e1336cc286e2daa3a8d548baaaf602a89359a2e8de217c27에 연결했다. rustc 출력/바이너리는 output에만 썼고 shared target을 바꾸지 않았다.

정상24bit authored WMF의 production 변환 SVG에서 visible top-level rect2개를 모두 제거하고 defs/title/desc/metadata만 남겼다. 기존 issue_6865_multibit_pattern_blit_is_still_painted의 svg.contains("rop_pat")는 원본/definitions-only 모두 true다. visible paint가0이어도 통과하는 predicate 반례이며 보정 대상으로 판정했다. 제품 mutation/기존 formal test 자체의 반례 재실행/새검사FAIL→PASS/실물페이지90% 실패를 주장하지 않는다. 다른 문자열-only 검사들은 각각의 원본·visible reference 의미가 남아 pending이다. bit truth-table/장치 면적/비영향/결정성 계약은 정상 보호로 구분한다.

### 실제 실행

run_phase23_existing_tests.py exec session38846 terminal0. generated suites007/026 및 standalone issue4098 target의 current Native/native-skia metadata38개 중 기존 ignored generator1개를 제외한37개를 실제 실행했다. Summary0.824초 37PASS/0FAIL, 선택 target481skipped. 실제 바이너리 경로와 전후SHA 동일. generator/새Print/전체회귀/시각 capture는 실행하지 않았다. 기존 ignore를 추가/완화하지 않았다.

## 원격 이슈와 최종 상태

#8232 live body를23차 결과로 업데이트했다. OPEN, updated_at2026-10-25T23:00:46Z. 재조회 body를 phase23-final-issue-body.md의 실제bytes와 말미 개행까지 비교해 동일함을 확인했다. body SHA9832e39e91debd15cee16c0b3b5a11b9de704ea1664da7c3a019769e07af9709. phase23-remote-verification.json/phase23-semantic-ledger-validation.json에 근거가 있다. 완료 체크박스를 채우지 않았다.

Native/freshWASM 새capture0건, paired catalog115조건/각backend2588쪽은 이전 근거 그대로이며 이번 새통과로 재인용하지 않는다. 기존33/37 focused PASS를 전체회귀나 정상 visual 완료로 세지 않는다. 시작/종료 모두Git clean, 실제Cargo/nextest/capture/rustc/driver가 남아 있지 않다.

## 다음 작업

미분류4250와 pending484 및 source/공용/동적 소비 폐쇄를 계속 확인한다. 남은 우선 integration 후보에는 tests/cases/issue_7911_pdf_frontend.rs(617줄), issue_7998_pdf_border_structure.rs(371줄), issue_7998_pdf_table_composition.rs(1064줄) 등이 있다. 아직 전체 읽기/분류/실행하지 않았으므로 이 파일을 완료로 처리하지 않는다. audit-ledger/functions/assertions의 현재 실제 파일/등록에서 시작한다. 직전22/23차 기존 focused 실행과 probe를 중복 시작하지 않는다.

전수 조사와 독립 판정 이후 실제 부적합 검사 보정/issue_8232 새 명명·호출/수집 갱신·필요 출력 보정·영향 범위 full회귀/lint/build/시각 검증을 완료해야 한다. 현재 검사변경/개명0건, 목표 ACTIVE이며 PR은 나중에 사용자 지시를 따른다.
