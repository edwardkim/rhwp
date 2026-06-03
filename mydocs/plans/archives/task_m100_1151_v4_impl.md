# Task #1151 v4 구현계획서

수행계획서: [task_m100_1151_v4.md](task_m100_1151_v4.md)

## 0. 설계 결정

### 0-1. Fix #1 — `rendering.rs:1495` Image JSON 에 cellIdx 추가

`Rectangle` (line 1524-1530) 의 패턴 그대로 차용:

```rust
RenderNodeType::Image(image_node) => {
    // ... 기존 hf_str / doc_coords / wrap_str
    // [Task #1151 v4] 셀 안 inline picture 의 cellIdx / cellParaIdx 전달
    let cell_str = match (image_node.cell_index, image_node.cell_para_index) {
        (Some(cei), Some(cpi)) => format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi),
        _ => String::new(),
    };
    controls.push(format!(
        "{{\"type\":\"image\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}{}{}{}}}",
        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
        doc_coords, wrap_str, hf_str, cell_str
    ));
}
```

### 0-2. Fix #2 — ImageNode 생성 시 cell_index 설정

ImageNode 생성 4 곳:

1. `paragraph_layout.rs:2688` (inline tac picture, text run 처리 중)
2. `paragraph_layout.rs:3067` (run 범위 밖 inline tac)
3. `paragraph_layout.rs:3215` (빈 paragraph 의 inline tac, cell_ctx.is_none() 분기)
4. `layout.rs:4640` (Task #347 fallback)

각 위치에서 `cell_ctx.as_ref()` 의 path 첫 entry 로부터 cell_index / cell_para_index 추출:

```rust
RenderNodeType::Image(ImageNode {
    section_index: Some(section_index),
    para_index: Some(para_index),
    control_index: Some(tac_ci),
    // [Task #1151 v4] cell context 보존
    cell_index: cell_ctx.as_ref().and_then(|c| c.path.last().map(|e| e.cell_index)),
    cell_para_index: cell_ctx.as_ref().and_then(|c| c.path.last().map(|e| e.cell_para_index)),
    crop,
    // ...
})
```

`cell_ctx.path` 의 정확한 구조는 `CellContext` 정의 (cursor_nav.rs 또는 render_tree.rs) 에서 확인하고 결정.

`layout.rs:4640` 의 경우 cell_ctx 변수가 같은 이름인지 확인 필요. 다른 path 라면 본 위치는 본문 picture 만 거치므로 변경 불요 (검토 후 결정).

### 0-3. Fix #3 — `cursor_rect.rs:1218-1223` 셀 안 picture hit-test 처리

기존 코드:
```rust
for (key, &(sx, sy)) in tree.inline_shape_positions() {
    let (si, pi, ci, ref cell_path) = *key;
    if !cell_path.is_empty() {
        continue;
    }
    // ... hit-test
}
```

변경 방향:
```rust
for (key, &(sx, sy)) in tree.inline_shape_positions() {
    let (si, pi, ci, ref cell_path) = *key;
    // [Task #1151 v4] 셀 안 inline picture 도 hit-test. cell_path 가 있으면
    // 셀 컨텍스트와 함께 응답에 포함하여 studio 측에서 picture 속성 대화상자에
    // cellPath 를 전달할 수 있게 한다.
    if let Some(section) = self.document.sections.get(si) {
        let target_para = if cell_path.is_empty() {
            section.paragraphs.get(pi)
        } else {
            // 셀 안 inline shape: cell_path 의 끝 entry 가 picture 가 있는 셀 paragraph
            resolve_cell_paragraph(section, pi, cell_path)
        };
        if let Some(para) = target_para {
            // ... hit-test (기존 로직)
            // 응답에 cell_path 포함
        }
    }
}
```

`resolve_cell_paragraph` 같은 helper 가 이미 있는지 확인 (cursor_nav.rs / cell_path helper). 없으면 신설.

### 0-4. 단위 테스트

`src/document_core/queries/cursor_rect.rs` 또는 별도 mod 에 `#[cfg(test)] mod issue_1151_v4_cell_picture_click_tests`:

| 테스트 | 시나리오 | 단언 |
|--------|---------|------|
| `cell_picture_inline_shape_position_registered` | tac-img-02.hwp 파싱 + render → inline_shape_positions HashMap 에 셀 안 picture 의 cell_path 가 포함된 entry 존재 | OK |
| `cell_picture_hit_test_returns_cell_context` | 셀 안 picture 의 좌표로 cursor_rect query → 응답에 cellPath 포함 | OK |
| `body_picture_hit_test_unchanged` | 본문 inline picture hit-test → cell_path 빈 채로 응답 (회귀 0) | OK |
| `cell_floating_picture_outside_inline_path` | 셀 floating picture (v1 path) → inline_shape_positions 에 등록 안 됨, 기존 path 그대로 | OK |

### 0-5. 통합 시각 / 클릭 테스트 (Stage 7)

- tac-img-02.hwp 의 표 안 picture 좌표 추정 (export-svg 또는 render tree 분석).
- dev server 에서 그 좌표 클릭 → picture select 모드 진입 확인 (사용자 시각).
- v3 의 4 시나리오 회귀 (오버랩 없음 그대로).

---

## Stage 6 — Fix #1/#2/#3 + 단위 테스트

### 6-1. Fix #1: rendering.rs

`src/document_core/queries/rendering.rs:1495-1499` 의 Image JSON 분기에 cell_str 추가. 단위 테스트는 통합 테스트로 흡수 (JSON 출력 grep).

### 6-2. Fix #2: ImageNode cell_index 설정

4 곳의 ImageNode 생성 갱신:

```rust
cell_index: cell_ctx.as_ref().and_then(|c| c.path.last().map(|e| e.cell_index)),
cell_para_index: cell_ctx.as_ref().and_then(|c| c.path.last().map(|e| e.cell_para_index)),
```

cell_ctx 의 정확한 타입 (CellContext, &CellContext, Option<&CellContext>) 과 path entry 의 필드명은 Stage 6-2 의 첫 작업으로 확인하고 시그니처 결정.

### 6-3. Fix #3: cursor_rect.rs 셀 안 picture hit-test

`for` 루프 안의 `if !cell_path.is_empty() { continue; }` 제거 + 셀 안 picture 의 model 위치 (cell.paragraphs[cell_para_index].controls[ci]) 접근 + 기존 hit-test 로직 + 응답에 cell_path 포함.

응답 JSON 형식은 기존 cursor_rect 의 cell_path 포함 응답 패턴 (cell hit 등) 차용.

### 6-4. 단위 테스트

위 0-4 의 4 케이스 작성.

### 6-5. 검증

```bash
cargo test --lib issue_1151_v4
cargo test --lib              # 전수 회귀
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
```

GREEN 확인 후 Stage 6 commit (소스/문서 분리).

---

## Stage 7 — WASM 재빌드 + dev server 재시연 + 시각·클릭 검증

### 7-1. WASM 재빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

### 7-2. dev server 시작

```bash
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700
```

### 7-3. 클릭 시연 시나리오

1. tac-img-02.hwp 열기 → 표 안 inline picture 위 클릭 → select 모드 진입 확인.
2. picture select 후 우클릭 → "개체 속성" → 그림 속성 대화상자에 cellPath / cellIdx 전달 확인.
3. v3 의 4 시나리오 시각 회귀 (toggle 결과 정합 그대로).
4. v1 의 셀 floating picture 클릭 회귀.
5. 본문 inline picture 클릭 회귀.

### 7-4. Stage 7 commit

소스 변경 없음 (시각 검증만). Stage 7 보고서만 commit.

---

## Stage 8 — 통합 PR + 최종 보고서

### 8-1. 자동 회귀

```bash
cargo test --lib
cargo test --tests
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
cd rhwp-studio && npx tsc --noEmit
```

### 8-2. picture-props-dialog.ts 주석 갱신

`rhwp-studio/src/ui/picture-props-dialog.ts:2156` 주석을 v2+v3+v4 모두 반영하도록 갱신.

### 8-3. 통합 PR 발행

```bash
git push origin local/task1151
gh pr create --repo edwardkim/rhwp \
  --base devel \
  --head johndoekim:local/task1151 \
  --title "Task #1151: 표 + picture 한컴 정합 (삽입 + 토글 + 시각 + 클릭)" \
  --body "..."
```

PR body: closes #1151 + v1/v2/v3/v4 scope 합산 + 검증 결과.

### 8-4. 최종 보고서

`mydocs/report/task_m100_1151_v4_report.md` — Task #1151 전체 합산 (v1+v2+v3+v4 결함 분석, fix, 검증, 회귀 가드, 산출물).
