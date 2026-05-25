# PR #1096 최종 보고 — render: expand CanvasKit image replay coverage (P18)

## 1. 결정

**merge 수용** — 검증 7/7 통과, CI 전부 pass, scope 명확, 회귀 없음.

| 항목 | 내용 |
|------|------|
| PR | [#1096](https://github.com/edwardkim/rhwp/pull/1096) |
| 작성자 | seo-rii (Seohyun Lee) — CanvasKit P-series 핵심 컨트리뷰터 (P8~P18) |
| 이슈 | `Refs #536` (멀티 렌더러 트래킹 — 단계 진행, closes 아님) |
| 변경 | +592 / -39, 5 files (TS 4 + Rust 1) |
| commits | 3 (feat + review feedback 2회 반영) |
| merge 방식 | merge commit (3 commits 보존) |

## 2. 본 P18 의 범위

P17 (#1057, direct replay contract 강화) 후속:

1. **CanvasKit image replay 확장**: `crop` / `originalSize` / `fillMode` / `transform`
   payload 직접 소비
2. **image cache key 강화**: base64 fingerprint 포함 → 같은 ref 다른 payload 충돌 방지
3. **순수 함수 helper 분리**: `image-replay.ts` 신규 (cache key + source rect + fill-mode anchor)
4. **replay item 분류 정교화**: direct (단순 image) vs deterministic detail
   (effect/brightness/contrast)

## 3. 검증 결과 (메인테이너 재검증)

PR head (`pr-1096-head`) 직접 체크아웃 후 로컬 검증:

| 항목 | 명령 | 결과 |
|------|------|------|
| canvaskit lib tests | `cargo test --release --lib canvaskit` | ✅ **18 passed** |
| lib 전체 | `cargo test --release --lib` | ✅ **1341 passed / 0 failed** |
| clippy | `cargo clippy --release --lib -- -D warnings` | ✅ clean |
| fmt | `cargo fmt --all --check` | ✅ clean |
| TS noEmit | `npx tsc --noEmit` | ✅ OK |
| npm test | `npm test` | ✅ **34 passed / 0 failed** (cache key / source rect / fill-mode 회귀 가드 포함) |
| npm run build | `npm run build` | ✅ vite + PWA 정상 |
| CI (GitHub Actions) | Build & Test / CodeQL / Canvas visual diff / Analyze | ✅ 전부 pass |

## 4. 코드 변경 평가

### 4.1 `types.ts` — LayerImageOp 확장

```typescript
originalSize?: { width: number; height: number };
crop?: { left: number; top: number; right: number; bottom: number };
transform?: LayerPathTransform;  // Task #1067 재사용
```

optional field 추가만 — 호환적.

### 4.2 `image-replay.ts` 신규 helper (138 lines)

순수 함수 영역 — `canvasKitImageCacheKey` (fnv1a32), `canvasKitImageSourceRect` 등.
회귀 테스트 동반. 다른 영역 영향 없음.

### 4.3 `canvaskit-renderer.ts` (+198 lines)

`renderImage` 본체 + cache key + transform 적용 영역. Task #1067 의 `renderPath` 정정과
동일 파일이지만 함수 영역 분리 — conflict 없음 (mergeable=MERGEABLE).

### 4.4 `canvaskit_policy.rs` (+252 lines)

Rust 측 정책 — replay item 분류 (direct vs deterministic detail) + diagnostics. 실제 동작
변경 부재 (PR 본문 보증, canvaskit 18 test 확인).

## 5. 호환성 평가

- 기본 renderer = **Canvas2D** (무영향 ✓)
- CanvasKit = **opt-in path** (확장 영역만)
- public Skia/PDF API 변경 없음
- image effect/brightness/contrast 는 **결과 변경 없음** — diagnostics 만

## 6. 처리

- PR head 직접 검증 → 모두 통과 → CI 통과 → merge 결정
- GitHub PR merge (merge commit, 3 commits 보존 — feat + review feedback 영역 가시화)
- review/report → `mydocs/pr/archives/` 이동
- 이슈 #536 은 트래킹 이슈 — close 안 함 (P19+ 후속 예정)

## 7. 메모리 룰 정합

- ✅ `feedback_contributor_cycle_check` — seo-rii P-series (P8~P18) 핵심 컨트리뷰터
- ✅ `feedback_pr_supersede_chain` — P-series 단계적 진행 (개별 merge 패턴)
- ✅ `feedback_pr_comment_tone` — 반복 컨트리뷰터, 차분한 사실 중심 merge 메시지
- ✅ `feedback_release_sync_check` — devel merge 전 origin/devel 동기화
- ✅ `feedback_push_full_test_required` — Rust lib + tests + clippy + fmt + npm test/build 모두 통과

## 8. 후속

- 본 PR merge 후 `pr/pr_1096_review.md` + `pr_1096_report.md` → `pr/archives/` 이동 commit
- 이슈 #536 트래킹 유지 (P19+ 후속)
- 본 PR 의 non-goals (grayscale/blackWhite/pattern8x8 filter direct, RawSvg/WMF/OLE, TextRun effect/glyph, renderer sweep/WebGPU/PDF) 는 별도 후속 PR 영역
