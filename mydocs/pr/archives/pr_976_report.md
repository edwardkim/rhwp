# PR #976 최종 보고 — 복학원서 JPEG 워터마크 배경 사각형 제거 및 톤 보정

## 1. 결정

**merge (옵션 A 수용)** — 구조적 개선 + 전 검증 통과 + 시각 판정 통과.

| 항목 | 값 |
|------|-----|
| 번호 | #976 |
| 제목 | Task #938: 복학원서 JPEG 워터마크 배경 사각형 제거 및 톤 보정 |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 (25번째 PR) |
| base ← head | `devel` ← `codex/issue-938-watermark` |
| 연결 이슈 | Refs #938 |
| 처리 방식 | cherry-pick (단일 커밋 `2a48ed37` → 최신 devel) |

## 2. 검증 결과

cherry-pick `7a1f22f6` (최신 `local/devel` = PR #971 + Task #987 +
fmt fix 위), 충돌 없음.

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| 타깃 테스트 (issue_938/514/516) | ✅ 2 + 3 + 8 passed |
| 전체 `cargo test` | ✅ 1484 passed, 0 failed |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 |
| WASM 빌드 (Docker) | ✅ 성공, jpeg feature 반영 (+201KB) |
| 워터마크 emit | ✅ image/png, opacity="0.17" 미적용 |
| **시각 판정 (작업지시자)** | ✅ **통과** |

## 3. 평가 요약

### 강점
- SVG / WebCanvas / studio overlay 3개 렌더 경로를 단일 baked PNG 로
  통일 — 경로별 톤 불일치 구조적 해소.
- WASM JPEG decode 불가 문제 해소 (`image` crate `jpeg` feature).
- baked 시 중복 필터(effect/bc/opacity 0.17) 명시 생략.
- 워터마크 미감지/decode 실패 시 기존 fallback 유지 (격리).

### 수용된 쟁점 (옵션 A — 작업지시자 결정)
`map_watermark_gray()` 6구간 piecewise 계수가 복학원서 정답 PDF
단일 문서 역산 매직 넘버. 일반화 보장은 없으나:
- near-white opaque 배경 처리(구조)는 일반적
- 복학원서 시각 개선이 명확하고 시각 판정 통과
- 미감지 시 fallback 유지로 타 문서 회귀 격리

→ 작업지시자 옵션 A(수용) 결정. 톤 매핑이 단일 문서 특화임은
  본 보고서 및 PR 코드 주석에 명시됨. 후속 워터마크 문서 발생 시
  별도 이슈로 일반화 검토 권고.

### 회귀 가드 변경
`tests/issue_514.rs` + `golden_svg/issue-677` 기대값을
"투명 alpha" → "opaque gray PNG" 로 전환. 전체 test 1484 passed +
시각 판정에서 issue #514/#677 회귀 없음 확인.

## 4. 후속 권고

- 톤 매핑(`map_watermark_gray`) 은 복학원서 전용 튜닝. 다른 워터마크
  JPEG 문서에서 회귀 보고 시 일반화/문서별 가드 별도 이슈로 추적.
  관련 메모리: `feedback_pdf_not_authoritative`,
  `feedback_v076_regression_origin`,
  `feedback_hancom_compat_specific_over_general`.

## 5. 처리

- cherry-pick → 검증 → 시각 판정 통과 → `local/devel` merge
- 이슈 #938 close (PR merge 반영 확인 후)
- `pr_976_review.md` / `pr_976_report.md` → `pr/archives/`
