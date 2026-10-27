---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase33-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase33-goal-checkpoint.md` · 원문 SHA-256: `924d47f13d5837eca5d22fc8acdfd906a3372a317bfbf2b64d38ec5e33eeb63b`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 33차

## 목표와 권한

전수 조사·독립 판정 → #8232 반영 → 부적합 검사와 필요한 공통 구현 보정 → 실제 보정 파일·함수를 `issue_8232`로 명명하고 helper/module/suite manifest/생성/스크립트/E2E 호출 갱신 → 필수 실제 재검증까지 완료한다. 기존의 잘못된 판정은 보존할 의무가 없으며 독립 근거가 있는 검증 목표를 재구성한다. 정상 검사와 원본 단위를 일괄 삭제·개명하지 않고 옛 이름의 wrapper·중복 등록을 만들지 않는다.

PR은 나중이며 현재 push/PR 승인은 없다. #8232 업데이트와 필요한 MCP Print는 승인됐다. agent/task 메시지·예약·메모리 쓰기는 금지돼 있다. 전체 목표는 ACTIVE다. 직전 turn과 이번 turn은 각각 새 직접 판정·실행·독립 근거·이슈 반영이라는 실제 진전이 있었으며 전체 완료나 차단 상태가 아니다.

## 현재 작업장

R=`/Users/tsjang/rhwp`, branch=`audit/8232-px-regressions-20261025`, HEAD/base=`238de6de41a208648f4ade6b494fe433a81c1fce`. Git clean. 제품·검사 수정 0건, 개명 0건. 모든 임시 자료는 ignored `output/issue8232-audit/`에 있다. jobs 8/threads 8/캡처 총 8/Cargo 순차와 원본·PDF·공유 target·다른 작업장 보존을 유지한다. 이번은 Node 단위 실행과 mock 진단이며 실제 WASM binary·브라우저·OS IME·Native build·시각 캡처·MCP Print는 없다. 이번 실행은 모두 terminal exit 0으로 끝났다.

## 조사와 실제 실행

ai-write-tools.test.ts, overwrite-mode.test.ts, support/overwrite-mode.runner.mjs를 전체 읽었다. 원래 정의 176/1/137개를 동일 줄의 원장 키로 합쳐 174/1/130그룹, 총 305개를 중복 없이 추가했다. 실제 Node root는 AI 45개와 부모 1개, 합계 46개다. 러너의 scenario callback 정의 31개와 기타 mock/helper·중첩 callback 228개는 별도다. 함수별 의미·단언·같은 줄의 정의 목록은 `record_studio_helper_batch39.py`와 `non-numeric-helper-semantics-39.json`을 따른다.

`run_phase33_studio_tests.py`는 Node v24.15.0에서 기존 검사 46 PASS / 0 FAIL / 0 skipped를 확인했다. TAP 892.799167ms, wrapper 0.987147375초다. `phase33-studio-tests-producer.json/log`가 정본이다. loader는 nextLoad의 결과를 바꾸지 않고 파일/hash만 기록한다. 관찰 103건과 모든 실제 source의 전후 SHA가 같으며, 부모가 실행한 같은 자식 PID에서 runner와 production input-handler-text/input-handler/command/history를 확인했다.

부모는 실제 spawn 오류·자식 exit 0·성공 marker도 검사한다. source의 literal 호출 30개와 deactivate/dispose 반복 2개는 32호출 구조다. 각 시나리오별 runtime event를 32개 따로 수집한 것은 아니며 Node 검사 수에 더하지 않는다. 러너는 @wasm 엔트리를 data-URL 이름 stub으로 대체하고 문단 모델·DOM 대역을 쓴다. production 입력 handler·명령·history는 실제 실행하지만 실제 WASM core·브라우저와 모형의 완전한 동등성을 뜻하지 않는다.

## 수치 의미와 검출 범위

AI fakeDoc은 BMP 중심 length/slice와 call 기록·ok 반환을 많이 사용한다. cell 2 요청 등은 실제 cell 문자열을 편집하지 않는다. 그림의 10/5바이트 배열도 실제 PNG/JPEG decode가 아니다. replace-all은 제안/재검색 개수·호출을 보호하지만 mock 자체는 전체 문자열을 치환하지 않는다. 이런 단위 결과를 실제 HWP 저장·전체 render 수용으로 확대하지 않는다.

도형 좌표 9,283HU는 작성 본문 하단 120 CSS px×75HU+1mm의 상대 계산이다. `probe_phase33_ai_units.mjs`는 기존 fakeDoc body를 추출하고 실제 prepareWrite를 호출했다. 본문 40px 이동은 12,283HU, cell 조각만 1,000px 이동은 9,283HU 유지, 본문 0.5px 이동은 반올림해 9,321HU이고 다른 도형 속성은 같다. 실물 HWP에서 계산한 페이지 원점 핀이 아니다.

`inspect_phase33_independent_units.py`는 독립 유리수 7200/25.4, 7200/72, 7200/96으로 mm/pt/CSS 단위를 대조했다. 1/8/12/25/30/40/50/60/182/257mm의 HU 반올림은 각각 283/2268/3402/7087/8504/11339/14173/17008/51591/72850이다. 16.5pt→1,650HU와 12pt→1,200HU도 확인했다. 여백·들여쓰기 pt×200은 명시된 host API 이중 저장 전달 계약이며, 이 진단이 전체 HWP 파서 단위나 독립 Print 좌표를 검증한 것은 아니다.

😀는 scalar 1/UTF-16 2, ❤️는 2/2, 👍🏽는 2/4, e+결합 악센트는 2/2다. Python 표는 source 단위만 확인하며 모든 문자소 규칙을 구현한 것은 아니다. 기존 러너의 Unicode/field/link 범위, logical offset, 현재 호출 undo/redo·snapshot, fragment 해제, fake clock의 300/301ms 경계는 의미·시간 계약이며 실물 페이지 px 핀이 아니다. 현재의 단위·mock 결과를 실물 문서 90/98% 자격이나 font/glyph 시각 수용으로 인용하지 않는다.

## 원장·외부 상태·다음

직접 판정 6,993개/743소스 파일, 원장 11,265그룹/2,117파일이다. 미분류 3,942개와 메모만 있는 330개를 합친 분류 미확정은 4,272개다. 독립 근거 pending 508개는 직접 판정 기록 안에 있어 단순 합산하지 않는다. `validate_phase33_ledger.py`가 현재 source SHA·전체 직접 기록 연결·TAP 제목 46개·실제 자식 source 연결을 확인했다. 전체 감사·호출 경로 검증·보정·개명은 미완료다.

#8232 본문에 이번 결과 5개 bullet을 추가했고 API 재조회에서 말미 개행 포함 byte 동일을 확인했다. 상태는 OPEN이며 updated_at·본문 SHA는 `phase33-remote-verification.json`을 따른다. 완료 checkbox·close·PR 생성은 없다.

phase32의 profile guard 검출 누락과 원 PR #2297, phase30의 암호 HWP3 vpos 4개·배경 차이·Native 3쪽 raw 39.00717%와 전 24쪽/WASM 자격 미완료는 계속 남는다. 이전 checkpoint/독립 JSON을 따르고 과거 결과를 새 source 수용으로 재인용하지 않는다. 남은 미분류·메모·독립 근거와 source cfg/전체 tests/Studio E2E·동적/생성/공용 경로를 계속 판정한다. 다음 후보는 현재 원장의 ruler-resize.cases.mjs, nested_tables.rs, issue_4764_pdf_raster_fidelity.rs 등이다. 끝난 이번 46개와 이전 65/41/37/60개를 자동 재시작하지 않는다. 실제 부적합 검사 보정·새 명명·모든 호출/등록 갱신·필수 재검증까지 끝나야 전체 목표가 완료된다.
