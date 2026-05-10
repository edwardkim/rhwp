---
PR: #730
제목: Task #172 — section.xml 컨트롤 디스패처 표/그림/도형 직렬화 연결
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 3번째 PR)
base / head: devel / contrib/hwpx-control-dispatch
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS
변경 규모: +619 / -37, 5 files (소스 4 + 테스트 1)
검토일: 2026-05-10
---

# PR #730 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #730 |
| 제목 | Task #172 — section.xml 컨트롤 디스패처 표/그림/도형 직렬화 연결 |
| 컨트리뷰터 | @oksure — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 PR #728/#729 후속 영역 3번째) |
| base / head | devel / contrib/hwpx-control-dispatch |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +619 / -37, 5 files |
| 커밋 수 | 5 (Task 2 + fix 2 + feat 1) |
| closes | #172 |

## 2. 결함 본질 (Issue #172)

### 2.1 결함 영역
Task #164 (PR #170) 영역 영역 텍스트 직렬화 영역 영역 완료 영역 영역 — 그러나 `Paragraph.controls` IR (표/이미지/도형 등) 영역 영역 직렬화 부재 영역.

### 2.2 채택 접근
`section.rs::render_run_content() / render_control_slot()` 영역 영역 확장 영역 영역 Issue #172 체크리스트 6 항목 영역 영역 모두 커버:
- [x] 표 (`Control::Table`) → `<hp:tbl>`
- [x] 그림 (`Control::Picture`) → `<hp:pic>` + `BinData`
- [x] 도형 (`Control::Shape`) → 변형별 공통 속성 XML
- [x] 각주/미주 (`Control::Footnote/Endnote`) → `<hp:ctrl><hp:footNote>`/`<hp:endNote>` + `<hp:subList>` 재귀 문단 직렬화
- [x] BinData ZIP 엔트리 (mod.rs 기존 3-way 동기화 활용)
- [x] `content.hpf` manifest 자동 등록 (mod.rs 기존 활용)

## 3. PR 의 정정 — 5 영역

### 3.1 `src/serializer/hwpx/section.rs` (+159/-33)

**컨트롤 디스패처 확장**:
```rust
fn render_control_slot(out: &mut String, control: &Control, ctx: &mut SerializeContext) {
    match control {
        Control::Equation(eq) => out.push_str(&render_equation(eq)),
        Control::Table(tbl) => writer_to_string(|w| table::write_table(w, tbl, ctx)),
        Control::Picture(pic) => writer_to_string(|w| picture::write_picture(w, pic, ctx)),
        Control::Shape(shape) => out.push_str(&render_shape(shape, ctx)),
        Control::Footnote(note) => out.push_str(&render_footnote(note, ctx)),
        Control::Endnote(note) => out.push_str(&render_endnote(note, ctx)),
        _ => {}
    }
}
```

**Equation-only 게이트 정정**: `slots.iter().any(matches!(Equation))` 영역 영역 → `slots.is_empty()` 영역 영역 → Table/Picture/Shape/Footnote/Endnote 모두 직렬화.

**각주/미주**: `<hp:ctrl><hp:footNote>...<hp:subList>...재귀 문단...</hp:subList>` 영역 영역 — 내부 문단 영역 영역 `render_paragraph_parts` 재귀 호출.

**`is_hwpx_inline_slot`** 영역 영역 `Footnote/Endnote` 영역 영역 추가 영역.

**Shape 디스패치** (`render_shape`):
- `Rectangle` / `Line` → Writer-based serializer (기존 `shape::write_rect/write_line`)
- `Picture` → `picture::write_picture` 위임 (Copilot 리뷰 반영)
- `Ellipse` / `Arc` / `Polygon` / `Curve` / `Group` / `Chart` / `Ole` → `render_common_shape_xml` (공통 속성)

### 3.2 `src/serializer/hwpx/mod.rs` (+210/-2)

라운드트립 테스트 4건 신규 — `picture_bindata_roundtrip` / `table_control_roundtrip` / `footnote_endnote_roundtrip` / `tac_img_sample_has_pictures_and_bindata`.

### 3.3 `src/serializer/hwpx/shape.rs` (+122/-2)

