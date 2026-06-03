# Task #1151 v2 구현계획서

수행계획서: [task_m100_1151_v2.md](task_m100_1151_v2.md) · 한컴 동작: [hancom_picture_tac_toggle.md](../tech/hancom_picture_tac_toggle.md)

## 0. 설계 결정

### 0-1. helper 시그니처 — `migrate_picture_floating_to_inline`

```rust
/// floating picture → inline migration (한컴 정합, H1).
///
/// [Task #1151 v2] floating picture 의 treat_as_char 가 false→true 로
/// 토글될 때 한컴 산출물 (samples/tac-verify/scenario-{a,b,c,d}-after.hwp)
/// 과 동일한 결과를 만들기 위한 4가지 필드 갱신.
fn migrate_picture_floating_to_inline(
    parent_para: &mut Paragraph,
    pic: &mut Picture,
)
```

비고:
- self 메서드가 아닌 자유 함수로 두고 `DocumentCore` 의 메서드는 호출만. parent_para 와 pic 의 동시 가변 참조가 충돌하지 않도록 호출자가 borrow 분리 책임.
- 반환값 없음 — 실패 case 없음 (모든 picture / paragraph 에 대해 안전한 mutation).

### 0-2. mutation 진입점의 snapshot 정책

`set_picture_properties_native` 의 흐름:

```text
A) section_idx / para_idx / control_idx 유효성 검증
B) picture 의 was_tac (현재 treat_as_char) 를 snapshot
C) apply_picture_props_inner 호출 (treat_as_char 비트 + 기타 속성 갱신)
D) snapshot 비교: was_tac == false && pic.common.treat_as_char == true 이면 →
   D-1) parent_para 의 가변 참조 획득
   D-2) migrate_picture_floating_to_inline(parent_para, pic) 호출
   D-3) parent_para 가 Table sibling 을 갖는 경우 Table.dirty = true
E) section.raw_stream = None (재직렬화 유도)
F) mark_section_dirty(section_idx) → paginate_if_needed()
G) 이벤트 로그 (기존 패턴)
```

### 0-3. parent_para 와 pic 의 동시 가변 borrow

같은 `parent_para.controls[ci]` 에서 pic 을 꺼내려면 split-borrow 필요. 구현 패턴:

```rust
let para = &mut self.document.sections[section_idx].paragraphs[para_idx];
if let Some(Control::Picture(pic_box)) = para.controls.get_mut(control_idx) {
    let pic = pic_box.as_mut();  // &mut Picture
    // 여기서 para 와 pic 둘 다 가변. helper 호출 전 line_segs 만 분리.
    let line_segs = &mut para.line_segs;
    // ...
}
```

대안: helper 의 시그니처를 `(parent_para_line_segs: &mut Vec<LineSeg>, pic: &mut Picture)` 로 좁혀서 borrow 충돌 회피. 더 안전.

**채택**: 후자.

```rust
fn migrate_picture_floating_to_inline(
    parent_line_segs: &mut Vec<LineSeg>,
    pic: &mut Picture,
)
```

### 0-4. baseline 계산

`(seg.line_height as f64 * 0.85) as i32`. 정수 절단 결과 한컴 산출물 값과 1-2 오차 가능 (예: 5331 × 0.85 = 4531.35, 한컴 4531 일치). 단위 테스트 단언에서 절대 오차 허용 (`assert_eq!(seg.baseline, (lh as f64 * 0.85) as i32)`).

### 0-5. `apply_picture_props_inner` 와 migration 의 책임 분리

- `apply_picture_props_inner` 는 JSON 속성을 picture 에 적용 (treat_as_char 비트 + h/v_rel_to + h/v_offset + ...). dialog 에서 사용자가 토글 외 다른 속성 (예: cropLeft) 도 변경했을 수 있으므로 그 부분은 정상 적용.
- migration path 는 **그 이후에** picture 의 h/v_rel_to / h/v_offset 을 강제로 Para / 0 으로 덮어쓴다. 즉 사용자가 dialog 에서 floating geometry 를 동시에 바꿔도 toggle 우선.

### 0-6. paginate / dirty 마킹 정책

