# PR #2080 검토 — 프론트 웹 리팩터링 계획 v2 문서 (postmelee)

- 이슈: #2023 / docs-only (mydocs 9파일) / 작성일: 2026-07-09
- 성격: 우리 리뷰(#2023 코멘트, 보완 7건) + jangster77 리뷰의 **반영본(v2) 재리뷰**.

## 보완 7건 반영 확인 (v2 §11 응답 매핑 + 본문 대조)

| 보완 | 반영 |
|---|---|
| 1 도구 확정 | §3.1 eslint+sonarjs/cognitive-complexity 고정, Phase 0 advisory→fail 승격 유예 ✓ |
| 2 산식·모집단·과도기 | §3.3 포함/제외군(generated/vendored/e2e), §3.5 과도기 허용+예외 심사제 ✓ |
| 3 행동 고정 게이트 | §7 smoke manifest 표면별 분리, **WASM JSON schema = frontend 소비자 ownership + 상호 참조** ✓ |
| 4 stage-gate 준용 | §1 명시 ✓ |
| 5 금지 목록 3건 | §5 확정 승격(3-browser sync=§6, 혼합 금지, publicDir false/inline script) ✓ |
| 6 실측 수치 | v1 수치 heuristic 격하 + Phase 0 공식 재측정 (선반영 대신 지위 정리 — 수용) |
| 7 실행 분리 원칙 | §10 하위 이슈 정책 — v2 승인 후 Phase 0/A 만 선분리, 해체는 재평가 후 ✓ |

채점 앵커 부록은 Phase 0 로 이연(§4) — 수용 가능. jangster77 3건도 반영 확인.

## 판단

**approve + merge.** docs-only 라 코드 게이트 불요, orders 파일 1줄 접촉만 병합 시 확인.
merge 후 #2023 에 v2 승인 코멘트 — Phase 0/A 하위 이슈 분리 진행 가능 통보.
