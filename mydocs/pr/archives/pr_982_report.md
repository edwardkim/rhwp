# PR #982 최종 보고 — PageLayerTree textRun display text contract

## 1. 결정

**merge** — 순수 additive 설계 + 전 검증 통과. PUA 규칙 단일 출처화.

| 항목 | 값 |
|------|-----|
| 번호 | #982 |
| 제목 | Task #948: PageLayerTree textRun display text contract |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `task-948-pagelayertree-display-text` |
| 연결 이슈 | Refs #948 (관련 #937/#947) |
| 처리 | cherry-pick (`babb6816` → 최신 local/devel) |

## 2. 검증 결과

cherry-pick `72f75969`. 충돌은 `mydocs/orders/20260518.md` 1건뿐
(코드 충돌 없음) — 기존 #971/#987 기록과 PR #948 기록 양쪽 보존하여
해소.

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| `issue_948` / `issue_937` | ✅ 4 + 1 passed |
| `paint::json` / `paint::schema` | ✅ 9 + 1 passed |
| 전체 `cargo test` | ✅ 1487 passed, 0 failed (issue_948 +3) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 |
| WASM 빌드 (Docker) | ✅ 성공 (composer/json WASM 호환) |
| CI | ✅ 전부 pass |

## 3. 평가 요약

### 강점
- **순수 additive contract**: 기존 `text`/`positions`/`clusters`
  의미·값 불변. 신규 `displayText`/`displayPositions` optional,
  `requiredFeatures: []` 로 기존 consumer fallback 보존.
- `expand_pua_display_text()` helper 추출 + 기존
  `expand_pua_render_text()` 위임 → PUA 규칙 단일 출처화.
  PageLayerTree JSON 경로가 SVG/Canvas/Skia 렌더러와 동일
  display 규칙 사용 (이슈 #948 핵심 해소).
- `displayText == text` 인 일반 run 은 신규 필드 생략 (JSON 크기/
  하위 호환). 미지 PUA 는 추측 없이 원문 유지 (안전).
- schema minor 11 → 12 additive revision 규약 부합.

### 검토 포인트 해소
- CONFLICTING 은 cherry-pick + orders 충돌 수동 병합으로 해소
  (양쪽 기록 보존, 코드 무충돌).
- paint contract 회귀 가드(paint::json/schema, issue_937/948)
  전부 통과 — renderer-independent contract 안정성 확인.
- 본질이 시각 판정 아닌 JSON contract/PUA 매핑 → 계약·단위
  테스트가 판정 핵심, 전부 통과.

## 4. 처리

- cherry-pick → 검증 통과 → `local/devel` merge
- PR #982 close (cherry-pick 반영 명시) + 이슈 #948 close
- `pr_982_review.md` / `pr_982_report.md` → `pr/archives/`
