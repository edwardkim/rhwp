# PR #1252 검토 — HWPX 용지 기준 BehindText z-order 보존

- **작성일**: 2026-06-02
- **PR**: #1252 (OPEN)
- **제목**: `Task #1197: HWPX 용지 기준 BehindText 그림/표 z-order 보존`
- **컨트리뷰터**: @postmelee
- **연결 이슈**: #1197
- **base/head**: `devel` ← `local/task1197`
- **Head SHA**: `9a0e63e76cb82db5a8d9db992aa24efc9f5a2f65`
- **PR 기준 base SHA**: `4fdb9c1f2cab72a8024f3cead7cb75c7f7d9769b`
- **현재 local/devel**: `2efd437a`
- **규모**: 36 files, +2195 / -369, 11 commits
- **GitHub mergeable**: `false`
- **CI**: head SHA 기준 workflow run 없음
- **PR 댓글**: 없음

## 1. PR 요약

PR #1252는 HWPX에서 용지/페이지 기준으로 배치된 `BehindText` 그림, 표, 도형이 타입별 렌더 경로에서 분리되어 같은 z-order 축으로 합성되지 않는 문제를 수정한다.

대상 증상은 #1197의 `[2027] 온새미로 1 본교재.hwpx` 주차 안내 페이지다.

기대 동작:

```text
낮은 z-order 표들
→ 전체 페이지 배경 이미지
→ 최종 표시용 표
→ InFrontOfText `01` 글상자/도형
```

