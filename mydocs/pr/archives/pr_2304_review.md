# PR #2304 검토 — undo/redo 후 stale 개체 선택 ref 정리 (lpaiu-cs, #2303)

- 검토일: 2026-07-16 / 4파일 +275/−0 (input-handler +21 + 테스트 2종 +
  scripts) / CI 12 green / DIRTY (package.json scripts 1줄 충돌 —
  maintainerCanModify=true, 직접 해소 예정)
- 작성자: undo/스냅샷 라우팅 계열 재기여자 (merged 8건 — #2028/#2039/#2040
  /#2074/#2076/#2078/#2302). 본 PR 은 어제 merge 된 #2302(undo e2e 계약)의
  직접 연장.

## 구조 검토

- 근인 분석 정확: undo/redo 가 위치 기반 선택 ref({sec,ppi,ci})의 "실제
  컨트롤과 일치" 불변식을 깨는 유일한 잔여 경로 (Delete 경로는 기존 정리,
  키보드 Ctrl+Z 는 차단, **메뉴 undo 만 진입로** — 비대칭 지적 타당).
- 정정은 기존 정리 관례와 동일 3단(exit + 렌더러 clear + 이벤트 발행),
  개체·표 양쪽, 비선택 시 no-op — 텍스트 undo 무영향. #2230 에서 확립한
  selectedPictureRef 수명주기와 정합.
- 범위 외 기록(방어층 try/catch 는 증상 은폐라 비채택) — 판단 타당.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| studio npm ci + tsc + `node --test` | 클린 / **286/0** (기존 283 무회귀) |
| 신규 e2e `undo-object-selection` (headless) | 전 항목 PASS (undo/redo 대칭 + 표 케이스 + 개체 속성 no-op) |
| 컨트리뷰터 FAILED 실증 | stash-revert 로 계약 assert 7건 FAIL 재현 기록 — 관례 준수 |

## 판단

**approve → merge 수용 권고.** DIRTY 는 scripts 1줄 충돌 — maintainer
edit 로 직접 해소 후 CI green 확인, merge.

---

## 처리 완결 (2026-07-16)

- 충돌 해소: package.json scripts 양쪽 보존(e2e:undo + e2e:undo-object-selection)
  — maintainer edit 로 fork push (LFS locksverify 우회 필요). 해소 후 298/0.
- CI green → CLEAN merge (12:04 UTC). 감사 코멘트(undo 연작 인정) 게시.
