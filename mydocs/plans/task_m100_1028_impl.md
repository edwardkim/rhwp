# Task M100-1028 구현 계획서

- 이슈: [#1028](https://github.com/edwardkim/issues/1028)
- 브랜치: `local/task1028` (base: `local/devel = 20b544f8`)
- Stage 1 보고서: `mydocs/working/task_m100_1028_stage1.md` (승인 2026-05-20)

## 1. 근본 원인 (Stage 1 확정)

**HWPX `parse_draw_text` 의 `<hp:subList>` 처리에서 `textDirection` 속성 누락.** 결과 `text_box.list_attr` bit 0~2 (text_direction) 항상 0 → renderer 의 `shape_layout.rs:1652` `(list_attr & 0x07)` 분기 미발동 → 세로쓰기 미적용.

**HWP/HWP5 는 정상 렌더링** (parser/control/shape.rs 가 list_attr u32 그대로 보존 → bit 0~2 자동 포함). HWPX 만 미구현.

## 2. 구현 범위 (최소 변경)

**대상 파일**: `src/parser/hwpx/section.rs` 단일 파일

**대상 위치**: `parse_draw_text` 의 `b"subList"` attribute 루프 (라인 ~2249)

**변경량**: 약 12 줄 (1 분기 추가)

```rust
b"subList" => {
    for attr in ce.attributes().flatten() {
        match attr.key.as_ref() {
            b"vertAlign" => {
                // 기존 처리 (vertical_align)
            }
            // === [Task #1028] 추가 ===
            b"textDirection" => {
                let val = attr_str(&attr);
                let direction_code: u32 = match val.as_str() {
                    "VERTICAL" | "VERTICALALL" => 1,
                    _ => 0,
                };
                // bit 0~2 (text_direction) 영역에 set
                text_box.list_attr =
                    (text_box.list_attr & !0b111) | direction_code;
            }
            // === 추가 끝 ===
            _ => {}
        }
    }
}
```

**비트 위치 근거**: `src/renderer/layout/shape_layout.rs:1649-1652` 가 글상자 list_attr bit 0~2 를 text_direction 으로 해석. HWP5 parser 와 동일 IR 사용.

**`VERTICALALL` 값 매핑**: 본 구현은 `"VERTICAL"` 과 `"VERTICALALL"` 모두 코드 1 매칭 (세로). `shape_layout.rs:1650` 주석의 "1=영문 눕힘, 2=영문 세움" 구분은 Stage 3 검증 시 한컴 PDF 와 시각 비교 후 필요 시 정밀화 (별도 task 권고).

## 3. Stage 분해

### Stage 2 — parser 변경 + 단위 검증 (예상 30분)

1. `parse_draw_text` 의 `subList` 루프에 `textDirection` 분기 추가 (위 코드)
2. `cargo build --release` 통과 확인
3. `cargo test --release --lib hwpx` 통과 확인 (영향 검토)
4. `cargo fmt` + `cargo clippy --release --lib -- -D warnings` 통과
5. 변경 commit

### Stage 3 — 검증 (예상 1시간)

1. **export-svg 비교**:
   - `rhwp export-svg samples/hwpx/tbox-v-flow-01.hwpx -o output/poc/task1028/hwpx/`
   - HWPX 출력 rotate transform 4건 (HWP5 동일) 확인
2. **광범위 sweep** (BEFORE devel ↔ AFTER):
   - `samples/hwpx/tbox-v-flow-01.hwpx` (타깃) 의도된 변동 입증
   - 일반 HWPX fixture (hy-001 등) 회귀 부재 (`text_direction` 미설정 케이스 영향 0 확인)
   - HWP5 변환본 (`samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp`) 무영향
3. **WASM 빌드** + rhwp-studio 시각 정합
4. **한글 2022 PDF 정합** (`pdf/hwpx/tbox-v-flow-01-2022.pdf`) — 작업지시자 시각 판정
5. **`feedback_push_full_test_required` 정합**: `cargo test --release --tests` 전체 + `cargo fmt --all -- --check` + `cargo clippy -- -D warnings` (CI 패턴) 모두 통과

### Stage 4 — 회귀 가드 + 보고서 (예상 30분)

1. **회귀 가드 `tests/issue_1028_hwpx_textbox_vertical.rs`** 추가:
   - HWPX 파싱 후 `text_box.list_attr & 0x07 == 1` 단언
   - 또는 export-svg 출력의 rotate transform 4건 단언 (svg_snapshot 패턴)
2. golden snapshot 추가 검토 (svg_snapshot 등록 여부 — 본 fixture 영구 가드 가치)
3. Stage 3 검증 결과 통합 + 최종 보고서 (`mydocs/report/task_m100_1028_report.md`)
4. orders/20260520 갱신
5. PR 생성 또는 직접 머지 (메인테이너 task 이므로 직접 머지)

## 4. 회귀 위험 평가

| 영역 | 위험도 | 근거 |
|------|--------|------|
| HWPX 글상자 (`textDirection` 미설정) | **낮음** | 기존 behavior 유지 (분기 미발동 시 코드 0, 가로 출력) |
| HWPX 글상자 (`textDirection="VERTICAL/VERTICALALL"`) | **의도된 변경** | 세로 렌더 활성화 (본 task 본질) |
| HWP5 변환본 | **영향 없음** | 본 PR 은 HWPX parser 만 변경 |
| Cell `textDirection` (section.rs:1108) | **영향 없음** | 별도 코드 경로 |

## 5. 메모리 룰 정합

- `feedback_image_renderer_paths_separate` — HWP5/HWPX parser 동일 IR (`list_attr` u32) 로 정합 → 동일 renderer 경로 (`shape_layout.rs:1649`) 사용
- `feedback_diagnosis_layer_attribution` — parser 단계 누락 식별 (renderer 변경 불필요)
- `feedback_hancom_compat_specific_over_general` — `textDirection` 매칭에 `VERTICAL`/`VERTICALALL` 만 추가, 기존 cell `VERTICAL` 매칭 영향 없음
- `feedback_visual_judgment_authority` — `pdf/hwpx/tbox-v-flow-01-2022.pdf` 시각 판정 게이트
- `feedback_push_full_test_required` — Stage 3 cargo test --tests + fmt --all + clippy 전체 (CI 패턴) 모두 검증
- `reference_authoritative_hancom` — 한글 2022 PDF baseline
- `project_output_folder_structure` — 산출물 `output/poc/task1028/`

## 6. 잔존 / 후속 권고 (본 task 범위 외)

- `section.rs:75 sec_def.text_direction`, `:1108 cell tcPr`, `:1139 cell legacy format` 의 `textDirection` 매칭에 `VERTICALALL` 추가 — 별도 task 권고 (현재 `VERTICAL` 만 매칭)
- `shape_layout.rs:1650` 주석의 "1=영문 눕힘, 2=영문 세움" 구분 정밀화 (PDF 시각 차이 발견 시) — 별도 task
