---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-helper-audit-20261026.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# 중첩 helper 감사 진행 기록

- 작업 브랜치: `audit/8232-px-regressions-20261025`
- 현재 head: `238de6de41a208648f4ade6b494fe433a81c1fce`
- 마지막 확인에서 tracked/untracked 작업장 변경은 없었다. 산출물은 ignored output에만 있다.
- 전수 감사·검사 보정은 미완료이며 PR을 만들거나 push하지 않는다. #8232 본문·조사 댓글 갱신은 승인 범위다.

## 이번에 확보한 증거

- phase3 main, 기존 Print 재사용, phase4 bitmap, phase5 textbox의 실제 프로세스는 종료됐다. 각 state의 완료 표시와 전쪽 TSV·명시적 종료 로그를 대조했다. 중복 실행하지 않는다.
- 기존 Print 재사용: unit7160 3쪽 Native99.69646/WASM99.68550, logos6465 13쪽 Native93.40074/WASM93.28630.
- phase4 bitmap3조건은 Native/fresh WASM 각3쪽 모두100%. 실제 Rust 복합 unit 검사는 별도이며 실행했다고 주장하지 않는다.
- phase5 textbox4조건은 새 MCP Print 성공, 각2쪽 Native/fresh WASM TSV 완료. hy001 p2는 HWP81.85601/81.85987, HWPX81.04687/81.04735. hy002 p2는 HWP99.03462/99.03339, HWPX98.77377/98.77579.
- 기존 hy001/hy002 Native unit 두 건은 각1PASS를 실제 실행했다. 같은 head Print tree의 hy001 두 로고 시작점 간격539.0px와 독립 PDF532.99334px 차이6.00666px는 기존525..550범위에서 검출되지 않는다. unit 내부 좌표는 출력되지 않았으므로 Print 좌표와 동일하다고 역추정하지 않는다.
- hy002 선언48190×4331HU, Print bbox약642.4×57.7px, 독립 PDF642.21465×57.69733px. 그림 ID/소속과 원본 치수 계약을 고정 크기 selector와 분리한다.
- 관련 자료는 `phase5-hy001-gap-counterexample.json`, `phase5-textbox-image-pdf-bounds.json`, `phase5-existing-test-results.json`과 각 TSV/review/overlay에 있다.

## 감사 수집기 보정

`test_helper_backstop.py`는 원본 후보 목록을 고정 입력으로 사용한다. 확장 원장을 입력으로 다시 쓰면 후보가 사라지는 오류를 피했다. 외부 cfg(test) 모듈, 중첩 테스트 helper, 같은 파일 호출의 숫자 단언을 기하 단어 유무와 무관하게 추가했다.

9,664함수/helper 그룹·2,103파일이며 추가6,558그룹은 대부분 논리 인덱스·형식·항목 수 후보도 포함한다. 위반 수가 아니다. method dispatch/동명/외부 호출 토큰의 실제 해결과 의미 심사는 아직 남아 있다. 파일 단위 태그 또는 국소 숫자 의미 분류를 전체 적합 판정으로 쓰지 않는다.

- 기존 source batch01~13:529그룹의 숫자 목적 분류. 독립 근거/중첩 호출 미검증 별도 유지.
- batch14:새 helper6개 분류, hy001/hy002는 독립 근거 및 출력 보정 대기.
- batch15~17:추가230그룹의 직접 수치 의미 분류. schema/차트/선택자/암호화/진단/IR/doclang 원본·직접 입력 계약으로 구분.
- 모든 레코드의 full_conformance_verified는 false이다. `build_audit_ledger.py`는 미심사 nested helper가 있는 부모를 별도 pending으로 표시한다.
- `bindata_fingerprint_preserved_on_roundtrip`는 이름과 달리 원본 fingerprint 존재만 확인하며 실제 roundtrip을 수행하지 않고 fixture가 없으면 return한다. 숫자 px 여부와 별도 의미 보완 후보로 기록했다.

## 다음 작업

1. 현재 git·프로세스를 다시 확인한 뒤 중단 지점에서 재개한다. 이미 완료된 캡처를 다시 실행하지 않는다.
2. `source-review-batch-18` 이후 추가 숫자 후보의 assertion 의미·원본·helper 소비를 확인한다. tests/E2E/fixture 동적 수집 및 보존 자료37건의 실제 호출 연결도 계속한다.
3. hy001의 공백 advance/선언 그림 폭·표/글상자 위치 차이, hy002의 선언 치수/원본 소속, source3257/tac-spacing 등 독립 좌표를 연결한다. 90% 미달을 먼저 검사 변경으로 숨기지 않는다.
4. 전수 판정 결과 이슈 업데이트 후 실제 부적합 검사와 관련 호출/등록을 issue_8232 이름으로 재작성하고 수정 전후·정상 대조군·필수 시각/회귀 게이트를 수행한다. 이름만 바꾸거나 유효한 보호를 삭제하지 않는다.

공개 갱신: https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6400402490

본문은 `issue-body-20261026-helper-audit.md`, 댓글은 `phase4-helper-audit-report.md`와 대조해 게시 내용을 재확인했다. 최신 댓글 영수증은 `phase4-comment-updated-receipt.json`이다.
