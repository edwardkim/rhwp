# Task m100-1: HWP3 convert_para_shape() border_connection 배선 누락 수정

## 배경

`#2968`(shade_color 누락), `#2521`(위/아래첨자 attr 매핑 누락) 수정과 동일한 방법론을
`convert_para_shape()`(`src/parser/hwp3/mod.rs`)와 `convert_style()`에 적용했다. 즉,
HWP3 바이너리 레코드를 읽어들인 struct의 각 필드가 공통 IR(`Document` 모델)까지
실제로 배선되는지, 아니면 struct에는 읽혔지만 IR 변환 지점에서 조용히 누락되는지를
필드 단위로 대조했다.

## 결과 요약

- `convert_para_shape()`: 결함 발견 → 수정 완료 (이슈 [#2976](https://github.com/edwardkim/rhwp/issues/2976))
- `convert_style()`: 대조 결과 clean. 이름, 다음 스타일 참조, lang font, para-shape-ref,
  char-shape-ref 모두 IR로 정상 배선되어 있음을 확인.

## 발견한 결함

`src/parser/hwp3/records.rs`의 `Hwp3ParaShape`는 `border_connection: u8` 필드와
이를 bool로 해석하는 접근자 `border_connection() -> bool`(약 396~397행)을 갖고 있다.

```rust
pub fn border_connection(&self) -> bool {
    self.border_connection == 1
}
```

그런데 `convert_para_shape()`(`src/parser/hwp3/mod.rs` 288행~)는 `margin_left`,
`margin_right`, `indent`, `line_spacing`, `margin_bottom`, `margin_top`, `align`,
`tabs`는 모두 IR로 옮기지만 `border_connection`은 어디에도 사용하지 않았다.

IR 쪽에서 "문단 테두리를 인접 문단과 연결해서 그릴지" 플래그는 이미 확립된 규약으로
`ParaShape.attr1`의 bit 28에 인코딩된다.

- `src/serializer/hwpx/header.rs:1101` — HWPX 내보내기 시 이 비트를 읽어 `connect` 속성으로 씀
- `src/document_core/commands/formatting.rs:800` — 편집 커맨드에서 같은 비트를 읽음
- `src/model/style.rs:909, 1004` — `ParaShapeMods.border_connect`가 같은 비트를 세팅

즉 IR과 그 소비자들은 이미 완성되어 있었는데 HWP3 파서만 이 비트를 채우지 않아,
HWP3(.hwp) 문서에서 문단 테두리를 인접 문단과 연결하도록 설정한 경우 그 정보가
파싱 단계에서 항상 소실되고 있었다. `convert_char_shape()`가 과거 위/아래첨자
접근자는 있었지만 attr 매핑이 빠져 있던 결함(#2521)과 정확히 같은 패턴이다.

## 영향

HWP3(.hwp) 문서에서 문단 테두리를 "연결"로 설정한 경우, rhwp로 파싱한 결과(및 이를
HWPX로 재저장한 결과)에서는 항상 연결되지 않은 것으로 처리된다. 문단 테두리를 사용하는
옛 HWP3 문서(표 유사 레이아웃, 강조 박스 등)에서 렌더링·재저장 시 테두리가 문단
경계마다 끊어져 보이는 시각적 회귀가 발생한다.

## 수정 내용

`convert_para_shape()`에 아래 3줄을 추가했다.

```rust
if hwp3_ps.border_connection() {
    ps.attr1 |= 1 << 28;
}
```

기존에 이미 존재하던 접근자를 그대로 호출해 IR의 확립된 비트 규약에 맞춰 배선하는
최소 수정이며, 다른 로직·스케일 변환에는 영향을 주지 않는다.

## 테스트 (red → green)

`src/parser/hwp3/mod.rs`의 `tests` 모듈에
`test_convert_para_shape_wires_border_connection_into_attr1_bit28`을 추가했다.

- **Red (수정 전)**: `Hwp3ParaShape { border_connection: 1, .. }`을
  `convert_para_shape()`에 넣으면 반환된 `ParaShape.attr1`의 bit 28이 0으로 남아
  `assert_eq!((ps.attr1 >> 28) & 1, 1, ...)`가 실패했다.
- **Green (수정 후)**: 동일 입력에 대해 bit 28이 1로 세팅되어 통과.

```
running 1 test
test parser::hwp3::tests::test_convert_para_shape_wires_border_connection_into_attr1_bit28 ... ok
```

## 검증

- `cargo check --lib` — 통과 (경고 없음)
- `cargo test --lib test_convert_para_shape_wires_border_connection_into_attr1_bit28` — 통과
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs` — 적용 완료

## 변경 파일

- `src/parser/hwp3/mod.rs` (로직 3줄 + 회귀 테스트)

## 이슈/PR

- 이슈: https://github.com/edwardkim/rhwp/issues/2976
- PR: (커밋 후 생성)