- `parent_para` 가 Table sibling 을 갖는 경우 (셀 영역 floating picture 의 일반 case): Table.dirty=true 로 표 layout 재계산 유도. v1 의 cell_path 분기와 동일.
- `mark_section_dirty` + `paginate_if_needed` 호출. 기존 set_picture_properties_native 의 후처리 패턴 그대로.

### 0-7. WASM / TS API

변경 없음. `setPictureProperties(sec, paraIdx, ci, propsJson)` 가 기존 그대로 동작하고 Rust 측에서 migration 을 흡수. dialog 측은 주석만 갱신.

---

## Stage 1 — helper + migration 분기 + 단위 테스트

### 1-1. RED — 단위 테스트 작성

`src/document_core/commands/object_ops.rs` 의 기존 `#[cfg(test)] mod issue_1151_*` 옆에 `mod issue_1151_v2_tac_toggle_tests` 추가.

#### 테스트 1: `tac_toggle_table_sibling_floating_to_inline`

Scenario A 등가. 1×1 표 + 표 sibling floating picture 를 만든 후 `set_picture_properties_native(sec, para_idx, control_idx, r#"{"treatAsChar":true}"#)` 호출.

```rust
#[test]
fn tac_toggle_table_sibling_floating_to_inline() {
    let mut doc = test_helpers::doc_with_1x1_table_and_sibling_floating_picture(
        /*pic_width_hu=*/5977, /*pic_height_hu=*/5331,
    );
    let section_idx = 0;
    let para_idx = 0;
    let control_idx = 3;  // 구역나누기, 단정의, 표, 그림

    let before_paragraph_count = doc.document.sections[section_idx].paragraphs.len();
    let result = doc.set_picture_properties_native(
        section_idx, para_idx, control_idx,
        r#"{"treatAsChar":true}"#,
    );
    assert!(result.is_ok(), "set_picture_properties_native failed: {:?}", result);

    let para = &doc.document.sections[section_idx].paragraphs[para_idx];
    let pic = match &para.controls[control_idx] {
        Control::Picture(p) => p.as_ref(),
        _ => panic!("picture not at expected control_idx"),
    };

    // picture 위치 불변
    assert_eq!(para.controls.len(), 4);  // 구역나누기, 단정의, 표, 그림 그대로
    assert_eq!(doc.document.sections[section_idx].paragraphs.len(), before_paragraph_count);

    // 4가지 필드 갱신
    assert!(pic.common.treat_as_char);
    assert_eq!(pic.common.attr & 0x01, 0x01);
    assert!(matches!(pic.common.horz_rel_to, HorzRelTo::Para));
    assert!(matches!(pic.common.vert_rel_to, VertRelTo::Para));
    assert_eq!(pic.common.horizontal_offset, 0);
    assert_eq!(pic.common.vertical_offset, 0);

    // LINE_SEG[0].line_height = picture height
    let seg = &para.line_segs[0];
    assert_eq!(seg.line_height, 5331);
    assert_eq!(seg.text_height, 5331);
    assert_eq!(seg.baseline, (5331_f64 * 0.85) as i32);

    // text / char_offsets 불변
    assert_eq!(para.text, "");
    assert_eq!(para.char_offsets.len(), 0);
}
```

#### 테스트 2: `tac_toggle_body_floating_to_inline`

Scenario D 등가. 표 없이 본문 floating picture 만.

#### 테스트 3: `tac_toggle_3x3_center_cell_floating_to_inline`

Scenario C 등가. 3×3 표 + 셀 (1,1) 영역 floating picture.

#### 테스트 4: `tac_toggle_when_already_tac_true_no_migration`

이미 tac=true 인 picture 의 다른 속성만 변경 → migration 미진입.

```rust
let before_lh = para.line_segs[0].line_height;
doc.set_picture_properties_native(0, 0, 3, r#"{"brightness":50}"#)?;
assert_eq!(doc.document.sections[0].paragraphs[0].line_segs[0].line_height, before_lh);
```

#### 테스트 5: `tac_toggle_true_to_false_no_migration_this_pr`

tac=true → false 토글 → migration 미진입 (한 방향만).

#### 테스트 6: `tac_toggle_with_empty_line_segs_creates_new_seg`

`line_segs.is_empty()` 인 (드문) parent paragraph → tac false→true → line_segs[0] 신설.

### 1-2. GREEN — helper 신설 + migration 분기

