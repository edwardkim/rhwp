# PR #2348 검토 — 다단·감추기·블록계산 히스토리 기록 (#2347, 연작 5)

- PR: https://github.com/edwardkim/rhwp/pull/2348 (lpaiu-cs) — Fixes #2347
- 충돌 0 (연작 1~4 merge 후 클린)

## 변경 본질

명령 팔레트/메뉴의 다단 설정·문단 감추기·표 블록 계산 미라우팅 —

1. page.ts: 동파일 page:break 패턴과 동형 snapshot 라우팅 (다단 ×3 + 감추기)
2. table.ts 블록 계산: **dry-run 검증 후 commit 만 snapshot** — 실패 시
   no-op 스냅샷 방지 설계. 원장 baseline 33→34 는 dry-run 검증 호출 증가로
   의식적 갱신 (트립와이어 규약 준수 — 사유가 diff 에 명시)
3. 소스 가드 동반. HF 구조 조작·다이얼로그 경로는 사유와 함께 범위 외 명시

## 로컬 재실증 (merged tree)

가드 7/7(원장 갱신 포함) · studio **345/345** · tsc 0 · e2e 24/0.

## 판단

**merge 권고.** 원장 baseline 갱신의 첫 사례가 규약대로(사유 명시·리뷰
경유) 수행됨 — 트립와이어 운용이 설계 의도대로 작동함을 보여주는 건.
