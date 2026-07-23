# M100 #2308 구현 계획서

## 메타데이터

- 이슈: [#2308](https://github.com/edwardkim/rhwp/issues/2308)
- 수행 계획: `mydocs/plans/task_m100_2308.md`
- 브랜치: `issue-2308-render-normalized-derived-state`
- 최초 기준선: `upstream/devel@12f8a820c82e34cbc61042df4b613532b8459a37`
- 구현 기준선: `upstream/devel@29b5547e256a3d6a1f8c94c9434c14a351b5543a`
- 작성일: 2026-07-22
- 승인일: 2026-07-23
- 상태: 구현 진행중

## 1. 구현 불변식

1. 원본 `Paragraph`, `ComposedParagraph`, `Table`, `Cell`을 렌더 정규화 캐시에 복제하지 않는다.
2. 정규화 결과는 원본 IR과 revision으로부터 언제든 재생성할 수 있어야 한다.
3. 캐시 hit는 동일 revision에서만 허용하고, invalid entry를 stale fallback으로 사용하지 않는다.
4. 렌더링·조판 소비부는 source IR과 `RenderNormalizationView`를 조합해 effective 값을 읽는다.
5. 저장·undo/redo가 관찰하는 원본 IR은 정규화 전후로 동일해야 한다.
6. `DocumentCore: Send` 계약과 현재 `build_page_render_tree(&self, ...)` 호출 형태를 보존한다.
7. 경로 표현에는 caption sentinel 같은 magic number를 사용하지 않는다.

## 2. 선택 설계

### 2.1 희소 overlay

`render_normalized`의 섹션 복제본을 `RenderNormalizationState`로 교체한다. 상태는 section별
revision ledger와 희소 overlay entry만 소유한다.

```rust
struct RenderNormalizationState {
    document_epoch: u64,
    sections: Vec<SectionNormalizationState>,
}

struct SectionNormalizationState {
    structural_revision: u64,
    page_geometry_revision: u64,
    path_revisions: HashMap<RenderPath, u64>,
    overlay: RefCell<RenderNormalizationOverlay>,
}
```

구현 시 실제 필드명과 컨테이너는 borrow 범위와 성능 측정에 따라 조정할 수 있지만, 캐시가 원본
노드의 owned clone을 보유하지 않는다는 불변식은 바꾸지 않는다.

### 2.2 배제한 대안

- 기존 섹션 clone에 revision만 부착: 동기화·메모리 비용과 mirror 갱신 책임이 남는다.
- mutation마다 전체 overlay 재생성: 안전하지만 영향받지 않은 derived entry 재사용 목표를 충족하지 못한다.
- raw pointer 기반 overlay key: 구조 변경 후 주소 재사용과 생명주기 추론이 어렵다.
- 원본 IR에 effective 렌더 값을 기록: 저장 무결성과 undo/redo 책임을 오염시킨다.

## 3. 논리 경로 계약

특수 경로를 명시적 enum으로 표현한다.

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RenderPathEntry {
    TableCell {
        control_index: usize,
        cell_index: usize,
        paragraph_index: usize,
    },
    TableCaption {
        control_index: usize,
        paragraph_index: usize,
    },
    ShapeTextBox {
        control_index: usize,
        paragraph_index: usize,
    },
    PictureCaption {
        control_index: usize,
        paragraph_index: usize,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RenderPath(Vec<RenderPathEntry>);
```

- top-level section/paragraph index는 캐시 lookup context로 전달한다.
- 중첩 표는 `RenderPathEntry`를 순서대로 추가한다.
- 경로 해석기는 각 단계의 control 종류와 index 범위를 검증한다.
- 해석 실패는 `None`으로 무시하지 않고 invalidation/fallback 정책에 전달한다.

## 4. revision ledger와 dirty scope

```rust
enum RenderNormalizationDirtyScope {
    Document,
    Section {
        section_index: usize,
        reason: SectionDirtyReason,
    },
    Path {
        section_index: usize,
        path: RenderPath,
    },
}
```

### 4.1 revision 의미

- `document_epoch`: 문서 교체, 전체 재초기화처럼 모든 경로가 무효인 변경
- `structural_revision`: 문단/control/cell 삽입·삭제·이동처럼 index 안정성이 깨지는 변경
- `page_geometry_revision`: section/page/column 크기처럼 모든 effective layout 값에 영향을 주는 변경
- `path_revision`: 구조가 안정적인 상태에서 특정 nested paragraph 내용·속성만 바뀐 변경

### 4.2 기본 정책

- 기존 `mark_section_dirty(section_idx)`는 우선 section 범위 normalization 무효화를 동반한다.
- path 범위 API는 #2214에서 특성화된 deferred cell text edit처럼 구조 안정성이 테스트로 입증된
  진입점에만 적용한다.
- revision 증가와 기존 composed/layout dirty 마킹은 하나의 mutation 후처리 함수에서 수행해
  누락 순서를 방지한다.
- 구조 안정성을 판정할 수 없으면 section scope로 승격한다.

## 5. overlay 데이터와 소비 API

overlay는 현재 복제본에서 실제로 달라지는 값만 저장한다.

```rust
struct RenderNormalizationOverlay {
    image_stacks: HashMap<RenderPath, Arc<ImageStackProjection>>,
    nested_table_widths: HashMap<RenderPath, Arc<NestedTableWidthProjection>>,
}

struct ImageStackProjection {
    source_revision: RenderSourceRevision,
    effective_tac_controls: Arc<[usize]>,
    synthetic_lines: Arc<[SyntheticPictureLine]>,
}

struct NestedTableWidthProjection {
    source_revision: RenderSourceRevision,
    effective_width: i32,
}
```

`RenderNormalizationView`는 조판과 renderer가 clone 구현을 알지 못하도록 다음 조회 계약을 제공한다.

```rust
trait RenderNormalizationView {
    fn effective_treat_as_char(&self, path: &RenderPath, control_index: usize) -> Option<bool>;
    fn effective_stacked_picture_lines(&self, path: &RenderPath) -> Option<Arc<[SyntheticPictureLine]>>;
    fn effective_nested_table_width(&self, path: &RenderPath) -> Option<i32>;
}
```

`None`은 override 없음이라는 뜻이다. invalid cache와 derivation 실패는 별도 오류 상태로 구분해
`None`에 섞지 않는다.

## 6. lookup, 재파생, fallback

1. source IR에서 논리 경로를 해석한다.
2. cache entry의 document/section/path revision을 현재 ledger와 비교한다.
3. 모두 일치하면 기존 `Arc`를 반환한다.
4. path revision만 다르면 해당 entry를 재파생하고 원자적으로 교체한다.
5. 경로 해석 또는 구조 revision이 다르면 section overlay를 비우고 현재 source에서 다시 파생한다.
6. section 재파생도 실패하면 명시적 `RenderError`를 반환한다.

금지 사항:

- 경로 불일치 후 이전 entry 반환
- 오류를 삼키는 조기 `return`
- 원본 source 대신 오래된 clone에서 재파생
- 오류 회피를 위한 무조건 전체 문서 cache clear

## 7. mutation 진입점 감사

첫 구현 커밋에서 다음 범주를 표로 작성하고 dirty scope를 지정한다.

- `src/document_core/commands/text_editing.rs`
- `src/document_core/commands/table_ops.rs`
- `src/document_core/commands/clipboard.rs`
- `src/document_core/commands/object_ops/`
- `src/document_core/commands/header_footer_ops.rs`
- `src/document_core/commands/footnote_ops.rs`
- `src/document_core/commands/html_import.rs`
- `src/document_core/queries/rendering.rs`의 재조판 helper

초기 안전값은 section scope다. path scope는 다음 조건을 모두 만족할 때만 허용한다.

- control/cell/paragraph 개수가 바뀌지 않는다.
- 경로상의 control variant가 바뀌지 않는다.
- page geometry가 바뀌지 않는다.
- source paragraph와 composed 상태 갱신 순서가 테스트로 고정돼 있다.

## 8. #2004 이전 절차

1. `compute_render_normalized`의 floating image stack 판정 입력을 순수 derivation 함수로 분리한다.
2. clone paragraph를 N개로 분할하는 대신 synthetic line projection을 생성한다.
3. TAC 강제값은 control index별 override로 저장한다.
4. typeset/height/layout 경로가 projection을 소비하도록 바꾼다.
5. 원본 control의 TAC 값과 paragraph/control 개수가 전후 동일함을 테스트한다.
6. #2004 fixture의 페이지 수, 이미지 순서, line advance를 기존 결과와 비교한다.

## 9. #2195 이전 절차

1. nested table stretch 계산을 source-only 순수 함수로 분리한다.
2. 부모 cell의 가용 폭과 원본 nested table 폭으로 effective width projection을 만든다.
3. TAC 제외 규칙과 빈 줄/빈 cell 규칙을 기존 #2195 동작과 동일하게 보존한다.
4. `height_measurer`와 layout 경로가 동일 view의 effective width를 사용하게 한다.
5. pointer-key `cell_units_cache`와의 결합은 revision 검증 뒤에만 허용한다.
6. #1195, #1891, #1949 결과와 #2214 sibling cache identity를 함께 검증한다.

## 10. 기존 경로 제거 기준

두 projection이 모두 새 view를 통해 소비된 뒤 다음 심볼과 역할을 제거한다.

- `DocumentCore.render_normalized`의 paragraph/composed clone 저장
- `compute_render_normalized`의 `section.paragraphs.clone()`
- `section_render_paragraphs` / `section_render_composed` clone 선택 분기
- `refresh_render_normalized_cell_paragraph_after_edit`
- caption sentinel `65534`에 의존하는 mirror 경로
- `render_normalized` clone을 전제로 한 주석과 테스트 assertion

제거 시 source guard를 추가해 동일 clone/mirror 패턴의 재도입을 막는다. 기존
`tests/issue_2724_passthrough_invalidation_guard.rs`에는 역할을 섞지 않고 #2308 전용 guard를 둔다.

## 11. 파일별 예상 변경

| 파일 | 변경 |
|---|---|
| `src/document_core/mod.rs` | clone 필드를 revision/overlay state로 교체 |
| `src/document_core/queries/rendering.rs` | 경로 해석, derivation, lookup, fallback, 기존 clone 경로 제거 |
| `src/document_core/commands/text_editing.rs` | 안정적인 deferred edit에 path-scope invalidation 연결 |
| `src/document_core/commands/*.rs` | mutation 감사 결과에 따른 section/document invalidation 연결 |
| `src/renderer/height_measurer.rs` | effective nested width와 image projection 소비 |
| renderer/layout 관련 소비 파일 | `RenderNormalizationView` 전달 및 source+overlay 조회 |
| `tests/issue_2308_render_normalized_derived_state.rs` | 새 revision/overlay 계약 통합 테스트 |
| `tests/issue_2308_render_normalized_guard.rs` | clone/mirror 재도입 방지 source guard |
| `mydocs/tech/rendering_engine_design.md` | derived state 소유권·무효화 계약 반영 |
| `mydocs/report/task_m100_2308_report.md` | 구현·검증·성능·시각 근거 기록 |

renderer/layout의 정확한 추가 파일은 Stage 1 call graph를 고정한 뒤 working 문서에 기록한다.

## 12. 테스트 계획

### 12.1 신규 테스트

- 동일 revision의 두 번 렌더에서 overlay `Arc` identity가 유지된다.
- 한 cell text edit 후 대상 path entry만 바뀌고 sibling entry는 유지된다.
- 문단/control/cell 구조 변경 후 section overlay가 재생성된다.
- 표 caption, 도형 textbox, 그림 caption 경로가 sentinel 없이 해석된다.
- 유효하지 않은 경로에서 stale 값을 반환하지 않고 fallback 또는 오류가 발생한다.
- 정규화 전후 save-visible source IR checksum이 동일하다.
- `DocumentCore: Send` compile-time assertion이 유지된다.

### 12.2 focused 회귀

```text
cargo test --test issue_2308_render_normalized_derived_state
cargo test --test issue_2308_render_normalized_guard
cargo test --test issue_2214_page_local_repaint
cargo test --test issue_2004_cell_image_stack_pagination
cargo test --test issue_1195_cell_table_empty_line
cargo test --test issue_1891
cargo test --test issue_1949_giant_cell_render_perf
```

기존 테스트 바이너리 이름이 달라졌으면 `cargo test --test` 목록을 먼저 확인하고 실제 이름으로
working 문서에 고정한다. focused 결과를 공유한 뒤 전체 검증은 저장소 지침에 따라 별도 승인받는다.

## 13. 시각 검증

렌더링·레이아웃 변경이므로 `visual_verification_governance.md`와 `visual_sweep_guide.md`를 따른다.

- OVR: HWP/Hancom 없이 재현 가능한 최소 fixture로 source와 effective overlay를 시각화한다.
- OVL: 변경 전 PDF, 변경 후 PDF, Hancom 기준 PDF를 같은 페이지·배율로 overlay한다.
- 표 셀 경계, 이미지 순서/수직 간격, 중첩 표 폭, 페이지 분할 지점을 수동 판정한다.
- 자동 pixel/geometry metric은 보조 증거로만 사용하고 최종 판정은 사람이 한다.
- 근거 파일 경로와 판정 결과를 완료 보고서에 기록한다.

## 14. 성능·메모리 기준

- 반복 렌더 hit에서 section paragraph clone이 0회여야 한다.
- 단일 안정 경로 편집 시 재파생 entry 수가 대상 경로 범위를 넘지 않아야 한다.
- 구조 변경 시 section 범위 재파생은 허용하지만 document 전체 재파생은 하지 않는다.
- #1949 fixture의 wall time과 peak memory가 기존 기준 대비 유의하게 악화되면 중단한다.
- 구체 임계값은 Stage 1 baseline 측정 후 working 문서에 수치로 고정한다.

## 15. 커밋 단위

1. RED/characterization 및 mutation 감사 문서
2. revision ledger, path, overlay cache core
3. #2004 image stack projection 이전
4. #2195 nested width projection 이전 및 clone/mirror 제거
5. 경로·fallback·재사용 회귀 테스트와 source guard
6. 시각 검증, 기술 문서, 완료 보고서

각 단계가 끝나면 해당 단계 변경을 커밋한 뒤 다음 단계로 넘어간다.

## 16. 중단·재승인 조건

- source IR 변경 없이 정확한 layout을 만들 수 없음
- `build_page_render_tree(&self, ...)` 또는 `DocumentCore: Send` 계약 변경 필요
- 공개 API, 직렬화 포맷, undo/redo schema 변경 필요
- #2004/#2195 권위 fixture의 의도적 출력 변경 필요
- 전체 문서 캐시 폐기 외에는 correctness를 확보할 수 없음

## 17. 구현 완료 판정

- clone/mirror 저장과 갱신 심볼이 제거되고 source guard가 통과한다.
- 모든 cache hit가 revision 검증을 거친다.
- path/structure mismatch에서 stale 결과가 사용되지 않는다.
- focused 회귀와 승인된 전체 검증이 통과한다.
- OVR/OVL 시각 근거와 성능 비교가 완료 보고서에 포함된다.
- 이슈 코멘트에 구현 결과와 검증 근거를 남길 준비가 된다.

이 계획은 2026-07-23 사용자 승인을 받았으며 Stage 1 특성화부터 구현을 진행한다.
