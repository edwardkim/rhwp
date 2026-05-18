# PR #971 최종 보고 — HWP3 margin_bottom 원본값 보존

## 1. 결정

**merge 권고** — 설계·구현·프로젝트 규칙 적절, 전 검증 통과.

| 항목 | 값 |
|------|-----|
| 번호 | #971 |
| 제목 | fix: HWP3 margin_bottom 원본값 보존 — 쪽 테두리/페이지 번호 위치 정상화 |
| 작성자 | oksure (Hyunwoo Park) — 외부 컨트리뷰터 |
| base ← head | `devel` ← `contrib/fix-hwp3-margin-bottom` |
| 연결 이슈 | closes #951 |

## 2. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| 시각 판정 (작업지시자, 한컴 정답지 대조) | ✅ **통과** |
| CI (Build & Test / CodeQL) | ✅ SUCCESS |
| mergeable | MERGEABLE (BEHIND — 충돌 아님, merge 시 자동 갱신) |

## 3. 평가 요약

- `margin_bottom` 직접 변조(`saturating_sub(1600)`)를 제거하고
  `PageDef.pagination_bottom_tolerance` 신규 필드로 책임 분리.
- `1600u32.min(margin_bottom*4)` clamp가 기존 `saturating_sub` 상한과 동치 —
  회귀 위험 없이 부작용(쪽 테두리/페이지번호 위치 오류) 차단.
- tolerance를 `available_body_height()`에만 반영, `body_area`는 불변.
- CLAUDE.md "HWP3 전용 로직은 `src/parser/hwp3/` 안에서만" 규칙 준수
  (공통 모듈엔 포맷 중립 필드만 추가).
- 회귀 테스트 y좌표 갱신(1061.4→1050.8)은 보정 결과와 일관.

상세 검토: `pr_971_review.md`

## 4. 후속

- merge 후 `mydocs/pr/pr_971_review.md`, `pr_971_report.md` → `pr/archives/` 이동
- 이슈 #951 close (PR merge로 자동 — `closes #951`)
- merge 후 `git branch --contains` 로 devel 반영 검증