기존에는 그림, 표, 도형이 별도 렌더 경로를 타면서 낮은 z-order 표 텍스트가 배경 이미지 위에 다시 보이거나, Studio Canvas2D 경로에서 front overlay canvas의 흰 배경이 하위 layer를 덮는 문제가 있었다.

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/render_tree.rs` | `RenderLayerInfo` 추가, `RenderNode.layer`로 `textWrap`, `zOrder`, `stableIndex` 보존 |
| `src/renderer/layout.rs` | Picture/Table/Shape top-level node에 공통 layer metadata 부여, paper/page 객체 정렬 보강 |
| `src/renderer/svg.rs` | `RenderNode.layer`를 우선 사용해 plane/z-order/stableIndex 정렬 |
| `src/paint/*` | `LayerNode.layer` 보존, inherited layer 기반 PaintOp replay plane 판정 |
| `src/renderer/skia/renderer.rs` | native Skia 렌더 경로에 inherited layer 전달 |
| `src/renderer/web_canvas.rs`, `src/wasm_api.rs` | filtered render에 `background` plane 추가 |
| `src/renderer/canvaskit_policy.rs` | CanvasKit policy에서 layer metadata 기준 replay plane 판정 |
| `rhwp-studio/src/view/page-renderer.ts` | `pageBackground -> behindText -> flow -> inFrontOfText` canvas layer 합성 |
| `rhwp-studio/src/view/canvaskit/*` | TypeScript CanvasKit replay plane도 layer metadata 상속 |
| `rhwp-studio/src/core/*` | WASM bridge / type 정의에 layer metadata 반영 |
| `tests/issue_1197_svg_object_zorder.rs` | #1197 synthetic z-order 회귀 테스트 추가 |
| `rhwp-studio/tests/render-backend.test.ts` | Studio layer 합성 회귀 테스트 추가 |
| `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` | 시각 판정 대상 HWPX fixture 추가 |
| `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp` | 한컴 HWP 변환본 fixture 추가 |
| `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf` | 정답 PDF fixture 추가 |
| `mydocs/*task_m100_1197*` | 계획, 구현, 단계별 작업, 완료 보고서 문서 |

## 3. 타당한 부분

### 3.1 Picture/Table/Shape를 공통 layer contract로 묶는다

#1197의 핵심은 객체 타입별 렌더 경로가 달라도 `textWrap`, `zOrder`, 입력 순서가 같은 축에서 보존되어야 한다는 점이다.

PR은 `RenderNode`와 `LayerNode`에 layer metadata를 추가해 SVG, PaintOp replay, Skia, WebCanvas, CanvasKit, Studio Canvas2D 경로가 동일한 판단 재료를 갖도록 한다. 문제 원인과 수정 방향은 잘 맞는다.

### 3.2 non-image 객체도 replay plane을 갖도록 확장한다

기존 replay plane 판정은 사실상 이미지 중심이었다. 이번 PR은 inherited layer를 통해 표/도형에서 생성된 non-image PaintOp도 `behindText` 또는 `inFrontOfText` plane으로 분류할 수 있게 한다.

이는 `BehindText` 표가 배경 이미지와 같은 z-order 공간에서 가려져야 하는 #1197 증상에 필요한 보강이다.

### 3.3 Studio Canvas2D의 overlay 배경 문제를 함께 처리한다

PR 본문에 적힌 것처럼 Studio의 front canvas가 공통 CSS 흰 배경을 상속하면, 실제 front layer에 `01`만 있어도 하위 layer 전체를 가린다.

Canvas layer 합성 순서와 overlay 투명 배경을 함께 조정한 것은 웹 캔버스 시각 판정 관점에서 필요하다.

## 4. 위험 및 주의 사항

### 4.1 PR은 Draft 해제되었지만 GitHub CI 실행이 확인되지 않는다

GitHub 기준 PR #1252는 이제 `draft: false`다.

다만 `9a0e63e76cb82db5a8d9db992aa24efc9f5a2f65` head SHA 기준 workflow run이 조회되지 않았다. PR 본문에는 로컬 검증 목록이 있지만, 수용 전에는 현재 `local/devel` 기준 로컬 전체 검증과 WASM/Studio 빌드가 필요하다.

### 4.2 현재 `local/devel`과 병합 충돌이 있다

비파괴 `merge-tree` 확인 결과 현재 `local/devel`과 병합 시 `mydocs/orders/20260602.md`에서 충돌이 발생한다.

코드 충돌은 확인되지 않았지만, PR 브랜치를 그대로 병합하려면 문서 충돌 해소가 필요하다.

### 4.3 PR base가 현재 devel보다 뒤처져 있다

PR의 base SHA는 `4fdb9c1f`이고 현재 `local/devel`은 #1249 처리 커밋 `2efd437a`다. 단순 diff에는 최신 devel 문서 변경과 섞인 착시가 생길 수 있으므로, 검증 브랜치에서 실제 3-way merge 결과 기준으로 확인해야 한다.

### 4.4 변경 범위가 여러 렌더 백엔드에 걸쳐 있다

SVG, PaintOp, Skia, WebCanvas, CanvasKit, Studio Canvas2D를 동시에 건드린다. 한 경로만 맞고 다른 경로가 틀어질 수 있으므로, SVG와 rhwp-studio 웹 캔버스 양쪽에서 시각 판정해야 한다.

### 4.5 정답 샘플이 PR에 포함되었다

이전 draft 상태에서는 #1197 대상 원본이 repo에 포함되지 않았지만, 최신 PR에는 다음 파일이 추가되었다.

```text
samples/hwpx/[2027] 온새미로 1 본교재.hwpx
samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp
pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf
```

PR 본문에도 before/after/PDF 비교 이미지가 첨부되었다. 이제 메인테이너가 같은 fixture로 직접 SVG/웹 캔버스 시각 판정을 수행할 수 있다.

### 4.6 PR 본문은 `Related: #1197` 상태다

PR이 수용되어도 #1197은 자동 close되지 않는다. 최종 병합 후에는 이슈를 수동 close하거나 PR body를 `Closes #1197`로 조정해야 한다.

## 5. 권장 검증

현재 상태에서 바로 `local/devel`에 반영하지 말고, 검증 브랜치에서 충돌 해소 후 진행한다.

```text
git checkout -b local/pr1252-verify local/devel
git merge pr/1252
```

충돌 예상:

```text
mydocs/orders/20260602.md
```

충돌 해소 후 권장 테스트:

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --lib replay_order
cargo test --lib canvaskit_policy
cargo test --tests
cd rhwp-studio && npm test && npm run build
docker compose --env-file .env.docker run --rm wasm
```

시각 판정 후보:

| file | page | 확인 항목 |
|---|---:|---|
| `[2027] 온새미로 1 본교재.hwpx` | rhwp 4쪽 / CLI `-p 3` | 중앙 이미지, `01`, 하단 `1주차` 및 설명 텍스트 z-order |
| `[2027] 온새미로 1 본교재.pdf` | PDF 3쪽 | 정답 비교 기준 |
| `tests/issue_1197_svg_object_zorder.rs` synthetic SVG | generated | 낮은 z 표 < 배경 이미지 < 최종 표 < front shape 순서 |

판정 포인트:

- `PROLOGUE` 및 출처 목록이 최종 출력 위에 남지 않아야 한다.
- 중앙 장식 이미지가 뒤쪽 전체 페이지 이미지에 가려지지 않아야 한다.
- `01` front shape가 배경 위에 표시되어야 한다.
- Studio Canvas2D에서 흰 overlay canvas가 하위 layer를 덮지 않아야 한다.

## 6. 권장 처리

권장안: **수용 후보로 분류하되, 현재는 merge 보류한다.**

보류 사유:

- GitHub CI 실행이 확인되지 않는다.
- 현재 `local/devel`과 문서 충돌이 있다.
- 변경 범위가 여러 렌더 백엔드에 걸쳐 있어 로컬 전체 검증과 WASM/Studio 빌드가 필요하다.
- #1197 자동 close 키워드가 없어 최종 이슈 close 절차가 별도로 필요하다.

다만 구현 방향은 #1197의 원인과 잘 맞으며, Picture/Table/Shape의 layer contract를 공통화하는 접근은 장기적으로도 타당하다.

## 7. 다음 승인 요청

권장 절차:

```text
1. `local/pr1252-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1252 병합 및 `mydocs/orders/20260602.md` 충돌 해소
3. 테스트/WASM/Studio 빌드 수행
4. `[2027] 온새미로 1 본교재.hwpx` 기준 SVG/웹 캔버스 시각 판정
5. 판정 통과 후 local/devel 반영 및 #1197 close 처리
```

## 8. 검증 브랜치 진행 기록

2026-06-02 `local/pr1252-verify`에서 PR #1252를 병합하고 `mydocs/orders/20260602.md` 충돌을 해소했다.

실행 완료:

```text
git diff --check
cargo fmt --all --check
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --lib replay_order -- --nocapture
cargo test --lib canvaskit_policy -- --nocapture
cargo test --tests
cd rhwp-studio && npm test && npm run build
docker compose --env-file .env.docker run --rm wasm
```

웹 캔버스는 메인테이너 시각 판정 통과. SVG는 최초 판정에서 모두 실패했다.

원인:

- SVG 단일 스트림 렌더러가 root 자식들을 `BehindText → Flow → InFrontOfText`로 재정렬했다.
- `MasterPage`가 Flow plane으로 분류되어 BehindText 용지 기준 표보다 뒤에 출력되었다.
- 짝수 바탕쪽의 전체 페이지 그림이 #1197의 최종 표시용 BehindText 표를 다시 덮었다.

보완:

- SVG z-plane을 `PageBackground → MasterPage → BehindText → Flow → InFrontOfText`로 조정했다.
- `tests/issue_1197_svg_object_zorder.rs`에 바탕쪽이 BehindText 용지 기준 객체보다 먼저 출력되는 회귀 테스트를 추가했다.

재생성한 SVG 판정 후보:

```text
output/poc/pr1252-zorder/hwpx-page4-fixed/[2027] 온새미로 1 본교재_004.svg
output/poc/pr1252-zorder/hwp-page4-fixed/[2027] 온새미로 1 본교재_004.svg
```

재검증:

```text
git diff --check
cargo fmt --all --check
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
```

SVG 재판정:

| file | 판정 | 비고 |
|---|---|---|
| `output/poc/pr1252-zorder/hwpx-page4-fixed/[2027] 온새미로 1 본교재_004.svg` | 통과 | 2026-06-03 메인테이너 시각 판정 통과, HWPX 원본 target |
| `output/poc/pr1252-zorder/hwp-page4-fixed/[2027] 온새미로 1 본교재_004.svg` | 통과 | 2026-06-03 메인테이너 시각 판정 통과 |
