# Task M100-1028 최종 결과 보고서

- 이슈: [#1028](https://github.com/edwardkim/rhwp/issues/1028) — HWPX 글상자 세로 쓰기 미구현 (HWP5는 지원, HWPX 미적용)
- 처리일: 2026-05-20
- 브랜치: `local/task1028` (base `local/devel = 20b544f8`)
- 마일스톤: v1.0.0
- assignee: @edwardkim

## 1. 해소 요약

HWPX `parse_draw_text` 의 `<hp:subList>` 처리에서 `textDirection` 속성 파싱 누락 → `text_box.list_attr` bit 0~2 (text_direction) 항상 0 → renderer 의 세로쓰기 분기 미발동. **parser 12 줄 추가**로 HWPX 글상자 세로쓰기 활성화. HWP5/HWP3 영향 없음.

## 2. 처리 내역

| Stage | 커밋 | 내용 |
|-------|------|------|
| **사전** | `20b544f8` | fixture 3 개 추가 (samples/hwpx + hancom-hwp + pdf/hwpx) |
| **Stage 2** | `7a601033` | parser 12 줄 + 단위 검증 |
| **Stage 4** | (본 커밋) | 회귀 가드 + 최종 보고서 + orders |

## 3. 변경 본질 (단일 파일)

`src/parser/hwpx/section.rs::parse_draw_text` 의 `<hp:subList>` attribute 루프에 `textDirection` 분기 추가:

```rust
b"textDirection" => {
    let direction_code: u32 = match attr_str(&attr).as_str() {
        "VERTICAL" | "VERTICALALL" => 1,
        _ => 0,
    };
    // bit 0~2 (text_direction) 에 set
    text_box.list_attr =
        (text_box.list_attr & !0b111) | direction_code;
}
```

**비트 위치 근거**: `src/renderer/layout/shape_layout.rs:1649-1652` 가 글상자 list_attr bit 0~2 를 text_direction 으로 해석 — HWP5 와 동일 IR (`list_attr` u32) 공유.

**Renderer 영역 변경 불필요**: 이미 `layout_vertical_textbox_text_with_paras` 분기 보유 (HWP5 가 사용 중).

## 4. 자기 검증 (`feedback_push_full_test_required` + CI 패턴)

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | **1308 passed** |
| `cargo test --release --lib hwpx` | 123/0 |
| `cargo test --release --tests` 전체 통합 | FAILED 0 |
| `cargo test --release --test issue_1028_hwpx_textbox_vertical` | **2/2** (vertical/horizontal 양쪽) |
| `cargo clippy --release --lib -- -D warnings` | 통과 |
| `cargo fmt --all -- --check` (CI 패턴) | exit 0 |
| WASM 빌드 (Docker) | 4.84 MB, rhwp-studio/public 동기화 |

## 5. sweep 검증 (BEFORE devel `20b544f8` ↔ AFTER) — case-specific 입증

| Fixture | 결과 | 판정 |
|---------|------|------|
| **tbox-v-flow-01.hwpx (타깃)** | rotate 0 → **4** (HWP5 정답지 동일) | ✓ |
| hy-001 HWPX / HWP5 | **diff=0** | ✅ 무회귀 |
| sample16-hwp5 / sample16-hwp3 | **diff=0** | ✅ 무회귀 |
| exam_kor / aift / table-vpos-01 | **diff=0** | ✅ 무회귀 |

PR 변경이 **`textDirection` 속성 보유 HWPX 글상자에만 영향** — 일반 fixture(`textDirection` 미설정 또는 `HORIZONTAL`)는 무영향. case-specific 동작 입증.

## 6. 작업지시자 시각 판정 — 통과

- `samples/hwpx/tbox-v-flow-01.hwpx` (윤동주 "서시" 세로쓰기 글상자) — 한글 2022 PDF (`pdf/hwpx/tbox-v-flow-01-2022.pdf`) 정합
- 작업지시자 명시 "HWP 는 정상 렌더링, HWPX 만 미렌더" → 본 PR 로 HWPX 정합 달성
- rhwp-studio (WASM 4.84MB) 동기화 + dev 서버 7700

## 7. 회귀 가드 — `tests/issue_1028_hwpx_textbox_vertical.rs` (2 tests)

영구 회귀 가드 등록:

1. **`issue_1028_hwpx_textbox_vertical_direction_parsed`**:
   - `tbox-v-flow-01.hwpx` 파싱 후 글상자 `list_attr & 0x07 == 1` 단언
   - `VERTICALALL` → code 1 매핑 정합 검증

2. **`issue_1028_hwpx_horizontal_textbox_unchanged`**:
   - `hy-001.hwpx` 의 일반(가로) 글상자가 `list_attr & 0x07 == 0` 유지
   - case-specific 가드 (textDirection 미설정 케이스 영향 0)

## 8. 잔존 / 후속 권고 (본 task 범위 외)

- `section.rs:75 sec_def.text_direction`, `:1108 cell tcPr`, `:1139 cell legacy format` 의 `textDirection` 매칭에 `VERTICALALL` 추가 — 현재 `VERTICAL` 만 매칭. 별도 task 권고 (영향 좁음)
- `shape_layout.rs:1650` 주석의 "1=영문 눕힘 vs 2=영문 세움" 구분 정밀화 — 본 fixture 시각 판정 통과로 우선순위 낮음. 영문 텍스트 + 세로쓰기 fixture 발견 시 별도 task

## 9. 메모리 룰 정합

- `feedback_image_renderer_paths_separate` — HWP5/HWPX parser 동일 IR (`list_attr` u32) 사용 → 동일 renderer 경로 (`shape_layout.rs:1649`). parser 단계 누락만 정정
- `feedback_diagnosis_layer_attribution` — Stage 1 진단으로 parser 단계 누락 정확 식별 (renderer 변경 불필요 확인)
- `feedback_hancom_compat_specific_over_general` — `textDirection` 매칭에 `VERTICAL/VERTICALALL` 만 추가, 기존 cell `VERTICAL` 매칭 영향 없음. case-specific
- `feedback_visual_judgment_authority` — 한글 2022 PDF 시각 판정 + 작업지시자 통과
- `feedback_push_full_test_required` — Stage 2/3/4 모두 `--tests` 전체 + fmt --all + clippy 전체 (CI 패턴) 검증
- `reference_authoritative_hancom` — `pdf/hwpx/tbox-v-flow-01-2022.pdf` baseline 정합
- `feedback_assign_issue_before_work` — 이슈 등록 직후 assignee 지정 (Stage 0)
- `project_output_folder_structure` — 산출물 `output/poc/task1028/` 배치
- `feedback_fix_scope_check_two_paths` — Stage 1 진단으로 HWP5 (정상) vs HWPX (누락) 두 경로 차이 식별 + HWPX 만 정정
