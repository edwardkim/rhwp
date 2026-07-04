# PR #1533 처리 보고서 — studio HWPX 직접 저장 활성화 (#196 베타 게이트 해제)

- PR: https://github.com/edwardkim/rhwp/pull/1533
- 제목: `Task #1532: studio HWPX 직접 저장 활성화 (#196 베타 게이트 해제)`
- 작성자: planet6897 (collaborator)
- 연결: Closes #1532
- base ← head: `devel` ← `planet6897:pr-task-studio-save`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge (베타 해제 정책 작업지시자 승인).** HWPX 직렬화 충실도 확보로 studio 의 HWPX
출처 직접 저장 비활성(#196 베타)을 해제한다. 코드/전제조건 충족 + 검증 통과 + 충돌 0건.

## 2. 변경 범위 (studio TS 전용)

| 파일 | 내용 |
|---|---|
| `rhwp-studio/src/command/commands/file.ts` | 포맷 인식 저장(HWPX→exportHwpx, 그 외 exportHwp), 베타 alert+unsupported 분기 제거, saveFileNameFor/saveBaseNameFor 헬퍼, canSave=true |
| `rhwp-studio/src/hwpctl/index.ts` | SaveAs 의 #196 HWPX 차단 게이트 제거 |
| `rhwp-studio/e2e/hwpx-direct-save.test.mjs` | alert 0 + blob type(application/hwp+zip) + PK 매직 + reopen 검증 |

## 3. 베타 해제 전제 조건 — 충족 확인

#196 의 본질은 "한컴 호환 미보장 고지" 게이트이고, 코드 주석상 해제 조건은 `#197 완전 변환기
완료 시까지`였다. 확인 결과:

- **#197 (HWPX→HWP 완전 변환기) CLOSED** — 해제 조건 달성.
- HWPX serializer 한계 해소: `run 평탄화`(#1378), `셀·글상자 컨트롤 미출력`(#1379) 메모리/매뉴얼
  취소선 처리 + 직전 머지 #1597 에서 직렬화 결함 다수(ClickHere/holdAnchorAndSO/도형 지오메트리/
  Ruby) 해소.
- `hwpx_roundtrip_baseline` 4/4 + `visual_roundtrip_baseline` 3/3 통과(구조·시각 정합).

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 |
| studio `tsc` (수정 파일) | 에러 0 (canvaskit-wasm 미설치는 무관·기존) |
| `npm test` | 147/147 |
| e2e `hwpx-direct-save.test.mjs` 구문 | OK |
| `hwpx_roundtrip_baseline` / `visual_roundtrip_baseline` | 4/4 / 3/3 |

## 5. 정책 판단

"한컴 호환 미보장 고지 제거 + HWPX 직접 저장 허용"은 사용자 영향 정책 결정이다. 전제조건
(#197 완료, 직렬화 충실도 확보)이 충족되어 작업지시자가 베타 해제를 승인했다. HWPX 충실도에
대한 시각 판정 권위는 작업지시자 환경에 둔다(`feedback_visual_judgment_authority`).

> autosave 복구본은 종전대로 HWP(별도 트랙). 본 PR 은 사용자 직접 저장 경로 활성화에 한정.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1533_review.md`