#### helper 추가 (object_ops.rs 내부 free fn)

```rust
fn migrate_picture_floating_to_inline(
    parent_line_segs: &mut Vec<crate::model::paragraph::LineSeg>,
    pic: &mut crate::model::image::Picture,
) {
    use crate::model::shape::{HorzRelTo, VertRelTo};

    pic.common.treat_as_char = true;
    pic.common.attr |= 0x01;
    pic.common.horz_rel_to = HorzRelTo::Para;
    pic.common.vert_rel_to = VertRelTo::Para;
    pic.common.horizontal_offset = 0;
    pic.common.vertical_offset = 0;

    let picture_height_hu = pic.common.height as i32;
    if let Some(seg) = parent_line_segs.first_mut() {
        if seg.line_height < picture_height_hu {
            seg.line_height = picture_height_hu;
            seg.text_height = picture_height_hu;
            seg.baseline = (picture_height_hu as f64 * 0.85) as i32;
        }
    } else {
        parent_line_segs.push(crate::model::paragraph::LineSeg {
            line_height: picture_height_hu,
            text_height: picture_height_hu,
            baseline: (picture_height_hu as f64 * 0.85) as i32,
            line_spacing: 600,
            ..Default::default()
        });
    }
}
```

#### `set_picture_properties_native` 의 migration 분기

```rust
pub fn set_picture_properties_native(
    &mut self, section_idx: usize, para_idx: usize, control_idx: usize, props_json: &str,
) -> Result<(), HwpError> {
    // ... 유효성 검증 (기존)

    let para = &mut self.document.sections[section_idx].paragraphs[para_idx];
    let was_tac = match para.controls.get(control_idx) {
        Some(Control::Picture(p)) => p.common.treat_as_char,
        _ => return Err(HwpError::RenderError("...".into())),
    };

    // mutation: picture 속성 적용
    let now_tac = if let Some(Control::Picture(pic_box)) = para.controls.get_mut(control_idx) {
        let _ = apply_picture_props_inner(pic_box.as_mut(), props_json);
        pic_box.common.treat_as_char
    } else {
        return Err(HwpError::RenderError("...".into()));
    };

    // [Task #1151 v2] floating → inline migration (H1 정합)
    if !was_tac && now_tac {
        let para = &mut self.document.sections[section_idx].paragraphs[para_idx];
        // split borrow: line_segs 와 picture 를 별개로 빌림
        let (line_segs, controls) = (
            &mut para.line_segs as *mut Vec<_>,
            &mut para.controls as *mut Vec<_>,
        );
        // SAFETY: line_segs 와 controls 는 같은 paragraph 의 다른 필드. 별개 영역.
        unsafe {
            if let Some(Control::Picture(pic_box)) = (*controls).get_mut(control_idx) {
                migrate_picture_floating_to_inline(&mut *line_segs, pic_box.as_mut());
            }
        }
        // 표 sibling 인 경우 outer Table.dirty=true
        if let Some(table_idx) = find_outer_table_control_idx(&para.controls) {
            if let Some(Control::Table(t)) = para.controls.get_mut(table_idx) {
                t.dirty = true;
            }
        }
    }

    // 후처리
    self.document.sections[section_idx].raw_stream = None;
    self.recompose_section(section_idx);
    self.paginate_if_needed();
    // 이벤트 로그 (기존)
    Ok(())
}
```

unsafe 가 마음에 걸리면 다른 패턴 (preflight 에서 split 후 helper 호출) 으로 대체. Stage 1-2 의 implementation 단계에서 결정.

#### 보조 helper

`find_outer_table_control_idx(&[Control]) -> Option<usize>`: controls 안에서 첫 Control::Table 의 index 반환.

### 1-3. REFACTOR

- helper signature 재검토.
- unsafe 제거 시도 (split_at_mut 또는 mem::take).
- 주석 정리 (이 단계 결정 기록).

### 1-4. 검증

```bash
cargo test --lib issue_1151_v2_tac_toggle    # 신규 6 케이스
cargo test --lib                              # 전수 회귀
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
```

