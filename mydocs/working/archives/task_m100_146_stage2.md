# 단계2 완료보고서: is_fullwidth_symbol 범위 추가 + 단위 테스트

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **단계**: 2 / 3 (is_fullwidth_symbol 범위 추가)
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v2.md`, `mydocs/plans/task_m100_146_v2_impl.md`

## 1. 수정 내역

### 1.1 소스 변경

`src/renderer/layout/text_measurement.rs` 의 `is_fullwidth_symbol` 함수 범위 리스트에 Geometric Shapes 블록을 추가 (한 줄):

```rust
|| ('\u{25A0}'..='\u{25FF}').contains(&c) // Geometric Shapes (□■▲◆○ 등, 섹션 머리 기호)
```

Enclosed Alphanumerics(U+2460-U+24FF) 와 Miscellaneous Symbols(U+2600-U+26FF) 사이에 삽입. 기존 주석 규칙과 일관.

### 1.2 테스트 추가

`src/renderer/layout/tests.rs` 끝에 2건 추가:

- `test_geometric_shapes_treated_as_fullwidth`: □ ■ ▲ ▼ ◆ ○ ● ◇ 8개 글자에 대해 advance == font_size (전각) 확인
- `test_square_bullet_with_space_preserves_layout`: "□ 가" + letter_spacing=-1.6 (자간 -8%) 시나리오에서 positions == [0, 18.4, 26.8, 45.2] 확인 (수정 전 기대값 붕괴)

### 1.3 호출 경로

`is_fullwidth_symbol` 는 같은 파일 내 3곳에서 호출된다:
- line 184: `EmbeddedTextMeasurer::estimate_text_width` 의 `char_width` 클로저
- line 278: `EmbeddedTextMeasurer::compute_char_positions`
- line 793: `WasmTextMeasurer` 측정 로직

**한 줄 추가로 네이티브/WASM 두 경로에 모두 반영됨.**

## 2. 검증 결과

### 2.1 단위 테스트

신규 2건 모두 통과:
```
running 1 test
test test_geometric_shapes_treated_as_fullwidth ... ok

running 1 test
test test_square_bullet_with_space_preserves_layout ... ok
```

### 2.2 SVG 재현 검증 (samples/text-align.hwp)

| 글자 | 변경 전 (svg) | 변경 후 (svg) | PDF 환산 | 오차 (변경 후) |
|------|-------------|-------------|---------|---------------|
| □ 시작 | 75.58 | 75.58 | 75.48 | 0.10 px |
| 국 시작 | 94.40 | **105.40** | 105.39 | **0.01 px** |
| 어 시작 | 114.88 | 125.88 | 125.85 | 0.03 px |
| 변 시작 | 145.17 | 156.17 | 156.33 | 0.16 px |

제목 전체가 한컴 PDF 와 **1 px 이내**로 수렴. Chrome headless 150dpi PNG 시각 확인 시 "□ 국어..." 공백 복원.

### 2.3 전체 테스트 영향

`cargo test --lib`:
- **변경 전**: 927 passed; 14 failed; 1 ignored (14 실패는 `serializer::cfb_writer` / `wasm_api` roundtrip, 본 타스크와 무관한 기존 실패)
- **변경 후**: 929 passed; 14 failed; 1 ignored (신규 테스트 2건 추가 통과)

**본 변경으로 추가된 실패 0건.** 기존 14건 실패는 스태시 후 재측정으로 사전 확인.

### 2.4 svg_snapshot 영향

`cargo test --test svg_snapshot`:
- 실패: `form_002_page_0` (1건)
- 실패 내용: form-002.hwp page 0 의 "□ 개념" / "□ 개발내용" 두 곳에서 "개" 이후 글자가 +9px 우측 이동

diff 샘플:
```
< 개 x=95.59    →   > 개 x=104.59   (+9.00 px)
< 념 x=113.69  →   > 념 x=122.69   (+9.00 px)
```

**+9px 이동은 text-align.hwp 와 동일 패턴** (□ 전각 인식으로 후속 글자 정상 advance). PDF 기준으로 가까워지는 방향의 변경이므로 올바른 수정. Stage 3 에서 golden 업데이트 + 사유 기록 예정.

### 2.5 clippy

`cargo clippy --lib -- -D warnings`: 경고 없이 통과.

## 3. 커밋 계획

본 단계 산출물 2개 커밋으로 분리:

**커밋 1 (소스 + 테스트)**:
- `src/renderer/layout/text_measurement.rs` (Geometric Shapes 범위 추가)
- `src/renderer/layout/tests.rs` (단위 테스트 2건)
- 메시지: `Task #146: Geometric Shapes(U+25A0-U+25FF) 를 전각 심볼로 처리`

**커밋 2 (문서)**:
- `mydocs/plans/task_m100_146_v2.md`
- `mydocs/plans/task_m100_146_v2_impl.md`
- `mydocs/working/task_m100_146_stage2.md`
- 메시지: `docs: Task #146 v2 계획서 + 단계2 보고서`

## 4. 다음 단계 (Stage 3)

- `svg_snapshot` golden 업데이트: `UPDATE_GOLDEN=1 cargo test --test svg_snapshot form_002_page_0` 실행 후 diff 내용 확인
- `samples/` 내 Geometric Shapes 다수 사용 문서 3~5개에 대해 150dpi 전후 비교 (회귀 여부 확인)
- `mydocs/report/task_m100_146_report.md` 작성
- `mydocs/orders/20260423.md` v2 체크리스트로 갱신
