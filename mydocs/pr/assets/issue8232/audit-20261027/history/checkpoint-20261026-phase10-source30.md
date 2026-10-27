---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-20261026-phase10-source30.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 계속 작업 지점

- 목표 ACTIVE. PR은 나중에 진행하며 push/PR 생성 승인 없음. 이슈/댓글 조사 업데이트는 승인됨. 다른 task/agent 호출 금지.
- 작업장 `/Users/tsjang/rhwp`, branch `audit/8232-px-regressions-20261025`, HEAD `238de6de41a208648f4ade6b494fe433a81c1fce`. 마지막 확인 tracked/untracked dirty 없음. 제품/테스트 수정·이름 변경·커밋 없음.
- 직전 사용자 질문에 대한 설명만으로는 실무 진척이 없었음. 이번 계속 턴에서는 실제 프로세스와 상태를 확인하고 아래 원장·독립 출력·원격 이슈 기록을 갱신하여 진척을 만듦.
- 기존 검사를 무조건 보존하지 않음. 현행 지침과 독립 기대값으로 유효한 검출을 유지하고 잘못된 단언은 보정/대체/제거. 실제 보정 대상만 issue_8232 새 명명, wrapper 중복 없음. GitHub 본문은 이미 이 원칙으로 되어 있어 불필요한 본문 변경은 하지 않음.

## 지금 실행 중 — 중복 금지

- 기존 unified session **42050**, driver PID **51710**. `write_stdin`으로 같은 handle을 관찰한다.
- driver `output/issue8232-audit/run_phase10_endnote.py`, log `phase10-endnote-driver.log`, state `phase10-endnote-evidence/state.json`.
- 마지막 실제 ps: driver 실행 14분, 둘째 Native parent **93846**, 8개 export-png worker 중 **5258/5259** 약98% CPU, 각1분53초. 정상 실행 중. PID는 재검증 필요.
- first `endnote-2023-sep` 완료: 새 MCP job `353a7e1f-ca53-4a9c-bdd1-ae359b01e986`, 독립 PDF20쪽, Native/fresh WASM20쪽씩 TSV 완료. 전쪽 min **97.67133/97.50439**, below90 양쪽0. p19 **99.18789/99.16450**.
- p19 Native review + Native/WASM overlay 직접 확인: 문28/29 제목·풀이·그림·tail이 동일 단에 이어지고 하단 프레임 내. 미세 글리프/수식/선 차이 남음. 보편적인 +40/+18 계약 또는 최종 px 검사 자격의 증거로 확대하지 않음.
- second `endnote-2022-oct` MCP 성공, job `a1d67be9-6b5b-4d4c-89e7-6573c60745b6`, PDF18쪽. Native 전쪽 실행 중, WASM 아직 없음.
- 이후 `endnote-2022-nov`21쪽 예정. 총 캡처8·shared fcntl lock 유지, Cargo 실행 없음. timeout/출력 공백으로 재시작 금지.

## 이번 원장 변경

- source29 **80** 직접 의미 분류 완료. `record_source_batch29.py` 및 `source-semantics-batch-29.json` 추가. 원본 단위/소유/계수·합성 포함/분할·backend 색상 등, 실물 절대 핀 아님. full_conformance=false 유지.
- source30 **100** 완료. 95 직접 비절대 핀,5 독립 계약 pending: index47 밑줄0.17em/문자열22.72,75 nearest-row 잔여25px,80/81 저장 tail 제한2줄/예외,82 각주16px흐름/40px마진. 합성 입력이라는 이유로 생산 규칙을 승인하지 않음.
- 이전 source24–28과 합쳐 source24–30 **900** =855 비절대 직접 의미/45 독립 근거 pending. 전체 적합/실행 PASS 수 아님.
- `build_audit_ledger.py` 실행 후9666그룹2103파일, unreviewed6328/function_px_policy_adjudicated2500/semantic_notes133/local_meaning_nested89/function_evidence_pending404/file_context212. full_conformance 모두 미완료.
- source31 **120그룹 준비만 함. 아직 본문/단언 읽지 않았고 분류 없음.** 파일 `source-review-batch-31.json/.txt/-assertions.txt`,4132 full lines1070assertion lines. 재생성/중복 선택 금지, 다음에 이 묶음을 읽는다.
- `record_other_fixture_consumers.py`와 미확정37목록 다시 확인했으나 새 소비 폐쇄 판정은 하지 않음. 대량 출력은 truncation이므로 전체 읽었다고 주장하지 않음.

## 증거 카탈로그

- `build_paired_evidence_catalog.py`에 phase10 완료 pair만 추가되도록 변경. partial case 제외, 원본/PDF·producer Native/JS/WASM/canonical SHA 및 PDF 쪽수와 TSV의1..N 연속 페이지 대조.
- 현재 **84조건/각2353쪽/167개 unique input·PDF hash**, 전건 최종 px 자격=false 유지. 다음 case 완료 후 다시 실행할 수 있음.
- frozen Native/WASM 및 동일 코드 출처는 이전 checkpoint/producer 그대로이며 새 코드 통과로 부풀리지 않음.

## #8232 게시 완료

- 새 댓글 **6401858311**, https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6401858311
- 본문 `phase10-source-endnote-report.md`, GitHub fetch로 exact 일치 확인.
- receipt `phase10-published-receipt.json`, SHA `bc63266576acebb6b85702d6dc5ab6423bba6d8762199fa7ec630df229baec33`.
- 900분류/첫20쪽 양쪽 TSV/남은2문서 실행상태/유효한 보호와 잘못된 단언 구분을 기록. 조사/보정 완료 아님.

다음: 기존 phase10을 끝까지 관찰하고 Oct/Nov 실제 TSV·review/overlay를 검토하여 댓글·카탈로그를 갱신한다. source31 읽기와 남은 helper/동적/스크립트 소비 감사 병행. 전수 조사·독립 판정 이후 실제 필요한 공통 출력/검사 보정과 issue_8232 명명·등록·검출력/필수 게이트를 완료해야 목표 달성이다.