GREEN 확인 후 Stage 1 commit:
```
Task #1151 v2 Stage 1: floating→inline tac migration helper + 6 unit tests

- migrate_picture_floating_to_inline helper (4 필드 갱신, H1 정합)
- set_picture_properties_native 에 was_tac snapshot + migration 분기
- find_outer_table_control_idx helper
- issue_1151_v2_tac_toggle_tests: 6 tests GREEN
- samples/tac-verify/scenario-{a,b,c,d}-after.hwp 와 model 단언 일치
```

`mydocs/working/task_m100_1151_v2_stage1.md` 작성.

---

## Stage 2 — WASM 빌드 + 브라우저 시각 검증

### 2-1. WASM 빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

### 2-2. rhwp-studio 시각 검증

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &
```

검증 시나리오:

1. 신규 문서 → 입력 → 표 → 1×1 → 셀 클릭 → 입력 → 그림 → 작은 이미지 선택 (Scenario A 등가).
2. 그림 우클릭 → "개체 속성" 또는 "그림 속성" → "글자처럼 취급" 체크 → 확인.
3. 시각 결과 확인:
   - picture 가 표와 겹치지 않음.
   - parent paragraph 의 line 이 picture height 만큼 자람.
   - 표가 picture 만큼 밀려남.
4. 큰 picture 케이스 (Scenario B 등가) 도 동일 확인.
5. 본문 floating picture (셀 없음, Scenario D 등가) → tac 토글.
6. 한컴 산출물 SVG 비교:
   ```bash
   rhwp export-svg samples/tac-verify/scenario-a-after.hwp -o /tmp/sa-after.svg
   ```
   브라우저 결과와 1:1 비교 (시각 / position / size).

### 2-3. 회귀 시나리오

7. 본문 inline picture 신규 삽입 → v1 의 본문 분기 결과 그대로.
8. 셀 안 picture 삽입 (v1 path, cell_path 분기) → floating sibling 결과 그대로.

### 2-4. 검증 후 Stage 2 commit

```
Task #1151 v2 Stage 2: WASM 빌드 + 브라우저 시각 검증 통과

- 작은/큰 picture / 본문 floating / 셀 영역 floating 4 케이스 시각 확인
- 한컴 산출물 (samples/tac-verify/) SVG 와 1:1 비교 일치
- v1 본문 inline / 셀 안 floating 삽입 회귀 0
```

`mydocs/working/task_m100_1151_v2_stage2.md`.

---

## Stage 3 — 자동 회귀 + PR 발행 + 최종 보고서

### 3-1. 자동 회귀

```bash
cargo test --lib
cargo test --tests
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
cd rhwp-studio && npx tsc --noEmit
```

### 3-2. PR 발행

`local/task1151` 브랜치 push:
```bash
git push origin local/task1151
```

신규 PR 생성:
```bash
gh pr create --repo edwardkim/rhwp \
  --base devel \
  --head johndoekim:local/task1151 \
  --title "Task #1151: 셀 영역 picture 삽입 (floating) + 글자처럼 취급 토글 — 한컴 정합" \
  --body "..."
```

PR body 에 다음 포함:
- closes #1151
- v1 (셀 안 floating 삽입) + v2 (toggle migration) scope 묶음
- 한컴 정합 산출물 (`samples/tac-verify/`) 와 model dump 일치 증거
- 단위 테스트 6 + 시각 검증 4 케이스

### 3-3. 최종 보고서

`mydocs/report/task_m100_1151_v2_report.md`:
- 결함 분석 (3-layer)
- 한컴 동작 검증 (Scenario A~D)
- fix 정책 (4 필드 갱신, H1 정합)
- 검증 결과 (단위 + 시각 + 회귀)
- v1 인프라 위에 추가된 변경 요약

### 3-4. 최종 Stage 3 commit + 푸시

```
Task #1151 v2 Stage 3 + 최종 보고서: 회귀 통과 + PR 발행

- cargo test --lib / clippy / fmt 모두 GREEN
- WASM 빌드 + npx tsc --noEmit clean
- 신규 PR 발행: closes #1151
- 최종 보고서 mydocs/report/task_m100_1151_v2_report.md
```

---

## 단계별 보고서 위치

각 Stage 완료 시 `mydocs/working/task_m100_1151_v2_stage{N}.md` 와 함께 커밋. CLAUDE.md "단계별 완료보고서는 해당 단계 소스 커밋과 함께 타스크 브랜치에서 커밋한다" 규칙 정합.
