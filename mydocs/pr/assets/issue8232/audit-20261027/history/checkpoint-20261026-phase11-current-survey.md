---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-20261026-phase11-current-survey.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 현재 조사 연결 기록

- 이 턴은 실제 조사·시각 검증·기존 검사 실행·이슈 갱신을 완료한 progress다. 전체 목표는 ACTIVE이며 완료/보류/차단으로 바꾸지 않았다.
- 작업장 /Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD 238de6de41a208648f4ade6b494fe433a81c1fce. tracked/untracked Git 변경 없음. 모든 이번 산출물은 ignored output/issue8232-audit에 있다.
- source 묶음33(160)·34(25)·35(62)와 integration09(172) = 419개 직접 의미를 기록했다. 직접 숫자가 실물 페이지 절대 px 핀이 아닌 항목400, 독립 근거 pending19이다. 별도 비수치 helper03 6개(그중 독립 근거 pending3)를 연결했다. 개별 목적 분류를 전체 호출 경로 적합/PASS로 세지 않는다.
- source34는 표 split/merge 내용·원본 행/열·WMF byte/overflow다. source35는 rhwp-desk/llm verifier/PDF-IR의 protocol·통계·원본 단위/병합 관계다.
- integration09는 issue7911 PDF 자원113그룹 + hwpx_to_hwp_adapter59그룹이다. 18그룹은 독립 기준 pending이다. 쪽수 oracle helper는 현재 rhwp로 한컴 저장 HWP를 다시 조판한 값이고 실패 시 rhwp HWPX page_count로 fallback한다. 9쪽 probe도 rhwp 재로드 기준으로 명시되어 있다. source LineSeg HU와 실제 px 위치 핀을 구분했다.
- HWPX 기존 ±2px 검사 phase11은 전20쪽 Native/fresh WASM 양쪽 완료·각 exit0·TSV 페이지1..20이다. 전쪽 최저 Native97.71868/WASM97.55401, below90 양쪽0. 해당14쪽 Native99.38688/WASM99.31356.
- 독립 engine2024 Print20쪽/PDF SHA f63ea5e5c15c6bcf1208a29fe5c2845ad5413d1e390ac8d27b93311ce3a02371, job837f88e3-90c1-48c8-ab79-07c74847830b. PDFium 문자 원점 문20(34.0799993,415.2000326)/문22(402.5599772,239.0400391)이 기존 기대값과 맞는다.
- Native/WASM 14쪽 review+overlay 및 양쪽 두 제목/후속 본문/첫 수식 3배 확대를 직접 읽었다. 위치·소유/순서는 맞지만 WASM 두께·수식 모양 차이가 남는다. 기존 위치 검사 유지 근거이며 전체 글꼴 모양 수용이 아니다. phase11-title-detail/producer.json에 원본 PNG hash와 표시용 crop/배율을 기록했다. canonical raw/PNG를 바꾸지 않았다.
- 이미 컴파일한 실제 기존 HWPX caller만 exact 실행: 1PASS/0FAIL/1filtered/167.55초/exit0. session18048 종료, Native rlib/source drift 없음. suite전체/CI PASS가 아니다. phase11-existing-hwpx-test.log의 다른 쪽 LAYOUT_OVERFLOW는 보존하고 실제 paint 대응 확인을 남겼다.
- phase11 driver PID75024/session42090와 capture31594, exact test36030은 실제 종료를 확인했다. 새 turn에서 ps/로그부터 재확인하고 동일 실행을 재시작하지 않는다. Cargo 작업은 시작하지 않았다.
- build_paired_evidence_catalog.py에 terminal phase11 연결을 추가했다. 현재87조건/각2412쪽/173고유 input+PDF. 기존 무변경 코드 capture와 현재 생산 증거를 구분하고 final_px_test_eligibility는 여전히 일괄 true가 아니다. 개별 자격은 phase10/phase11-existing-px-page-qualification.json이다.
- 이슈 본문은 원래부터 잘못된 기대값을 수정/대체/제거할 수 있게 되어 있었다. 새 상태 요약으로 갱신했고 댓글6401858311에도 이번 조사/phase11 결과를 추가했다. live fetch로 본문/댓글 exact equality를 확인했다. 영수증 phase11-published-receipt.json, 로컬 본문 issue-body-20261026-current-survey.md. 이전 phase10 receipt는 이전 snapshot이다.
- 제품·테스트 수정0, 이름 변경0, 커밋/PR/push0. 아직 전수 조사 완료가 아니다. 정상 검사 무조건 개명하거나 유효하지 않은 옛 결함 검출 동작을 무조건 보존하지 않는다. 실제 보정 대상만 issue_8232로 바꾸고 wrapper 중복을 만들지 않는다.

## 남은 작업

미분류/중첩·외부 helper/독립 근거/동적 수집·조건부 등록을 마무리한다. 원 목적·실제 통합·독립 근거에서 부적합을 판정한 검사와 필요한 출력 보정을 실제 수행하고 issue_8232 명명/등록을 갱신하며 필수 재검증을 한다. 전수 조사만으로 끝내거나 PR을 만들지 않는다. Hyperlink/상대 글자 크기/Recipe 잘못된 기대값, 미주 cursor 단위 상수, 실제 px 핀 등 기존 pending은 그대로 남아 있다. 아카이브 fixture37의 동적 소비 연결도 아직 false이며 부정 검색만으로 완료 처리하지 않는다.

현재 원장 snapshot:
```json
{
  "head": "238de6de41a208648f4ade6b494fe433a81c1fce",
  "function_groups": 9676,
  "files": 2103,
  "numeric_search_groups": 2707,
  "render_producer_groups_outside_numeric_search": 399,
  "review_state": {
    "unreviewed": 5616,
    "function_px_policy_adjudicated": 3186,
    "semantic_notes_present_requires_adjudication": 133,
    "local_meaning_reviewed_nested_helpers_pending": 99,
    "function_evidence_pending": 430,
    "file_context_only_requires_function_review": 212
  },
  "policy_adjudication_finished": false,
  "product_or_test_changes": false,
  "renames_completed": 0,
  "note": "This ledger unifies overlapping discoveries. A file note never marks all functions in that file compliant. Candidate inventories and semantic notes are not defect counts or completed fixes."
}
```
기록시각: 2026-10-25T19:40:01.654308+00:00
