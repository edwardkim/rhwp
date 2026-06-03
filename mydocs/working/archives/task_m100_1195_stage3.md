# Task #1195 Stage 3 완료 보고서 — 좌표 정밀 회귀 테스트 + 가드 유효성 입증

- **이슈**: #1195
- **브랜치**: `local/task1195`
- **단계**: Stage 3 / 4

## 회귀 테스트 방식 — 좌표 정밀 단언 (표 겹침 범위 한정)

golden 전체 스냅샷은 표 겹침 외 무관 변경에도 실패하므로, **표 겹침으로 범위를 좁힌**
좌표 단언 테스트로 확정 (작업지시자 지시):

- `tests/issue_1195_cell_table_empty_line.rs`: `render_page_svg_native(0)` SVG 에서
  4×1 표(폭 ~589.6px) top y 와 제목 "[필수]" 글자 y 를 추출, **표 top > 제목 y**(비겹침) 단언.
- 표/제목을 못 찾으면 명시적 panic(헐거운 통과 방지) — 첫 시도의 논리 결함 정정.

## 가드 유효성 입증 (보정 ON/OFF 양방향)

| 상태 | native 4×1 표 top y | 좌표 테스트 |
|------|---------------------|---------------|
| 보정 ON | **643.8** (제목 639.8 아래, 겹침 해소) | ✅ pass |
| 보정 OFF (코드 직접 무력화 `if false`) | **631.8** (제목 위, 겹침 복귀) | ❌ fail (재현) |

- 보정을 빼면 golden 과 달라져 테스트가 실패함을 확인 → **가드가 결함을 실제로 잡음**.
- (직전까지 "보정 없이도 통과"한 혼란의 원인: 보정이 Stage 2 에서 이미 커밋되어
  `git stash <file>` 로는 워킹트리 변경이 아니라 stash 대상이 아니었음 → 보정이 안 빠진 채
  항상 643.8 출력. 코드 직접 무력화로 631.8/fail 을 확인해 입증.)
- 검증 후 보정 재적용(`if false` 제거), golden 을 보정본(643.8)으로 확정.

## CLI vs native 동일성 확인

- CLI `export-svg`(시각 판정 SVG)와 `render_page_svg_native`(테스트/WASM) 모두 보정 적용
  시 4×1 표 643.8 동일 → **두 경로 정합**. (당초 "native 미적용" 의심은 stash 착시로 오판,
  실제 동일.)

## 샘플 픽스처 커밋

golden 테스트가 `samples/hcar-001.hwp` 를 읽으므로 픽스처 커밋:
- `samples/hcar-001.hwp`, `samples/hwpx/hcar-001.hwpx` (일반 git)
- `pdf-large/hwpx/hcar-001.pdf` (Git LFS, `filter: lfs` 확인 — 한컴 정답지)

## issue-267 golden 충돌 (무관, 미처리)

- `tests/golden_svg/issue-267/ktx-toc-page.svg` 가 UU(고아 충돌) 상태로 워킹트리에 존재.
- **Task #1195 와 무관**(issue-267=KTX, 본 작업=hcar). devel 에는 정상본 존재, 본 작업 커밋에
  미포함. 작업지시자 지시대로 **건드리지 않음** — devel 머지 후 devel 정상본으로 회귀.
- 본 Stage 커밋에 issue-267 미포함 확인(명시적 add 로 hcar 관련만 staging).

## 검증

| 항목 | 결과 |
|------|------|
| `issue_1195_hcar...` golden 테스트 | ✅ pass |
| 가드 유효성 (보정 OFF 시 fail) | ✅ 입증 |
| hcar-001 6페이지 회귀 (Stage 2) | ✅ p2~p6 무변경 |
| 보정 검증 잔재 (`if false`) | ✅ 0 (정상) |

## 다음 단계 (Stage 4)

- 전체 `cargo test --tests` (golden 추가분 포함) 0 failed 확인.
- WASM 빌드 (Stage 2 빌드본 = 보정 반영, 재확인).
- 최종 보고서 + orders + devel 머지(+ issue-267 은 devel 정상본 자동 회귀).
