# M100 #949 최종 결과 보고서

## 1. 작업 개요

- 이슈: #949
- 브랜치: `local/task949`
- 기준 커밋: `39d90d9d Merge local/devel: task #955 rustfmt policy`
- 목표: HWPX를 rhwp IR로 로드한 뒤 HWP5로 저장할 때 한컴 에디터에서 발생하던 `파일손상`/중단 문제를 정답 HWP contract 기준으로 해소한다.

대상 샘플:

```text
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hwpx-h-02.hwpx
samples/hwpx/hwpx-h-03.hwpx
samples/hwpx/hy-001.hwpx
```

`hwpx-h-01`, `hwpx-h-02`는 guard이고, `hwpx-h-03`, `hy-001`은 파일손상/중단 재현 대상이다.

## 2. 최종 판정

Stage 36 산출물 기준으로 네 샘플 모두 한컴 에디터와 rhwp-studio 판정을 통과했다.

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-03.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | target |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hy-001.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | #974 guard |

따라서 #949의 핵심 목표였던 HWPX -> HWP 저장 결과의 한컴 에디터 파일손상 문제는 해결로 판정한다.

## 3. 원인

최종 원인은 `SHAPE_COMPONENT`의 `renderingInfo` matrix 정밀도 contract였다.

정답 HWP와 generated HWP의 record 구조, count, tail은 이미 맞았고, 남은 차이는 rendering matrix block의 double 값 하위 바이트였다.

확인된 규칙:

```text
HWPX XML value: 0.723629
rhwp 기존 저장: f64(0.723629)
한컴 정답 저장: f64(f32(0.723629))
```

한컴의 HWPX -> HWP 저장 경로는 `renderingInfo`의 소수 matrix 값을 XML decimal에서 바로 `f64`로 저장하지 않는다. 먼저 `f32` 정밀도로 양자화한 뒤, HWP5 `SHAPE_COMPONENT` raw rendering payload의 8바이트 double slot에 기록한다.

이 차이가 `hwpx-h-03`과 `hy-001`의 글상자/그림 묶음 구간에서 한컴 로더 contract 위반으로 이어졌다.

## 4. 구현 내용

### 4.1 lineSegArray 명시 vpos 보존

수정 파일:

```text
src/document_core/commands/document.rs
```

`reflow_zero_height_paragraphs()`가 비-TAC `TopAndBottom` 개체 존재만으로 section 전체 `lineSeg.vertical_pos`를 재계산하던 경로를 정정했다.

변경 후에는 본문 문단의 `line_segs`를 실제로 합성하거나 보정한 경우에만 section vpos를 다시 계산한다. HWPX source에 이미 명시된 `lineSegArray`의 `vertpos`는 저장 시 덮어쓰지 않는다.

### 4.2 HWPX shape/object contract 보강

수정 파일:

```text
src/model/shape.rs
src/parser/hwpx/section.rs
src/parser/hwpx/utils.rs
src/parser/control/shape.rs
src/document_core/converters/common_obj_attr_writer.rs
src/document_core/converters/hwpx_to_hwp.rs
```

반영한 주요 항목:

```text
- id / instid 분리 파싱
- flowWithText, allowOverlap 파싱 및 HWP5 common attr 반영
- signed offset 파싱
- shape storage 기본값 보강
  - Picture: 0x24000000
  - Group: 0x00090000
  - TextBoxDrawing: 0x01000000
- rotateimage bit 반영
- lineShape color/style/headfill/tailfill 파싱
- shadow 속성 파싱
- drawText 내부 picture CTRL_DATA materialization 유지
- table axis contract materialization 유지
```

### 4.3 renderingInfo raw payload 생성 및 f32 양자화

수정 파일:

```text
src/parser/hwpx/section.rs
```

`parse_rendering_info()`에서 HWP5 writer가 그대로 쓸 수 있는 raw rendering payload를 생성한다.

적용 규칙:

```text
- 정수 matrix 값: 원값 유지
- 소수 matrix 값: f32로 양자화 후 f64로 승격
```

추가 단위 테스트:

```text
parser::hwpx::section::tests::test_rendering_info_materializes_hwp5_raw_rendering_count
parser::hwpx::section::tests::test_rendering_info_quantizes_fractional_matrix_values_like_hwp5
```

## 5. 정답 HWP byte 비교

`hwpx-h-03` 문제 구간은 정답 HWP와 byte-equal로 닫혔다.

```text
#824 CTRL_HEADER: equal
#825 SHAPE_COMPONENT: equal
#826 LIST_HEADER: equal
#827 PARA_HEADER: equal
#831 CTRL_HEADER: equal
#832 SHAPE_COMPONENT: equal
#834 SHAPE_PICTURE: equal
#835 CTRL_HEADER: equal
#836 SHAPE_COMPONENT: equal
#837 SHAPE_PICTURE: equal
#838 SHAPE_RECTANGLE: equal
```

