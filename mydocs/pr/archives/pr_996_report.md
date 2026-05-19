# PR #996 처리 보고서 — render: add browser CanvasKit direct renderer

- 처리일: 2026-05-19
- 컨트리뷰터: [@seo-rii](https://github.com/seo-rii)
- 결정: **옵션 A (수용)** — 작업지시자 승인
- 머지: `3d4a9c34` (no-ff, local/devel)

## 1. 결정 사유

@seo-rii render 백엔드 핵심 컨트리뷰터(15 PR 시리즈)의 P16. P15(#925 CanvasKit replay diagnostics) 다음 단계로 Studio에서 browser CanvasKit backend를 opt-in 실행하는 foundation. 하위 호환 100% (기본 Canvas2D, dynamic import 분리), opt-in 게이팅 견고, 케이스별 명시 가드(policy.ts), 본질을 회귀 테스트로 고정. Rust 무변경 TypeScript 전용.

## 2. 처리 내역

| 커밋 | 내용 | 작성자 |
|------|------|--------|
| `933c056e` | feat: browser CanvasKit direct renderer (본질, cherry-pick) | @seo-rii (메타데이터 보존) |
| `20482412` | docs: P16 폰트 한계 주석 (후속 폰트 단계 컨텍스트) | 메인테이너 |
| `3d4a9c34` | Merge PR #996 (no-ff) | 메인테이너 |

- 충돌: `package-lock.json` add-only 1블록만 → `--ours` + `npm install` 재생성으로 해결 (canvaskit-wasm + 전이 의존 정상 반영). 본질 소스 5파일 auto-merge 성공, 의미적 충돌 0.

## 3. 시각 판정 — 폰트 한계 발견 및 처리

작업지시자 시각 판정 중 **CanvasKit opt-in 시 모든 글자(영문 포함)가 안 나오는 현상** 확인.

근본 원인:
- 기존 `rhwp-studio/public/rhwp_bg.wasm`은 PR Rust 무변경이라 `getPageLayerTree`만 export (`getPageLayerTreeWithProfile` 없음)
- CanvasKit renderer는 자체 텍스트 렌더링 — 단일 기본 typeface(NotoSansKR-Regular.woff2)만 로드. 로딩/디코딩 실패 또는 미해석 op 시 textRun 미출력
- 이는 PR 본문이 명시한 **non-goal**: "TextRun effects, equation/raw SVG, glyph sidecar direct replay를 이 PR에서 완성하지 않습니다"

작업지시자 결정: **동일 컨트리뷰터(@seo-rii)가 후속 작업에서 폰트 처리를 진행**하므로, 본 PR은 한계를 명시하는 **주석만 추가 후 체리픽 처리**.

처리: `canvaskit-renderer.ts` 2개 지점에 P16 폰트 한계 주석 추가 (`20482412`)
- 기본 typeface 로딩부 (라인 84): 단일 CJK typeface만 로드, 실패 시 textRun 미출력 가능, 후속 폰트 단계 보강 명시
- textRun skip 가드 (라인 398): 비-Latin 텍스트 skip + diagnostics 기록, Canvas2D 미대체가 P16 본질, 후속 보강 명시

## 4. 자기 검증 결과

| 항목 | 결과 |
|------|------|
| `npm test` (Node 22.18, 무플래그) | **25/25 통과** (PR 신규 6 + 기존 19, 회귀 0) |
| `npm run build` (tsc + Vite) | 성공 — canvaskit dynamic chunk 분리 확인 (`canvaskit-renderer-*.js` 129KB + `canvaskit-*.wasm` 7.2MB 별도) |
| `cargo test --release --lib` | 1307 passed / 0 failed |
| `cargo clippy --release --lib -D warnings` | 통과 |
| `cargo fmt --check` | exit 0 (PR은 .rs 0개) |
| 주석 추가 후 재검증 | npm test 25/25 + build 회귀 0 |

> 초기 `npm test` 무플래그 실패는 로컬 Node가 `.ts` strip-types 미지원하는 환경 문제 (devel baseline 동일). 작업지시자가 Node v22.18.0으로 재구성 후 무플래그 25/25 통과.

## 5. 라이센스

`canvaskit-wasm@0.41.1` = **BSD-3-Clause** (Google Skia, Chrome 동일 엔진). 프로젝트 MIT와 호환, 법적 문제 없음.

## 6. 후속

- **@seo-rii 후속 폰트 단계** (Refs #536): fontFamily 별 typeface 매핑, glyph sidecar direct replay, 폴백 체인 — 본 PR 주석에 컨텍스트 명시됨
- `getPageLayerTreeWithProfile` WASM export — 후속 Rust 작업 시 renderProfile 반영 활성화 (현재 기존 WASM은 미존재, profile 무시됨)

## 7. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @seo-rii 15 PR render 시리즈 P15→P16 연속 점검
- `feedback_hancom_compat_specific_over_general` — policy.ts 케이스별 명시 가드 (권위 사례)
- `feedback_small_batch_release_strategy` — opt-in·하위호환 신규 모듈 점진 분리 (P15→P16, full parity non-goal 명시)
- `feedback_image_renderer_paths_separate` — 기존 Canvas2D 경로 무변경, 별도 backend 추가
- `feedback_visual_judgment_authority` — 작업지시자 시각 판정으로 폰트 한계 발견 → 후속 분리 결정 (sweep/test green ≠ 시각 완전성)
