# Task m100-2995: HWP3 문단 테두리(has_border) 플래그 소실 수정

## 배경

`#2976`(`convert_para_shape()`의 `border_connection` 배선 누락, PR #2983)과 동일한 방법론을
`Hwp3ParaShape`(`src/parser/hwp3/records.rs`)에 다시 적용했다. 즉 HWP3 바이너리 레코드를 읽어들인
struct의 각 접근자가 공통 IR(`Document` 모델)까지 실제로 배선되는지, 아니면 struct에는 읽혔지만
변환 지점에서 조용히 누락되는지를 필드 단위로 대조했다.

`grep -rn "\.has_border()\|\.border_connection()" src/`로 확인한 결과 `border_connection()`은
#2983에서 다루는 중이었고, `has_border()`(`src/parser/hwp3/records.rs` 392~394행)는 저장소 전체
어디서도 호출되지 않고 있었다.

## 발견한 결함

`Hwp3ParaShape.border: u8`는 문단 테두리 on/off 플래그이고 `has_border()`가 이를 bool로 해석하는
접근자다. 이 값을 소비할 수 있는 유일한 지점은 `src/parser/hwp3/mod.rs`의 `parse_paragraph_list()`
안에서 `shade_ratio`(문단 음영) 값만 검사해 `BorderFill`을 만들고 `ps.border_fill_id`에 배선하는
인라인 코드였는데, 이 블록이 `shade_ratio > 0`만 조건으로 삼고 `has_border()`는 전혀 참조하지
않았다. 결과적으로 음영 없이 테두리만 켜진 문단(`border == 1`, `shade_ratio == 0`)은
`border_fill_id`가 끝까지 0으로 남아 테두리 정보가 항상 소실됐다.

이슈 [#2995](https://github.com/edwardkim/rhwp/issues/2995)로 등록했다.

## 영향

HWP3(.hwp) 문서에서 문단에 음영 없이 테두리만 지정한 경우(강조 박스, 예시/정답 구획 등 옛 한글
문서에서 흔한 패턴), rhwp 파싱 결과 및 이를 HWPX로 재저장한 결과에서 해당 테두리가 항상 사라진다.

## 수정 내용

`src/parser/hwp3/mod.rs`에 순수 함수 `hwp3_para_shape_border_fill()`을 추가해 기존 shade 전용
인라인 로직을 이 함수로 옮기고, `shade_ratio > 0 || has_border()` 조건으로 확장했다.
`has_border()`가 true면 `BorderFill.borders`의 4방향(좌/우/상/하)을 `BorderLineType::Solid`로
설정한다. HWP3 `Hwp3ParaShape`에는 선 굵기·색상 필드가 없어(`border: u8` 단일 on/off 플래그) 두께·
색은 `BorderLine`의 기본값을 그대로 사용했다. `parse_paragraph_list()`의 호출부는 아래처럼
단순화됐다.

```rust
if let Some(bf) = hwp3_para_shape_border_fill(hwp3_ps) {
    doc_border_fills.push(bf);
    ps.border_fill_id = doc_border_fills.len() as u16; // 1-based (렌더러 규칙)
}
```

기존에 이미 존재하던 `has_border()` 접근자를 호출해 이미 확립된 `border_fill_id` 배선 경로에
맞춰 넣는 최소 수정이며, `convert_char_shape()`(#2521)·`convert_para_shape()`(#2976)와 같은
"접근자는 있지만 호출되지 않음" 패턴이다.

## 테스트 (red → green)

`src/parser/hwp3/mod.rs`의 `tests` 모듈에
`test_hwp3_para_shape_border_fill_wires_has_border_flag`를 추가했다.

- **Red (수정 전)**: `Hwp3ParaShape { border: 1, shade_ratio: 0, .. }`을
  `hwp3_para_shape_border_fill()`에 넣으면 (수정 전 로직 기준으로는) `None`이 반환되어
  `.expect("border_fill 이 생성되어야 함")`가 패닉했다.
- **Green (수정 후)**: 동일 입력에 대해 `Some(BorderFill)`이 반환되고, 4방향 테두리선의
  `line_type`이 모두 `BorderLineType::Solid`로 세팅되어 통과한다.

```
cargo test --lib hwp3_para_shape_border_fill
running 1 test
test parser::hwp3::tests::test_hwp3_para_shape_border_fill_wires_has_border_flag ... ok
```

## 검증

- `cargo check --lib` 통과.
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs` 적용.