`hy-001` 문제 구간도 정답 HWP와 byte-equal로 닫혔다.

```text
#222 CTRL_HEADER: equal
#223 SHAPE_COMPONENT: equal
#224 LIST_HEADER: equal
#225 PARA_HEADER: equal
#229 CTRL_HEADER: equal
#230 SHAPE_COMPONENT: equal
#231 SHAPE_PICTURE: equal
#232 CTRL_HEADER: equal
#233 SHAPE_COMPONENT: equal
#234 SHAPE_PICTURE: equal
#235 SHAPE_RECTANGLE: equal
```

## 6. 회귀 테스트

추가/갱신한 회귀 가드:

```text
tests/hwpx_to_hwp_adapter.rs
- task949_stage33_hwpx_h03_explicit_lineseg_vpos_preserved_on_load
- task949_stage33_hwpx_h03_explicit_lineseg_vpos_survives_adapter_export_reload

tests/issue_554.rs
- hwp3_sample_hwp5_16p
- hwp3_sample_hwpx_16p

tests/golden_svg/issue-157/page-1.svg
- HWPX 명시 lineSeg vpos 보존 기준으로 snapshot 갱신
```

`issue_554`의 두 known-limit 테스트는 기존에 "정답 16, 현재 15" 상태를 가드하고 있었다. #949의 lineSeg vpos 보존 후 실제 출력이 한컴 정답 16p로 회복되어 기대값을 정답 기준으로 갱신했다.

`issue-157` SVG snapshot은 비-TAC `wrap=위아래` 표 주변 y 위치가 명시 lineSeg 보존 기준으로 달라져 골든을 갱신했다.

## 7. 검증

실행한 검증:

```bash
cargo fmt --check
git diff --check
cargo build
cargo test parser::hwpx::section::tests::test_rendering_info_quantizes_fractional_matrix_values_like_hwp5 -- --nocapture
cargo test --test hwpx_to_hwp_adapter task949_stage33_hwpx_h03_explicit_lineseg_vpos -- --nocapture
cargo test --test hwpx_to_hwp_adapter -- --nocapture
cargo test --test issue_554 -- --nocapture
UPDATE_GOLDEN=1 cargo test --test svg_snapshot
cargo test --quiet
```

결과:

```text
cargo fmt --check 통과
git diff --check 통과
cargo build 통과
renderingInfo f32 양자화 단위 테스트 통과
hwpx_to_hwp_adapter 전체 통과: 49 passed, 11 ignored
issue_554 통과: 12 passed
svg_snapshot 통과: 8 passed
cargo test --quiet 통과
```

참고:

```text
cargo test --quiet 실행 중 기존 warning 6건이 출력되었지만 실패는 없다.
```

## 8. 문서화

추가/갱신 문서:

```text
mydocs/troubleshootings/hwpx2hwp_shape_rendering_matrix_precision.md
mydocs/troubleshootings/hwpx2hwp-rule.md
mydocs/plans/task_m100_949_stage22.md ~ task_m100_949_stage36.md
mydocs/working/task_m100_949_stage22.md ~ task_m100_949_stage36.md
```

새 troubleshooting 문서에는 다음 내용을 기록했다.

```text
- 한컴 파일손상 증상
- 정답 HWP와 generated HWP의 record bundle 비교 방식
- SHAPE_COMPONENT rendering matrix f32 양자화 규칙
- 재발 시 진단 절차
```

## 9. 잔여 관찰

한컴 로딩/저장 contract는 통과했지만 rhwp-studio의 조판 fidelity에는 다음 차이가 남아 있다.

```text
- hwpx-h-03: rhwp-studio에서 2페이지 문단과 표 사이의 간격이 한컴 에디터보다 넓다.
- hy-001: rhwp-studio에서 2페이지 문단 다음 엔터 두 번 후 배치되는 표가 한컴보다 더 아래에 배치된다.
```

이 항목은 HWPX -> HWP 저장 contract 실패가 아니라 rhwp-studio 페이지/문단/표 간격 조판 fidelity 문제로 분리한다.

## 10. 결론

#949는 완료 가능 상태다.

핵심 결론:

```text
1. table axis contract와 drawText/picture contract만으로는 파일손상을 닫을 수 없었다.
2. 마지막 한컴 파일손상 원인은 SHAPE_COMPONENT rendering matrix의 정밀도 contract였다.
3. HWPX 소수 matrix 값은 f32로 양자화한 뒤 HWP5 double slot에 기록해야 한다.
4. 해당 규칙 적용 후 hwpx-h-03, hy-001 모두 정답 HWP 문제 구간과 byte-equal이 되었고 한컴 판정도 성공으로 전환되었다.
```

## 11. 승인 요청

위 결과로 #949 완료 처리를 승인 요청한다.
