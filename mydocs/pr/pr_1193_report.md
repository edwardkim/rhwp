# PR #1193 처리 보고서 — Fix mac window resizing

- **작성일**: 2026-06-01
- **PR**: #1193 → **MERGED** (devel, 머지커밋 `8a2e60ca`)
- **컨트리뷰터**: @chkwon (Changhyun Kwon) — **rhwp 첫 기여자** 🎉
- **연결 이슈**: #1191 → **CLOSED** (`closes #1191`, cross-repo `--no-ff` 라 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

macOS 에서 창을 넓혔다 줄이면 문서가 우측으로 치우쳐 가운데 정렬이 깨지는 #1191 회귀.
`#editor-area` 의 `grid-template-columns/rows: 20px 1fr` 에서 `1fr`(=`minmax(auto,1fr)`)의
콘텐츠 기반 최소 크기가 한 방향 래칫으로 작동한 것이 원인. `minmax(0, 1fr)` 로 트랙 최소를
0 으로 고정해 축소 추종 정상화 — CSS grid 의 정석 해결책. 위험 매우 낮음.

## 변경 요약 (1 파일, +5 / −2)

```diff
-  grid-template-columns: 20px 1fr;
-  grid-template-rows: 20px 1fr;
+  grid-template-columns: 20px minmax(0, 1fr);
+  grid-template-rows: 20px minmax(0, 1fr);
```
(`rhwp-studio/src/styles/editor.css`, 의도 설명 주석 동반)

## 검증 결과

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| studio build | `npm --prefix rhwp-studio run build` | ✅ (tsc + vite, 115 modules) |
| WASM | `docker compose ... wasm` | ✅ pkg 빌드 (`pkg/rhwp_bg.wasm` 갱신) |
| **시각 판정** | 작업지시자 직접 (창 리사이즈 가운데 정렬) | ✅ **통과** |
| merge 검증 | PR head 조상 확인 | ✅ PR_MERGED_IN=YES |

> CSS only 라 cargo 테스트 무관. BLOCKED 는 required "Build & Test" 체크 부재(CI 미트리거) 때문, 결함 아님.

## 처리 절차

1. PR 정보 확인 — #1193 은 "Fix mac window resizing"(당초 #1183 셀 이미지와 무관). MERGEABLE/BLOCKED,
   merge-base = 현재 devel(BEHIND 아님), 1파일 CSS.
2. 컨트리뷰터 사이클 점검 — @chkwon PR 누적 1건 → **첫 기여자 확정**.
3. CSS diff 검토 + 로컬 `pr1193-verify` 머지 시뮬레이션(studio build) + WASM 빌드 → `pr_1193_review.md` → 승인.
4. **작업지시자 시각 판정 통과**.
5. 메인테이너 로컬 `--no-ff` 머지(`a593a7e0..8a2e60ca`) + push. PR head 조상 확인.
6. 이슈 #1191 수동 클로즈 + **첫 기여자 환영 코멘트** + 보고서.

## 비고

- cross-repo `--no-ff` 머지라 GitHub 자동 'Merged'/이슈 자동 클로즈 미작동 → 수동 처리(#1178 등 동일 패턴).
- **첫 기여자 환영**: feedback_pr_comment_tone 의 "반복 컨트리뷰터 매번 같은 인사 부적절" 취지와 별개로,
  첫 기여자에 대한 진심 어린 환영은 적절 — 따뜻한 톤으로 코멘트.
