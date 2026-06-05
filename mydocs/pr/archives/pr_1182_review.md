# PR #1182 검토 — RawSvg(OLE/차트) 첫 로드 백지 렌더 수정

- **작성일**: 2026-05-31
- **PR**: #1182 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1164/#1148/#1095 머지 등 누적)
- **연결 이슈**: Closes #1181
- **base/head**: `devel` ← `fix/1181-hancell-ole-first-render`
- **mergeable**: MERGEABLE / mergeStateStatus: **BEHIND** (충돌 없음)
- **규모**: 2 파일, +54 / −35
- **CI**: 전부 pass (Build & Test / Canvas visual diff / Analyze ×3 / CodeQL). WASM Build 만 skip.
- **마일스톤**: v1.0.0 / 라벨: bug

## 1. PR 정보 확인

한셀 OLE / 차트 OOXML / EMF 등 `PaintOp::RawSvg` 로 emit 되는 미리보기 페이지가
rhwp-studio 첫 로드 시 백지로 그려지던 회귀를 수정.

**회귀 출처**: #1154 v2 (PR #1164, **동일 컨트리뷰터 @planet6897 가 머지**) 의 flow
디코드 안전망이 `PaintOp::Image` 만 대상으로 잡아 `RawSvg` 경로가 누락됨.
→ `image_count == 0` → `scheduleReRender` 재시도 미발화 / `prefetchFlowImages` 미매칭
→ 첫 렌더 백지, 두 번째 로드 때만 모듈 static `IMAGE_CACHE` 캐시 덕에 그려짐.

> 권위 확인: `RawSvg` emit 위치 `src/paint/json.rs:814` (`PaintOp::RawSvg { bbox, raw }`) — 본문 주장과 일치.

## 2. 변경 내용 검토

### (1) `src/document_core/queries/rendering.rs` — `collect()`
- `if let PaintOp::Image {..}` → `match op {..}` 로 전환 (구조 재배치, **순수 리팩토링**).
  - `Image` 분기 로직(image_count++, BehindText/InFrontOfText overlay) **완전 동일** 보존.
- `PaintOp::RawSvg { .. } => *image_count += 1` 신규 분기 추가.
- [x] Image 처리 의미 변화 없음 (diff 라인 대조 확인)
- [x] RawSvg 를 image_count 에 포함 → scheduleReRender 재시도 발화 (의도대로)

### (2) `rhwp-studio/src/view/page-renderer.ts` — `prefetchFlowImages()`
- `enqueue(dataUrl)` 헬퍼 + `seen: Set<string>` dedupe 추가.
- 기존 `"type":"image"` 정규식 루프 **유지** (overlay wrap behindText/inFrontOfText 필터링 보존).
- 신규: 전체 JSON 에서 `data:(image/MIME);base64,...` 직접 스캔 → rawSvg 내장 data URL prefetch.
- [x] 기존 image overlay 필터링 보존
- [x] 두 번째 정규식이 일반 image data URL 도 매칭하나 **dedupe Set 으로 중복 prefetch 방지** (안전)
- [x] rawSvg 의 wrap 은 항상 flow → overlay 필터 불필요 (주석 근거 타당)

## 3. 위험 평가

- **낮음.** `PaintOp::Image` 경로 동작 불변(본문·diff 확인). RawSvg 페이지만 추가로
  prefetch/재시도 대상에 포함. dedupe 로 중복 작업 없음.
- 회귀 가능성: `image_count` 가 RawSvg 페이지에서 0→N 으로 바뀌어 scheduleReRender 가
  발화되지만, 이는 의도된 안전망 동작이며 일반 그림 페이지에는 영향 없음.

## 4. 검증 계획 (4단계)

1. `cargo build`
2. `cargo test --lib document_core::queries::rendering` (본문 6 passed 주장 확인) + `cargo test --tests` 회귀
3. `cargo clippy` (신규 match 경고 여부)
4. `cargo fmt --all --check`
5. (메인테이너) WASM 빌드 후 `samples/한셀OLE.hwp` 첫 로드 시각 판정 — 백지 회귀 해소 확인

## 5. 판단 (예정)

검증 + 시각 판정 통과 시 **머지** 권고. 결과는 `pr_1182_report.md` 에 기록.