`drawText` 글상자 직렬화 추가 — Rectangle 영역 영역 의 텍스트 박스 영역 영역.

### 3.4 `src/serializer/hwpx/context.rs` (+19)

`collect_from_document()` 영역 영역 인라인 Table 영역 영역 `borderFillIDRef` 영역 영역 1-pass 영역 영역 사전 등록 — Table 직렬화 영역 영역 borderFill 영역 영역 의 의존성 정합.

### 3.5 `tests/hwpx_roundtrip_integration.rs` (+109)

신규 테스트 4건:
- `stage5_table_control_preserved_on_roundtrip` (`samples/표-텍스트.hwpx`)
- `stage5_picture_bindata_preserved_on_roundtrip` (`samples/tac-img-02.hwpx`)
- `stage5_footnote_endnote_preserved_on_roundtrip`
- `stage5_tac_img_sample_has_pictures_and_bindata`

## 4. ⚠️ 한컴 호환 검증 한계 점검

### 4.1 자기 라운드트립 ≠ 한컴 호환 (`feedback_self_verification_not_hancom` 영역 정합)

PR 영역 영역 영역 자기 라운드트립 영역 영역 만 입증 — `parse_hwpx → serialize → parse_hwpx → IR 일치` 영역. 한컴 호환 영역 영역 입증 부재.

**관련 트러블슈팅** (`feedback_search_troubleshootings_first` 영역 정합):
- `mydocs/troubleshootings/picture_save_hancom_compatibility.md` — HWP picture 저장 영역 영역 한컴 호환 6 영역 영역 결함 영역 영역 (CommonObjAttr `prevent_page_break` / SHAPE_COMPONENT `ctrl_id` / 렌더링 행렬 / border/crop/extra / PARA_LINE_SEG / attr 비트 영역)
- `mydocs/troubleshootings/task178_first_attempt_failure.md` — HWPX→HWP 어댑터 첫 시도 실패
- `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md` — HWPX→HWP 어댑터 두 번째 시도 영역 한컴 거부 (rhwp 자기 호환 100% 통과 영역 영역 한컴 거부)

### 4.2 본 PR 영역 영역 영역 HWPX → HWPX (XML 직렬화)

본 PR 영역 영역 영역 **HWP (OLE 바이너리) 영역 영역 다름** — HWPX 영역 영역 ZIP+XML 영역 영역 평문 영역. 그러나 한컴 영역 영역 의 엄격성 영역 영역 동일 영역 영역 가능성 영역.

### 4.3 검증 게이트 영역 영역 권장 (DoD)

- 자기 라운드트립 ✅ (PR 영역 영역 4건 신규)
- **한컴2020/한컴2022 영역 영역 의 수동 검증** — 직렬화 산출물 영역 영역 한컴 영역 영역 정상 열림 + 표/그림/각주 영역 영역 정합
- `hancompfx-roundtrip` 영역 영역 의 광범위 sweep — 9 fixture (blank/ref_empty/ref_text/ref_mixed/ref_table/hwpx-02/form-002/2025-q1/2025-q2) 영역 영역 회귀 0

## 5. 본 환경 점검

