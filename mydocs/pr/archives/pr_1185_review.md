# PR #1185 검토 — Task #1143: external image bytes injection contract

- **작성일**: 2026-05-31
- **PR**: #1185 (OPEN)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터 — #1175/#1174/#1163/#1019 머지 등 이미지/PageLayerTree 영역)
- **연결 이슈**: #1143 (Parent #1141, Follow-up #1142/PR #1175)
- **base/head**: `devel` ← `local/task1143`
- **mergeable**: MERGEABLE / mergeStateStatus: **CLEAN** (충돌 없음, head 최신)
- **규모**: 9 파일, +727 / −119 (그 중 tests/issue_1143.rs +515 신규)
- **CI**: 전부 pass (Build & Test / Canvas visual diff / Analyze ×3 / CodeQL). WASM 만 skip.
- **마일스톤**: v1.0.0 / 라벨: enhancement
- **커밋 구조**: 4 stage (cache invalidation → key API → replay diagnostics → regression tests) + devel merge

## 1. PR 정보 확인

#1142(PR #1175, discovery contract)의 후속. discovery 된 external image reference 에
대응하는 bytes 를 core document state 에 주입하는 contract 를 고정.
책임 경계: consumer(파일/권한/경로/bytes 준비) → core(discovery key 로 bin_data 갱신)
→ renderer backend(주입된 state 와 resolved payload 를 replay). renderer 별 파일 탐색
분기를 추가하지 않음.

## 2. 변경 내용 검토

### (1) `src/model/document.rs` (+125) — helper 3개 + 리팩토링
- `external_image_loaded(bin_data_id)`: 렌더러와 동일한 **index-first(`bin_data_id-1`)** 조회,
  fallback 으로 id 검색. wasm_api 중복 로직을 model 로 끌어올려 SoT 단일화.
- `inject_external_image_data()`: index 위치 우선 갱신, 없으면 push. 반환값으로 호출자에게
  캐시 무효화 책임 위임.
- `update_external_image_display_path()`: Picture / Shape(Picture) 모두 순회.
- `populate_external_images_from_dir()`: `Vec`→`BTreeMap` 으로 변경 → **동일 id 중복 로드 방지**
  (idempotent). 인라인 로직을 helper 호출로 치환. 동작 동일.
- [x] index-first 규칙이 renderer lookup(utils.rs:23)과 일치
- [x] 리팩토링 전후 동작 동일 (중복 제거 + helper 추출)

### (2) `src/wasm_api.rs` (+152) — 신규 key API + 리팩토링
- `parse_external_image_key("binData:N")` → bin_data_id (0 거부).
- **`injectExternalImageByKey(key, data, displayPath)`** 신규 — discovery key 로 정확한 reference 지정.
- 기존 basename `injectExternalImage(...)` 는 동일 helper(`inject_external_image_by_bin_data_id`)
  재사용 + `Vec`→`BTreeSet`. **하위 호환 유지**.
- 두 API 모두 `injected>0` 일 때 `invalidate_page_tree_cache()`.
- 중복 `external_image_loaded` 제거 → `document.external_image_loaded()` 위임.
- [x] 신규 API 가 idempotent (이미 loaded 면 0 반환)
- [x] basename API source compatibility 보존

### (3) `src/renderer/canvaskit_policy.rs` (+44) — replay 진단 정교화
- `image_has_replayable_payload()` 신규: `resolved.is_some() || image.data 비어있지 않음`.
- `image_can_replay_directly`: **external_path 존재만으로 direct replay 차단하던 조건 제거**
  → 실제 payload 준비 여부로 판단. injected external image 도 direct replay 가능.
- detail: `externalImage;injectedImageData` vs `externalImage;missingImageData` 구분.
- [x] payload 없으면 여전히 missing 으로 차단 (회귀 없음)
- [x] 테스트 2건으로 direct/missing 동작 고정

### (4) `document_core/commands/document.rs` + layout 4파일 (+6)
- `populate_external_images_from_dir`: loaded>0 일 때 cache 무효화.
- layout.rs / paragraph_layout / picture_footnote / shape_layout: 각 picture 경로에서
  `external_path` 를 ImageNode 까지 전달. **4개 picture 경로 일관 적용**
  (feedback_image_renderer_paths_separate 준수).

### (5) `tests/issue_1143.rs` (+515 신규) — 회귀 11건
- HWP3 sample10 / HWP5 링크 변환 / HWPX sample10 key 기반 injection roundtrip.
- discovery loaded 상태, legacy basename API, PageLayerTree mime/base64, CanvasKit replay plan.
- 기존 fixture 재사용(hwp3-sample10, oracle.gif, rdb02.gif, s1.jpg) — 신규 파일 없음.

## 3. 위험 평가

- **낮음.** 신규 opt-in API 추가 + 기존 동작 보존 리팩토링. canvaskit_policy 변경은
  injected payload 를 direct 로 인정하는 정확한 완화이며, payload 없으면 여전히 missing.
- **clippy `--all-targets` baseline 실패(비차단)**: `cargo clippy --all-targets -- -D warnings`
  가 EXIT 101 로 실패하나, **PR 적용 전 devel 에서도 동일하게 실패**(baseline). 실패 항목은
  `table_layout.rs` / `wasm_api/tests.rs` / `integration_tests.rs` 등 PR 이 손대지 않은
  기존 테스트 코드의 린트이며, **PR 변경 파일에서 나온 clippy 항목은 0 건**. PR 본문도
  `cargo clippy -- -D warnings`(lib 범위)로 검증 — 그 범위는 통과. 별도 lint 정리 이슈 대상.

## 4. 검증 결과 (로컬, 머지 시뮬레이션)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| clippy(lib) | PR 변경 파일 clippy 항목 | ✅ 0 건 (PR 본문 검증 범위 통과) |
| clippy(--all-targets) | `cargo clippy --all-targets -- -D warnings` | ⚠️ EXIT 101 — **devel baseline 도 동일 실패**, PR 무관 |
| 대상 | `cargo test --test issue_1143` | ✅ 9 passed, 0 failed |
| 전체 | `cargo test --tests` | ✅ 98 그룹, 1836 passed, 0 failed |

## 5. 판단 (예정)

전체 테스트 통과 시 **머지** 권고. 결과는 `pr_1185_report.md` 에 기록.
