# Task #705 구현 계획서

## 코드 수준 결함 위치 재측정

### 결함 #1 — `src/renderer/pagination/engine.rs:519-544`

```rust
for (pi, para) in paragraphs.iter().enumerate() {
    for (ci, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            ...
            Control::PageHide(ph) => {
                page_hides.push((pi, ph.clone()));
            }
            ...
        }
    }
}
```

**결함**: 본문 paragraph 만 순회. `Control::Table(table)` 의 `table.cells[].paragraphs[]` 안의 PageHide 미수집.
(메인테이너가 지적한 line 516-531 는 `collect_header_footer_controls` 의 함수 시그니처 + 시작부; 실제 본문 paragraph 순회 본체는 519-544.)

### 결함 #2 — `src/renderer/layout.rs:411,414`

```rust
self.build_page_background(&mut tree, layout, page_border_fill, styles, bin_data_content);
self.build_page_borders(&mut tree, layout, page_border_fill, styles);
```

**결함**: 두 함수 호출 전 `page_content.page_hide.as_ref().map(|ph| ph.hide_fill/hide_border)` 가드 없음. (master_page/header/footer 는 :417, :427 등에서 가드 적용됨, fill/border 만 누락.)

### 결함 #3 — `src/main.rs:1644-1667` (dump 셀 안 controls)

```rust
for (ci, ctrl) in cp.controls.iter().enumerate() {
    match ctrl {
        Control::Picture(p) => { ... }
        Control::Shape(s) => { ... }
        _ => {}
    }
}
```

**결함**: `Control::PageHide(ph)` 분기 없음 → 셀[167] p[3] PageHide 디버깅 시 dump 에 표시 안 됨.

## Stage 별 구현 세부

### Stage 0 — 사전 측정 + 174 샘플 재조사

**산출물**: `mydocs/working/task_m100_705_stage0.md`, `examples/inspect_705.rs`, `examples/scan_cell_pagehide.rs`

