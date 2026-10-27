---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase72-chart-data-units/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase72-chart-data-units/goal-checkpoint.md` · 원문 SHA-256: `58b80e54817573a1ecb50841bf28c9a09628a47322a39ed39a5716f0a4e1e176`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 72차 체크포인트

목표는 ACTIVE다. 전수 조사·독립 판정 완료 뒤 #8232 갱신, 부적합한 검사/필요한 구현 보정, 실제 보정한 파일·함수의 issue_8232 개명과 helper/module/suite/manifest/생성/스크립트/E2E 호출 갱신이 남았다. 제품/검사 수정 0, 개명 0이다. 사용자 지시대로 PR은 나중이며 push/PR 승인은 없다. 새 agent·다른 task 메시지·예약·메모리 쓰기를 하지 않는다.

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, head=01ced3d42724285c8f9765a453393ab6af326617. 유일한 기존 dirty는 rhwp-studio/public/rhwp.js 97줄 추가이며 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존했다. 공유 target·원본·PDF·다른 작업장은 보존한다. 요청 없이 fetch/rebase하지 않는다.

이번 71–72차는 Cargo/prepare/새 빌드가 모두 0이다. 기존 현재 head 완료 metadata와 실행 바이너리 hash가 일치한 대상만 libtest --exact --test-threads 8로 실행했다. Native CLI hash는 9d496496e8561ff04c7c64b7b7171581d29fa6babe1dcfcbe38467574250ecf5 그대로다. 반복 빌드 지적은 유지한다. 정적 의미 조사와 기존 바이너리 재사용을 우선하고 실제 코드 변경/꼭 필요한 미빌드 target이 생길 때만 묶어서 빌드한다.

최신 원장은 phase72-chart-data-units/audit-ledger.json이다. 누적 12502그룹/2139파일, 직접9014함수/Rust6173검사/802파일이다. 의미판정8514/file-scope8/근거pending500/미분류3202+의미메모97+문맥181=3480이다. 신규 Rust95/삭제3(원장4그룹)/새JS callback1(schema2)·관련 helper의 사용자 제외를 그대로 유지한다. 이 두 차수와 교집합은0이다.

71차: tests/mcp_session_changed_pages_contract.rs 492줄, mcp_session_view_contract.rs 263줄, provenance_contract.rs 2858줄의 전체3613줄을 읽고68함수를 신규 판정했다(22검사/46helper, 초기 숫자 검색 밖50그룹 추가). 기존 바이너리22PASS/0FAIL, 21.582초, 건너뜀 메시지0, 새 px 보정 대상0. 논리 쪽 주소·파일 bytes·문자/명령/봉투 수·schema version·종료 코드는 화면 px 핀이 아니다. 동일 엔진 재독/봉투 형식/출처 root coverage를 독립 한컴 전쪽 시각 정답으로 확대하지 않는다. 원 PR3788→3816,3612→3647,3804→3816; 보정94cdd의u32 경계와adb805의가드4중화(#3912),d818경로반향(#7935),조건부PDF import(#7913)를 연결했다. 원 status/primary PR snapshots/함수별 근거/producer를 보존했다.

71차 측정 한계: replace/set-cell root는 양쪽 changedPages null도 같다고 볼 수 있고, renderable은 출력 bytes 양수만 확인하므로 내용/새 쪽 포함 전체성을 입증하지 않는다. doc_info/root는 envelope형태뿐 별도 무상태 전체 비교가 아니다. tools description root는 실제 봉투 재호출까지 하지 않는다. provenance oracle은 실제 export-text/tables로 만든 문자열 표본이라 완전한 독립 원본 parser가 아니며 root 단위 선언 비교는 모든 nested 새 경로를 잡지 못한다. 면제 root는 fields 존재/false를 보지만 빈 배열까지 직접 대조하지 않는다. 이런 원 범위는 기록하되 새 px 결함이나 범위 밖 보안 구현으로 자동 확장하지 않는다.

72차: tests/issue_4055_b1_chart_edit_probe.rs842줄, issue_4099_hwpx_chart_to_hwp.rs695줄, cases/csv_to_chart_structure_contract.rs433줄, support/issue_4055_chart_probe.rs407줄 전체2377줄을 읽었다. 신규50함수/27검사 + 이미 직접 판정한support21개 재독이다. helper21개는 hash를 확인하고 신규 집계하지 않았다. 새 원장27그룹, 기존unreviewed15/context8개를 의미 판정했다. 기존 CSV Native7PASS/0FAIL(0.125초). 미빌드Rust18개와ignore 생성기2개는 미실행이다. 기존 output/issue_4055_b1_spike와output/issue_4099를 덮어쓰는 생성기는 실행하지 않았다. 추가 빌드0·새 px 보정 대상0.

72차 독립 원본: olefile로 CFB를 읽고 raw deflate Section0의 레코드 헤더/struct를 직접 읽어 OLE tag84 offset617/level3/30바이트, extent_x/y7200HU, bin_data_id1을 확인했다. zipfile/ElementTree는 원 HWPX의orgSz7200×7200/extent7200×7200, chart id1117817146/ole instid0, 첫값4.3/3계열×4값, ZIP/중첩OOXML 사본bytes동일, rootCLSID4C3DA137-DC90-47B9-9BED-59DAE352A280을 확인했다. R samples/chart/세로막대형/묶은세로막대형.hwp SHA12e8b4034cb236cc2fc20b777f90628825111e2ee8d8ec81e97f3fa60987d644, .hwpx SHAe8ffa2ef81e630b5ac7e0a230c101924b96bd4d61be958cf608161bfe9a7365a. 결과 independent-source-units.json. 원본 속성/데이터/바이트 크기와 실물 배치의 절대 px 핀을 구분한다.

원 해결 계보:4055 original4061, CLSID4144→4171 및97c8d프로브 보정;4099 original4499→4571+bc401snapshot/후속1af7lowering;5652 original5968→6001+6037 original6052→6072이다. primaryPR snapshot, 원 보고서와 보정 이력은 phase72에 연결했다. 옛 수용 기준5 bin_count>1 검사는defe8/7861에서 제거된 현재 상태이므로 과거 보고서만 보고 복원하지 않는다. 과거 PDF/144DPI hash를 현재 head의 Native/fresh WASM 전쪽 통과로 재인용하지 않는다.

#8232는72차 최신 count/정상 요약만 갱신했다. 원 실패 findings·외부 본문·8266·완료 조건은 불변이며 이전status 역복구/EOF exact readback을 확인했다. publication-receipt.json updated_at2026-10-26T13:05:26Z, UTF16길이65529로 여유7자다. 다음 게시 전에 fresh GET과 길이부터 확인하고 정상 요약만 줄인다. 이전 본문 전문/상태는각 phase에 보존했다.

이번 turn은 신규118함수 판정·기존29검사PASS(미실행18/ignore2 구분)·독립CFB/ZIP원단위확인·원장/이슈갱신이라는 progress다. 제품 소스/회귀 수정과개명0, 전수조사미완료다. pending500은 그대로 남았다. phase68/footer의6pending과phase63/62/61의실제핀문제·57/58/60의관측한가드한계는이전체크포인트와이슈본문을보존하며완료로바꾸지않는다. 다음은남은3480그룹함수의단위·호출경로·독립근거를계속조사한다. 조사가끝나면요청된보정·issue_8232개명·참조갱신·필요검증을완료한다. complete/paused/blocked로바꾸지않는다.
