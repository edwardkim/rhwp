# PR #397 처리 보고서 — 수식 ATOP 파싱 및 렌더링 보정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#397](https://github.com/edwardkim/rhwp/pull/397) |
| 작성자 | [@cskwork](https://github.com/cskwork) — **본 저장소 첫 외부 컨트리뷰터** |
| 처리 결정 | **cherry-pick 머지** (작성자 정정 후) |
| 처리 일자 | 2026-04-29 |

## 작성자 정황

@cskwork — **본 저장소의 첫 번째 외부 컨트리뷰터**. 본 PR 이 첫 머지 PR.

작업지시자 git 정보 확인 결과 실제 사람 컨트리뷰터 (AI 에이전트 아님). "Agentic-Worker" 표기는 본인 직책이 아닌 의미 — 사람 개발자 본인.

## 처리 절차

### Stage 0: 1차 검토 (2026-04-28) — 옵션 B 절차 안내

1차 검토 시 다음 항목 안내:
1. devel 기반 rebase 필요 (base 가 PR #395, #396 머지 전)
2. 시각 검증 자료 첨부 요청
3. CI 실행 정황 (statusCheckRollup 비어있음)
4. clippy 정황 (작성자 환경의 716 warning 정황은 본 저장소 CI 환경 무영향)

### Stage 1: 작성자 대응 (2026-04-28T15:08:56Z)

작성자가 안내 항목 모두 정확히 대응:

| 항목 | 작성자 대응 |
|------|------------|
| devel 기반 rebase | ✅ `8be4940` 단일 커밋으로 rebase (PR #395 + #396 위) |
| 시각 검증 자료 | ✅ fork (`cskwork/rhwp/pr397-visuals` 브랜치) 에 export-svg 풀 파이프라인 SVG 3 종 (ATOP / OVER 회귀 확인 / GROUP) + 재현 데모 코드 (`atop_visual_demo.rs`) |
| CI 실행 | ✅ 작성자가 GitHub Actions "action_required" 정황 식별 후 메인테이너 승인 요청 |
| 로컬 테스트 | ✅ cargo test --lib 1033 passed, equation 52 passed |

### Stage 2: 메인테이너 CI 승인

GitHub Actions API 로 첫 외부 PR 의 CI 승인:

```bash
gh api -X POST repos/edwardkim/rhwp/actions/runs/25060944046/approve
gh api -X POST repos/edwardkim/rhwp/actions/runs/25060943866/approve
```

CI 자동 실행 → **Build & Test SUCCESS** ✅

### Stage 3: cherry-pick

`local/pr397` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `a9eb13c` (← `8be4940`) | @cskwork (donga-csk) | fix: 수식 ATOP 파싱 및 렌더링 보정 |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 4: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1046 passed** (1044 → +2 신규 ATOP 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo test --lib renderer::equation` | ✅ 52 passed |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 22s, 4,134,499 bytes |
| **GitHub CI (Build & Test)** | ✅ SUCCESS (메인테이너 승인 후 자동 실행) |

### Stage 5: 시각 판정

작업지시자가 ATOP 전용 테스트 샘플 추가 (`samples/atop-equation-01.hwp`):

`output/svg/pr397-visual/atop-equation-01.svg` 검증:

| 케이스 (SVG `<g>` 위치) | text | line | 평가 |
|-----------------------|------|------|------|
| 1번째 (y=134) | a, b | **0** | ATOP — 분수선 없음 ✅ |
| 2번째 (y=186) | a, b | **1** | OVER — 분수선 유지 (회귀 없음) ✅ |
| 3번째 (y=243) | x+y, u-v | **0** | ATOP 그룹 `{x+y} atop {u-v}` ✅ |

작업지시자 시각 판정: **통과**.

## 변경 요약

### 본질 — 한컴 수식 ATOP / OVER 의미 분리

기존 코드: `EqNode::Atop` AST 타입은 있으나 파싱 / 레이아웃 / 렌더 미완성 → `a atop b` 가 빈 노드로 처리되거나 `Fraction` 으로 대체되어 분수선 부적절 출력.

정정:

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | `OVER` + `ATOP` 통합 중위 연산자 처리 (top-level + group), `EqNode::Atop` 생성 |
| `src/renderer/equation/layout.rs` | `LayoutKind::Atop` 추가 + `layout_atop()` (분수선 없는 위/아래 배치) |
| `src/renderer/equation/svg_render.rs` | `LayoutKind::Atop` 분기 (line 없이 top/bottom 만 렌더) |
| `src/renderer/equation/canvas_render.rs` | 같은 패턴 (Canvas 경로) |

### 단위 테스트

- `parser::tests::test_atop` — `a atop b` 파싱 → `EqNode::Atop` 검증
- `svg_render::tests::test_atop_svg_has_no_fraction_line` — SVG 에 `<line>` 없음 + 두 텍스트의 y 좌표 다름 검증
- 회귀 보존: `test_fraction_svg` — OVER 의 분수선 유지

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1046 + svg_snapshot 6/6 + clippy 0 + WASM 빌드 + CI SUCCESS |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 직접 판정 통과 (atop-equation-01.hwp) |
| PR 댓글 톤 — 차분하고 사실 중심 | ✅ 단 첫 외부 컨트리뷰터 환영 표현 적절 (작업지시자 지시 반영) |
| output 폴더 가이드라인 | ✅ `output/svg/pr397-visual/` 정식 위치 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr397` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close (이슈 연결 없음 — 자체 발견 결함) |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr397` → `local/devel` → `devel` 머지 + push
3. PR #397 close + **첫 외부 컨트리뷰터 환영 + 감사 댓글**

## 참고

- 검토 문서: `mydocs/pr/pr_397_review.md` (1차 검토 + 정정 반영)
- PR: [#397](https://github.com/edwardkim/rhwp/pull/397)
- 작성자 fork 시각 자료: [cskwork/rhwp/pr397-visuals](https://github.com/cskwork/rhwp/tree/pr397-visuals/docs/pr397-visuals)
- 작업지시자 추가 샘플: `samples/atop-equation-01.hwp`
- 시각 판정 산출물: `output/svg/pr397-visual/atop-equation-01.svg`
