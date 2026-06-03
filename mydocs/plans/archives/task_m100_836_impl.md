# Task #836 구현계획서 — Endnote(미주) 본문 렌더링

## A안: 문서 끝 (또는 섹션 끝) 단일 영역에 모든 Endnote 모아서 렌더

## 검증 샘플

`samples/issue836/3-09월_교육_통합_2022.hwp` (커밋 제외, 라이센스 제한)
- 46 Endnote, 현재 10페이지
- PDF 권위: `samples/issue836/3-09월_교육_통합_2022-2022.pdf`

## Stage 1: Endnote 수집 — typeset.rs pagination

**목표**: `typeset.rs`에서 `Control::Endnote`를 수집하여 섹션별 목록 보존.

**변경 파일**: `src/renderer/typeset.rs`, `src/renderer/pagination.rs`

**구현**:

1. `pagination.rs`의 `PageContent`에 endnote 목록은 추가하지 않음 — 미주는 페이지별이 아니라 섹션별/문서별이므로 별도 수집.

2. `typeset.rs`의 `PaginationState`에 섹션별 endnote 수집 필드 추가:
```rust
/// 현재 섹션의 미주 목록 (섹션 끝에서 flush)
endnotes: Vec<EndnoteRef>,
```

3. `Control::Endnote` 분기 추가 (현재 `_ => {}` 영역):
```rust
Control::Endnote(en_ctrl) => {
    st.endnotes.push(EndnoteRef {
        number: en_ctrl.number,
        para_index: para_idx,
        control_index: ctrl_idx,
    });
}
```

4. 섹션 종료 시 수집된 endnotes를 마지막 페이지에 전달하거나, 별도 endnote 페이지 생성.

**산출물**: RED 테스트 — Endnote가 수집되는지 확인 (렌더링은 아직 미구현).

## Stage 2: Endnote 영역 렌더링 — layout.rs

**목표**: 수집된 Endnote를 문서 마지막에 렌더.

**변경 파일**: `src/renderer/layout.rs`, `src/renderer/layout/picture_footnote.rs`

**구현**:

1. `build_endnote_area()` 함수 신규 — `build_footnote_area()` 패턴 참조:
   - 구분선 렌더 (`endnote_shape.separator_*` 속성)
   - 각 Endnote의 번호 + 본문 paragraphs 렌더
   - `en.paragraphs` 순회하며 `format_paragraph` → `layout_paragraph` 호출

2. 렌더 위치 결정:
   - 마지막 섹션의 마지막 페이지에서 본문 영역 아래에 배치
   - 공간 부족 시 다음 페이지로 넘김 (pagination 연동)

3. `build_page_render_tree()`에서 endnote area 호출:
   - 마지막 페이지 (또는 endnote 전용 페이지)에서만 `build_endnote_area()` 호출

**참조**: `build_footnote_area()` (layout.rs:1146~1180), `layout_footnote_area()` 패턴.

## Stage 3: Endnote pagination — 여러 페이지에 걸치는 미주

**목표**: Endnote가 많아서 한 페이지에 안 들어갈 때 다음 페이지로 분할.

**변경 파일**: `src/renderer/typeset.rs`

**구현**:

1. 섹션 종료 후 수집된 endnotes의 총 높이 추정
2. 마지막 페이지 잔여 공간에 들어가면 해당 페이지에 배치
3. 안 들어가면 새 페이지 생성하여 배치
4. 한 페이지에 안 들어가는 경우 여러 endnote 페이지 생성

## Stage 4: 회귀 검증 + 시각 판정

**목표**: 기존 fixture 회귀 0 + 시험지 한컴 PDF 정합.

**검증**:
1. `cargo test --release --lib` 전체 통과
2. 기존 SVG fixture 회귀 sweep (aift/exam_kor/exam_math/exam_science 등)
3. `samples/issue836/3-09월_교육_통합_2022.hwp` SVG + 웹 에디터 시각 판정
4. PDF 권위 자료 비교

## 회귀 위험 분석

| 변경 영역 | 위험 | 완화 |
|---|---|---|
| typeset.rs Endnote 수집 | 낮음 — 기존 `_ => {}` 영역 추가만 | 기존 문서에 Endnote 없으면 무영향 |
| layout.rs endnote area | 중간 — 새 렌더 영역 추가 | Endnote 없는 문서에서 호출 안 됨 |
| pagination 페이지 분할 | 높음 — 페이지 수 변경 가능 | 기존 fixture에 Endnote 없으므로 무영향 |

## 예상 코드량

| Stage | 파일 | 예상 LOC |
|---|---|---|
| 1 | typeset.rs, pagination.rs | +30~50 |
| 2 | layout.rs, picture_footnote.rs | +100~200 |
| 3 | typeset.rs | +50~80 |
| 4 | (검증만) | 0 |
| **합계** | | **+180~330** |

## 작업지시자 결정 요청

구현계획서 승인 후 `local/task836` 브랜치에서 Stage 1부터 진행합니다.