- merge-base: `60aeaa8d` (5/9 영역 영역 매우 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: `src/serializer/hwpx/` 영역 영역 (4 files) + `tests/` 영역 영역 — 다른 파서/렌더러 영역 영역 무관
- HWPX 변환본 영역 영역 영역 시각 정합 영역 영역 영향 가능성 영역 — 광범위 sweep 영역 영역 점검 필요 (Rust 변경 영역 영역 직렬화 영역 만 영역 영역 시각 출력 무관 영역 영역, 그러나 Document IR 영역 영역 의 영향 영역 영역 영역 영역 영역 점검)

## 6. 영향 범위

### 6.1 변경 영역
- HWPX 직렬화 영역 영역 표/그림/도형/각주/미주 영역 영역 추가 (Issue #172 체크리스트 6 항목)
- BinData ZIP 엔트리 + content.hpf manifest 자동 등록 (기존 인프라 활용)

### 6.2 무변경 영역
- HWP (OLE 바이너리) 직렬화 — 별건
- 파싱 영역 영역 (rhwp parser) — 변경 부재
- 시각 출력 (SVG/PNG) — 변경 부재 (직렬화 영역 만)

### 6.3 위험 영역
- **한컴 호환 영역 영역 입증 부재** — 자기 라운드트립 영역 영역 만 통과
- HWPX 영역 영역 영역 한컴 영역 영역 의 엄격성 영역 영역 점검 필요 (필드 누락/순서/길이 등)
- 작업지시자 한컴 검증 게이트 권장

## 7. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 hwpx 직렬화 모듈 격리 영역 영역 충돌 부재

## 8. Copilot 리뷰 반영 (commit `328b00e5`)

- **에러 로깅** — `eprintln` 영역 영역 조용한 누락 방지
- **Cursor 제거** — `Writer<Vec<u8>>` 영역 영역 직접 영역 (Cursor 불필요)
- **ShapeObject::Picture 위임** — `picture::write_picture` 영역 영역 위임 영역 영역 코드 중복 제거

## 9. 처리 옵션

### 옵션 A — 5 commits cherry-pick + no-ff merge (추천)

PR 영역 영역 5 commits 영역 영역 모두 본질 영역 (Task 2 + fix 2 + feat 1). 모두 cherry-pick.

```bash
git checkout -b local/task172 815d6d6e
git cherry-pick ec0b509e 815759b2 328b00e5 99c8ac6c 7a544375
git checkout local/devel
git merge --no-ff local/task172
```

→ **옵션 A 추천** (PR #729 영역 영역 영역 squash 채택 영역 영역, 본 PR 영역 영역 영역 commit 별 의미 영역 영역 보존 영역 영역 — 별 영역 영역 commit 영역 영역 의 부분 변경 누적 영역 영역 충돌 가능성 영역 점검 필요).

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN (신규 4건 PASS 보장)
- [ ] `cargo test --release --test hwpx_roundtrip_integration` — 라운드트립 14건 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (직렬화 영역 영역 만 변경 영역 영역 무영향 보장)

### 작업지시자 한컴 호환 검증 게이트 — **권장**

본 PR 영역 영역 의 본질 영역 영역 **HWPX 직렬화 산출물 영역 영역 한컴 호환**:

#### A. 자기 라운드트립 (신규 4건 영역 영역 입증)
- `stage5_table_control_preserved_on_roundtrip` (표-텍스트.hwpx)
- `stage5_picture_bindata_preserved_on_roundtrip` (tac-img-02.hwpx)
- `stage5_footnote_endnote_preserved_on_roundtrip`
- `stage5_tac_img_sample_has_pictures_and_bindata`

#### B. 한컴 검증 게이트 (작업지시자 권장)
- 직렬화 산출물 영역 영역 한컴2020/한컴2022 영역 영역 정상 열림 점검
- 표/그림/도형/각주/미주 영역 영역 시각 정합 점검
- 파일 손상 메시지 부재 점검
- `feedback_self_verification_not_hancom` 정합

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 3번째 PR — PR #728/#729 후속) |
| `feedback_self_verification_not_hancom` | **권위 사례** — 자기 라운드트립 4건 ✅ 영역 영역 한컴 호환 입증 부재 영역 영역 작업지시자 검증 게이트 권장 |
| `feedback_search_troubleshootings_first` | **권위 사례** — `picture_save_hancom_compatibility.md` / `task178_first/second_attempt` 영역 영역 검색 영역 영역 한컴 호환 한계 점검 |
| `feedback_image_renderer_paths_separate` | hwpx 직렬화 영역 영역 격리 — 다른 파서/렌더러 영역 영역 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (BinData ZIP / content.hpf manifest) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 5 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo build/test/clippy + 광범위 sweep + hwpx_roundtrip_integration ALL GREEN)
3. **작업지시자 한컴 호환 검증** — 직렬화 산출물 영역 영역 한컴 영역 영역 정상 열림 + 표/그림/각주 정합 점검 (`feedback_self_verification_not_hancom` 정합)
4. 한컴 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #730 close (closes #172 자동 정합)

---

작성: 2026-05-10
