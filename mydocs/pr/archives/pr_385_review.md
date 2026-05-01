# PR #385 검토 — Task #363: native PageRenderTree bridge API

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#385](https://github.com/edwardkim/rhwp/pull/385) |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) |
| base / head | `devel` ← `feature/issue-363-native-render-tree-api` |
| state | OPEN (**DRAFT**) |
| mergeable | MERGEABLE |
| mergeStateStatus | CLEAN |
| 이슈 | [#363](https://github.com/edwardkim/rhwp/issues/363) |
| 변경 통계 | +760 / -51, 22 files |

## 작성자 정보

기존 활발한 컨트리뷰터:
- 8 머지된 PR (Firefox 확장, Safari, AMO 검증, Chrome 옵션 CSP 등)
- 신뢰도 높음

## 요약

`DocumentCore` 에 **native viewer/bridge 용 public API 2개** 추가:

```rust
pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError>
pub fn get_bin_data(&self, index: usize) -> Option<&[u8]>
```

`PageRenderTree` 와 관련 render node 타입에 `serde::Serialize` derive 추가 (native bridge 의 JSON 직렬화 지원).

## 배경 — 이슈 #363

작성자의 별도 프로젝트 [`alhangeul-macos`](https://github.com/postmelee/alhangeul-macos) (rhwp core 사용 macOS native viewer) 에서 발견된 요구.

기존 WASM `getPageRenderTree(pageNum)` 경로는 존재하지만 **Rust core 의 public API** 로 노출 안됨. native bridge (Swift / FFI 등) 에서 안정적 사용을 위해 release contract 로 노출 필요.

## 변경 내용

### 1. 신규 public API 2개 (`src/document_core/queries/rendering.rs`)

```rust
pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError> {
    let tree = self.build_page_tree(page_num)?;
    let _overflows = self.layout_engine.take_overflows();
    Ok(tree)
}

pub fn get_bin_data(&self, index: usize) -> Option<&[u8]> {
    self.document.bin_data_content.get(index).map(|b| b.data.as_slice())
}
```

**핵심**: 기존 `build_page_tree` (pub(crate)) 를 감싸는 thin wrapper. 새 로직 도입 없음.

### 2. `serde::Serialize` derive 추가

다음 타입에 `Serialize` 추가:
- `PageRenderTree` 와 주요 render node 타입 (TextRun, ImageNode, ShapeNode, TableCellNode 등)
- 이미지 bytes (`ImageNode.data`, `PageBackgroundImage.data`) 는 `#[serde(skip)]` — JSON 에 직접 포함 안 함 (대신 `bin_data_id` + `get_bin_data(index)` 로 별도 조회)

### 3. Cargo.toml 신규 의존성

```toml
+ serde = { version = "1", features = ["derive"] }
```

⚠️ **`serde` 가 현재 직접 의존성에 없음** (transitive 만 있음). 새 직접 의존성 추가.

### 4. 문서

- `mydocs/manual/native_render_tree_bridge_api.md` (신규) — native bridge API 사용법
- `mydocs/tech/hwp_spec_errata.md` 보강 — `bin_data_id` 인덱스 기준
- 5개 task_m100_363 문서 (수행/구현 계획서, stage1-4, report)

### 5. 테스트

- 신규 단위 테스트 2건 (build_page_render_tree, get_bin_data)
- cargo test --lib: 1010 passed (현재 1014 → 1016 예상)

## 변경 평가

### 강점
1. **Thin wrapper** — 기존 검증된 `build_page_tree` 를 감싸는 형태. 새 로직 도입 없음 → 회귀 위험 낮음
2. **명확한 use case** — alhangeul-macos 의 실제 요구. 작성자가 이미 검증 commit 보유 ([`1e9d78a`](https://github.com/edwardkim/rhwp/commit/1e9d78a1d40c71779d81c6ec6870cd301d912626))
3. **이미지 bytes 처리 합리적** — `#[serde(skip)]` 로 JSON 크기 폭발 차단, 별도 lookup API 제공
4. **이슈에 구체 제안 포함** — 작성자가 이슈에 본인이 검증한 commit 까지 첨부. 본 PR 은 그 방향 그대로 정리
5. **rhwp 자체 회귀 테스트** — `build_page_tree` 는 기존 SVG/HTML 경로에서 사용되므로 svg_snapshot 6/6 통과 시 회귀 0
6. **DRAFT 상태** — 작성자가 신중히 검토 받기를 의도

### 약점 / 점검 필요

1. **`serde` 직접 의존성 신규 추가** — Cargo.toml 에 새 의존성. 비록 가벼운 의존성이지만 정책 검토 필요
2. **Public API surface 확대** — 한번 노출된 API 는 호환성 부담. release contract 결정 (semver 영향 검토)
3. **alhangeul-macos 외부 의존성** — 작성자의 별도 프로젝트 요구. 본 프로젝트 (rhwp) 의 일반 use case 인지 작업지시자 검토 필요
4. **`PageRenderTree` 안정성** — 내부 구조였던 타입이 public API 로 노출. 향후 변경 시 호환성 깨짐 가능 (semver minor break 의식)
5. **`take_overflows()` 호출 의도** — wrapper 안에서 buffered overflows 를 버림. 외부 호출자가 overflow 정보를 못 받는 영향 (PR 본문에 명시는 안됨)

## 메인테이너 작업과의 중복 분석

검색 결과 — 본 결함의 메인테이너 동시 정정 없음. 본 PR 은 **신규 API 추가** 이고 작업지시자 측 동시 작업 없음.

## 처리 방향 후보

### 옵션 A: 정상 머지 (DRAFT → Ready 전환 후)
- 변경 작고 명확, 회귀 위험 낮음
- 작성자 신뢰도 높음
- 작업지시자가 외부 native bridge 정책 / API 노출 정책 결정 후 진행

### 옵션 B: API 노출 정책 검토 후 결정
- `PageRenderTree` 가 안정 API 로 적합한지 (자주 변경되는 내부 구조 가능성)
- semver 정책 (minor 또는 patch 결정)
- alhangeul-macos 가 rhwp 의 1차 use case 인지 검토

### 옵션 C: 일부 변경만 흡수
- `build_page_render_tree` / `get_bin_data` 만 (가벼움)
- `Serialize` derive 는 별도 task 로 (해당 타입의 안정성 검토 후)

## 권장

**옵션 B → A** — API 노출 정책 검토 후 정상 머지.

검토 항목:
1. `PageRenderTree` 의 안정성 — 향후 자주 변경될 타입인지
2. semver 영향 — public API 추가 → minor bump (v0.7.7 → v0.8.0?) 또는 patch (v0.7.8?)
3. native bridge 정책 — rhwp 가 native bridge 를 공식 지원하는 방향인지 (alhangeul-macos 같은 외부 프로젝트의 1차 use case)
4. `serde` 직접 의존성 — 정책상 문제 없음 (가벼움, 표준)

상기 검토 후 정상 머지 (DRAFT → Ready).

## 다음 단계 — 작업지시자 결정

옵션 A / B / C 중 결정 + 검토 항목 (API 노출 정책, semver, native bridge 방향 등) 답변 부탁드립니다.

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 (8 머지 PR 이력) ✅
- [x] 변경 작고 명확 (thin wrapper + Serialize derive) ✅
- [x] 회귀 위험 낮음 (기존 build_page_tree 재사용) ✅
- [x] 메인테이너 동시 정정 없음 ✅
- [x] DRAFT 상태 — 작성자 검토 받을 의도 ✅
- [ ] API 노출 정책 — 작업지시자 결정 필요
- [ ] semver 정책 — 작업지시자 결정 필요
- [ ] native bridge 공식 지원 방향 — 작업지시자 결정 필요

## 참고

- 이슈: [#363](https://github.com/edwardkim/rhwp/issues/363) (OPEN)
- PR: [#385](https://github.com/edwardkim/rhwp/pull/385) (DRAFT)
- 작성자 외부 프로젝트: [alhangeul-macos](https://github.com/postmelee/alhangeul-macos)
- 작성자 검증 commit: [1e9d78a](https://github.com/edwardkim/rhwp/commit/1e9d78a1d40c71779d81c6ec6870cd301d912626)