1. `PageHide` 모델 정의 확인 (`src/model/control.rs` 의 6 필드 이름 + 타입 — `hide_fill`/`hide_border`/`hide_page_num` 등 정확한 식별자)
2. aift.hwp page 2 의 셀[167] p[3] PageHide raw 측정 — `examples/inspect_705.rs` 작성
   - 본 환경 파서로 셀 안 paragraph 재귀 순회 (PR #640 의 H2 측정 재실행)
   - 6 필드 모두 true 확인
3. 174 샘플 전수 조사 — `examples/scan_cell_pagehide.rs` 작성
   - 모든 샘플의 본문 + 셀 안 paragraph 의 PageHide 컨트롤 검출
   - 셀 안 PageHide 분포 매트릭스 출력
4. 결과 정리: 영향 범위 + 신규 케이스 (Stage 1 테스트 추가 후보)

### Stage 1 — RED (결함 재현 통합 테스트 작성)

**산출물**: `mydocs/working/task_m100_705_stage1.md`

`src/renderer/layout/integration_tests.rs` 에 추가:

1. `test_705_aift_page2_cell_pagehide_hides_page_number` — aift.hwp 페이지 2 footer 글리프 0
2. `test_705_aift_page3_cell_pagehide_hides_page_number` — 페이지 3 동일
3. `test_705_aift_page2_cell_pagehide_hides_border` — 페이지 2 쪽 테두리 미렌더
4. `test_705_aift_page2_cell_pagehide_hides_fill` — 페이지 2 페이지 배경 미렌더
5. Stage 0 의 174 샘플 조사에서 신규 케이스 발견 시 추가

테스트 패턴: PR #641 의 `test_639_*` 참고 — SVG footer 영역 글리프 카운트.

**기대**: 모두 RED.

### Stage 2 — GREEN 결함 #1 정정 (engine.rs)

**산출물**: `mydocs/working/task_m100_705_stage2.md`

`src/renderer/pagination/engine.rs:504-547` 의 `collect_header_footer_controls` 수정:

1. 본문 paragraph 순회 안 `Control::Table(table)` 매칭 추가
2. `table.cells[].paragraphs[].controls[]` 재귀 순회로 `Control::PageHide` 수집 — 헬퍼 함수 `collect_pagehide_in_table(&Table) -> Vec<PageHide>` 분리
3. `page_hides.push((pi, ph.clone()))` 시 외부 paragraph index `pi` 사용 (페이지 매핑 정합성)
4. 중첩 표 (셀 안 표) 처리 — Stage 0 데이터로 판단 후 적용

**기대**: Stage 1 의 page_num 테스트 (test1, test2) PASS.

### Stage 3 — GREEN 결함 #2 정정 (layout.rs)

**산출물**: `mydocs/working/task_m100_705_stage3.md`

`src/renderer/layout.rs:411,414` 수정:

```rust
let hide_fill = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_fill).unwrap_or(false);
if !hide_fill {
    self.build_page_background(&mut tree, layout, page_border_fill, styles, bin_data_content);
}

let hide_border = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_border).unwrap_or(false);
if !hide_border {
    self.build_page_borders(&mut tree, layout, page_border_fill, styles);
}
```

(필드명은 Stage 0 에서 확인한 정확한 식별자로.)

**기대**: Stage 1 의 border/fill 테스트 (test3, test4) PASS.

### Stage 4 — 결함 #3 정정 (dump, main.rs)

**산출물**: `mydocs/working/task_m100_705_stage4.md`

`src/main.rs:1644-1667` 의 셀 안 controls 매칭에 `Control::PageHide(ph)` 분기 추가:

```rust
Control::PageHide(ph) => {
    println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
        indent, ci, ph.hide_header, ph.hide_footer, ph.hide_master_page,
        ph.hide_border, ph.hide_fill, ph.hide_page_num);
}
```

검증:
1. `cargo build --release`
2. `rhwp dump pdf/hwpx/aift.hwpx -s 0 -p 1` → 셀[167] p[3] PageHide 6 필드 출력 확인
3. (HWP5) `rhwp dump samples/aift.hwp -s 0 -p 1` 동일

**기대**: dump 출력에 PageHide 6 필드 모두 표시.

### Stage 5 — 회귀 검증 + 최종 보고

**산출물**: `mydocs/working/task_m100_705_stage5.md`, `mydocs/report/task_m100_705_report.md`

1. `cargo test --release --lib` (1142+ tests) 0 fail 확인
2. `cargo clippy --release` warning 0 확인
3. PR #638 회귀 가드 8건 (`test_634_*`) 모두 PASS 확인
4. aift.hwp 페이지 1, 4, 5, 6 표시/미표시 정합 확인 (`rhwp export-svg pdf/hwpx/aift.hwpx -p {N}`)
5. 174 샘플 페이지 카운트 무변화 확인
6. Stage 0 의 174 샘플 조사에서 발견된 신규 셀 안 PageHide 페이지 검증
7. 최종 보고서 작성 (변경 영역 / 회귀 위험 / 메모리 룰 정합)

## 위험 평가 + 완화

| 항목 | 영향 | 완화 |
|------|------|------|
| 셀 안 PageHide 재귀에서 중첩 표 (depth 2+) 처리 | Stage 0 데이터로 판단 | Stage 2 의 헬퍼 함수 (`collect_pagehide_in_table`) 재귀 호출 |
| 결함 #2 (border/fill) 적용으로 다른 페이지 영향 | 회귀 위험 | Stage 5 의 PR #638 회귀 가드 8건 + 174 샘플 sweep |
| 외부 `pi` 사용 시 페이지 매핑 정합 깨질 가능 | 페이지 정합 결함 | Stage 1 의 RED 테스트 + Stage 5 의 페이지 카운트 무변화 확인 |

## 산출물 매트릭스

- `mydocs/plans/task_m100_705.md` — 수행 계획서 (작성 완료)
- `mydocs/plans/task_m100_705_impl.md` — 본 문서
- `mydocs/working/task_m100_705_stage{0..5}.md` — 단계별 보고서
- `mydocs/report/task_m100_705_report.md` — 최종 보고서
- `examples/inspect_705.rs` — 셀 안 PageHide 측정 도구
- `examples/scan_cell_pagehide.rs` — 174 샘플 전수 조사 도구
- `src/renderer/pagination/engine.rs` — 결함 #1 정정
- `src/renderer/layout.rs` — 결함 #2 정정
- `src/main.rs` — 결함 #3 정정 (dump)
- `src/renderer/layout/integration_tests.rs` — 회귀 가드 신규 4+건
