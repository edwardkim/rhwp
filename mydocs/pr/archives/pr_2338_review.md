# PR #2338 검토 — HF/각주 편집 히스토리 기록 (#2337, 이관 연작 1)

- PR: https://github.com/edwardkim/rhwp/pull/2338 (lpaiu-cs) — Closes #2337
- #2327 이관 연작의 최중량 표본 (HF/FN 8 뮤테이터 ×16 호출처)

## 변경 본질

HF/FN 편집이 히스토리 완전 미기록 → (A) 본문 스냅샷 undo 가 그 사이 HF/FN
편집을 무언 파괴(복구 불가) (B) HF/FN 자체 undo 부재. 정정:

1. **역연산 커맨드 8종** — 본문 텍스트 커맨드 미러(스냅샷 비용 0, 순수 wasm
   접근), `editContext()` 로 HF/FN 커서 컨텍스트 노출
2. **라우팅** — kind:'record' 사후 기록 (본문 IME 패턴 미러), IME 게이트의
   HF/FN 명시 배제 제거
3. **undo/redo 후 컨텍스트 복원** — HF/FN 모드 (재)진입 + 오프셋 복원, 본문
   moveTo 스킵. peekUndoTop/peekRedoTop 신설
4. **Rust 최소 변경** — HF/FN 삭제가 deletedText 반환 (char 슬라이스로
   delete_text_at 클램핑과 동일 범위 — UTF-16 조인 모호성 회피 논거 타당)
5. 소스 가드 테스트 (역연산 정합·게이트 미배제·라우팅 개수 핀)

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| Rust | header_footer 16 / footnote 12, **전체 스위트 실패 0**, fmt/clippy 0 |
| studio | 신규 가드 17/17, 단위 **334/334**, tsc 0 |
| WASM 재빌드 + e2e | undo-contracts 24/0 · text-flow 0 FAIL |

브라우저 HF/FN undo/redo 실측(모드 유지·오프셋·0 에러)은 컨트리뷰터 제공 —
구조 검토와 게이트로 정합 확인.

## 판단

**merge 권고.** snapshotResourceCount=0 으로 #2332 예산과 안전 공존, 무언
데이터 손실(A)이라는 최중량 위험 제거. 연작 나머지(#2341~#2350)와 독립.
