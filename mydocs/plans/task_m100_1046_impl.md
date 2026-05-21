# 구현계획서 — #1046: 본문 하단 overflow 항목 다음 페이지 이월 (사후 reflow)

- 타스크: #1046 (M100)
- 브랜치: `local/task1046` (base `b61a022e`)
- 수행계획서: `task_m100_1046.md` (승인 완료)
- 작성일: 2026-05-21

## 설계 개요

페이지네이션은 단일 전방향 패스(`TypesetEngine::typeset_section`)로 페이지를 만들고,
레이아웃(`build_render_tree`→`build_single_column`)이 페이지별로 실측 배치하며
overflow를 `layout.rs:2529`에서 사후 진단만 기록한다(이월 없음).

**사후 reflow**: paginate → 전 페이지 레이아웃으로 overflow 수집 → overflow 항목 앞에
강제 페이지나눔 hint 추가 → 재paginate → 수렴까지 반복.

### 핵심 불변식 (안전성)

- **hint가 비면 출력 100% 불변** → overflow 없는 정상 문서/골든 SVG는 영향 0.
- **수렴·무한루프 가드**: overflow 항목이 **그 페이지의 첫 항목(위에 다른 항목 없음)**
  이면 이월해도 새 페이지에서 또 넘침 → "본문보다 큰 단일 항목(page-larger)"이므로
  **hint 미추가**. 따라서 (a) 무한루프 방지, (b) page-larger를 자연히 범위 외 처리.
  → 이월이 의미 있는 건 "위에 다른 항목이 있어 자리만 모자란" 경우뿐.
- `MAX_REFLOW_ITER`(예: 5) 상한으로 이중 안전.

### 호출 관계 (확인됨)

- `paginate()` (rendering.rs:1417)가 `self.pagination` 캐시 생산. 명시 호출만(문서 로드
  mod.rs:210, 편집 후). `build_page_tree`는 캐시를 읽을 뿐 paginate 미호출 → reflow
  측정 패스에서 layout 호출해도 **재귀 없음**.

## 단계별 구현 (5단계)

### Stage 1 — overflow 16건 분류 + 키 확정 (코드 변경 전 진단)

- `dump-pages`로 16개 overflow 페이지의 항목 순서 확인.
- 각 항목을 분류:
  - **(A) 이월 대상**: 위에 다른 항목이 있는 경우 (예: 28쪽 pi=242 — 위에 pi=238~241).
  - **(B) page-larger**: 그 페이지 첫 항목인데 overflow (예: pi=567 856.7px) → 범위 외.
- force-break hint 키는 **`para_index`** (분할 표/문단도 그 문단의 첫 청크 시작 = 문단
  시작에 강제나눔). PartialTable continuation(첫 항목)이 overflow면 (B)로 분류.
- 산출물: `task_m100_1046_stage1.md` (분류표 + 예상 해소/잔존 건수).

### Stage 2 — 페이지네이터 force-break-before 훅 (순수 메커니즘)

- `src/renderer/typeset.rs`:
  - `typeset_section`에 `force_break_before: &HashSet<usize>`(para_index) 파라미터 추가.
  - 문단/표 배치 직전(`typeset_paragraph` 등 진입부): `para_idx ∈ force_break_before`
    && 현재 페이지/단에 이미 항목 존재(`!st.current_items.is_empty()`) →
    `advance_column_or_new_page` 선행.
- `src/renderer/pagination/engine.rs` (Paginator fallback): 동일 시그니처 파라미터만
  추가(빈셋 무동작) — 두 엔진 API 정합 유지.
- `src/document_core/queries/rendering.rs`: 호출부에 빈셋 전달(이 단계는 동작 불변).
- 검증: 빈셋 시 `cargo test --release` 회귀 0. 새 단위테스트(hint 1개 → 해당 문단이 새
  페이지 시작).
- 산출물: `task_m100_1046_stage2.md`.

### Stage 3 — DocumentCore reflow 루프

- `src/document_core/queries/rendering.rs`:
  - `paginate()`를 `paginate_once(force_breaks)` + `paginate()`(reflow 래퍼)로 분리,
    또는 `paginate()` 말미에 reflow 루프 추가:
    1. 1차 paginate (빈 hint).
    2. 전 페이지 측정 패스: `build_page_tree` 호출 → `layout_engine.take_overflows()`
       누적 (섹션별 para_index 매핑).
    3. overflow 중 "첫 항목 아님"만 `force_break_before[sec]`에 추가.
    4. 추가분 없으면 종료. 있으면 hint 갱신 + 캐시 무효화 후 재paginate. iter ≤ MAX.
  - 측정 패스가 page tree 캐시를 더럽히지 않도록 루프 후 `invalidate_page_tree_cache()`.
  - "첫 항목 여부"는 overflow의 `column_index`+해당 페이지 items로 판정(첫 item의
    para_index == overflow para_index 면 page-larger).
- 검증: 대상 샘플 overflow 재측정, 무한루프 없음(iter 로그).
- 산출물: `task_m100_1046_stage3.md`.

### Stage 4 — 검증 (대상 샘플 + 회귀)

- `export-svg` overflow 재측정: 28/35/49쪽(및 대응) 해소, 잔여 = page-larger류만.
- `cargo build/clippy --release` 무경고, `cargo test --release` 0 failed.
- 골든 SVG: 빈셋 무동작 설계상 회귀 무 기대. 변동 시 한컴 2022 PDF 대조로 정당성 판정.
- 페이지 수 증가(이월 정상)는 `pdf/...-2022.pdf` 대조.
- 산출물: `task_m100_1046_stage4.md`.

### Stage 5 — WASM 정합 + 최종 보고

- `cargo check --lib --target wasm32-unknown-unknown` OK (reflow는 순수 Rust, WASM 동일).
- WASM 빌드(Docker) 후 스튜디오에서 사용자 보고 페이지 육안 확인(가능 범위).
- `task_m100_1046_report.md` 최종 보고서 + `orders/` 갱신.

## 리스크 / 대응

| 리스크 | 대응 |
|--------|------|
| reflow가 정상 페이지 흔듦 | hint 비면 불변 — overflow 페이지만 영향. 골든/테스트 가드 |
| 무한루프 | "첫 항목" 가드 + MAX_REFLOW_ITER |
| 측정 패스 성능 | reflow는 overflow 있을 때만 반복, 보통 1~2회. 캐시 재사용 |
| page-larger 미해소 | 범위 외 명시(별도 이슈), 잔여로 보고 |
| engine.rs fallback 미정합 | 시그니처만 추가(빈셋 무동작), reflow는 typeset 경로 우선 |

## 범위 외 (재확인)

- 본문보다 큰 단일 항목의 내부 분할(page-larger). 별도 이슈.
- 페이지네이터↔렌더러 측정 bit 정합(#1022 잔여).
