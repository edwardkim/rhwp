# Task M100-1071 Stage 2 완료 보고서

## 1. 목표

Stage 1에서 특정한 HWPX `hp:secPr` control slot 불일치를 정정해,
`shape-001.hwpx` 의 TAC 도형 가로 배치가 HWP 정답지와 같아지도록 한다.

## 2. 원인 재확인

Stage 1에서 확인한 문제는 다음과 같다.

```text
char_offsets stream:
  [secPr][colPr][shape1][" "][" "][shape2]

기존 HWPX para.controls:
  [colPr][shape1][shape2]
```

`char_offsets` 에는 `secPr` 의 8 UTF-16 code unit marker가 반영되어 있었지만,
`para.controls` 에는 대응하는 `Control::SectionDef` 가 없었다.
그 결과 TAC 도형 control slot 이 한 칸씩 당겨지고, 두 도형 사이의 공백 advance가
정상적으로 반영되지 않았다.

## 3. 구현

수정 파일:

```text
src/parser/hwpx/section.rs
```

변경 내용:

```text
hp:secPr 파싱 시
1. 기존처럼 Section.section_def 로 반환할 SectionDef 를 보존한다.
2. 동시에 같은 SectionDef 를 Control::SectionDef 로 para.controls 에 추가한다.
3. 기존 control marker(\u0002) 는 그대로 유지한다.
4. colPr 은 기존처럼 Control::ColumnDef 로 이어서 추가한다.
```

정정 후 HWPX 첫 문단은 HWP 정답지와 같은 control stream을 가진다.

```text
[SectionDef, ColumnDef, Shape, Shape]
```

## 4. 어댑터 영향

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
tests/hwpx_to_hwp_adapter.rs
```

기존 `hwpx_to_hwp` adapter는 HWPX 파서가 `SectionDef` control을 만들지 않는다는
전제에서 fallback 삽입을 수행했다. 이번 Stage 2 이후 파서는 직접 materialize 하므로,
어댑터 설명과 테스트를 다음 계약으로 조정했다.

```text
1. 현재 파서 산출물에는 첫 문단 Control::SectionDef 가 이미 있어야 한다.
2. 어댑터는 예전 파서 산출물/외부 IR 호환을 위해 fallback 삽입 경로를 유지한다.
3. 이미 SectionDef 가 있으면 no-op 이어야 한다.
```

## 5. 회귀 가드

수정 파일:

```text
tests/issue_1067_shape_rotation.rs
```

추가한 검증:

```text
1. shape-001 HWP/HWPX 의 control stream 순서가 동일해야 한다.
2. shape-001 HWP/HWPX 의 char_offsets 가 동일해야 한다.
3. shape-001 HWP/HWPX 의 control_text_positions() 결과가 동일해야 한다.
4. SVG export 에서 두 TAC 도형의 rotate center x 값이 HWP/HWPX 간 0.01 미만 차이여야 한다.
```

## 6. 검증 결과

```text
cargo fmt --check
cargo check
cargo test --test issue_1067_shape_rotation
cargo test --test hwpx_to_hwp_adapter stage4_section_def
cargo run --quiet --example dump_shape_para -- samples/shape-001.hwp samples/hwpx/shape-001.hwpx
```

결과:

```text
cargo fmt --check: success
cargo check: success
issue_1067_shape_rotation: 7 passed, 0 failed
hwpx_to_hwp_adapter stage4_section_def: 2 passed, 0 failed
dump_shape_para: HWP/HWPX 모두 controls=4, char_offsets=[24,25]
```

## 7. 결론

Stage 2는 HWPX TAC 도형 가로 위치 문제의 핵심 원인인 `secPr` control slot 불일치를 정정했다.
이제 HWPX `shape-001.hwpx` 는 HWP 정답지와 같은 문단 control stream과 SVG 도형 중심 x 값을 가진다.

남은 범위는 issue 본문의 두 번째 축인 "도형 컨트롤이 한글자처럼 캐럿 이동" 문제다.
Stage 3에서는 rhwp-studio cursor/navigation 경로에서 TAC 도형의 character width와 caret step을
검증한다.
