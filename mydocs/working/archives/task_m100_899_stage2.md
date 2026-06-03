# Task m100 #899 Stage 2 - GREEN 구현

## 1. 목적

Stage 1 RED 테스트를 GREEN으로 전환한다.

대상 버그:

- HWPX `winBrush`에 `faceColor`는 있으나 `hatchStyle`이 없는 셀 배경
- HWP 저장용 IR에서 `SolidFill.pattern_type`이 `0`으로 남음
- 한컴 HWP 저장 시 셀 배경색에 배경 무늬가 `무늬없음`으로 적용되지 않음

## 2. 스펙 확인

OWPML Core XML schema의 `FillBrushType` / `winBrush` 정의:

- `faceColor`: 면 색
- `hatchColor`: 무늬 색
- `hatchStyle`: 무늬 종류

`hatchStyle` 열거:

```text
HORIZONTAL
VERTICAL
BACK_SLASH
SLASH
CROSS
CROSS_DIAGONAL
```

`hatchStyle`이 없으면 HWP 저장 쪽에서는 `pattern_type=-1`로 정규화해야 한다.

## 3. 구현

수정 파일:

```text
src/parser/hwpx/utils.rs
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
tests/hwpx_to_hwp_adapter.rs
```

핵심 변경:

- `parse_hatch_style()` 추가
- OWPML 6개 `hatchStyle` 값을 HWP `pattern_type` 1~6에 매핑
- HWPX `winBrush` 파싱 시 `SolidFill.pattern_type` 기본값을 `-1`로 설정
- `hatchStyle`이 명시된 경우에만 매핑값으로 덮어씀

매핑:

```text
HORIZONTAL      -> 1
VERTICAL        -> 2
BACK_SLASH      -> 3
SLASH           -> 4
CROSS           -> 5
CROSS_DIAGONAL  -> 6
```

## 4. 적용 범위

`header.xml`의 `borderFill/fillBrush/winBrush`를 수정했다.

또한 같은 의미를 쓰는 `section.xml`의 shape `fillBrush/winBrush` 경로도 같은 기본값과 매핑을 사용하도록 맞췄다.

## 5. 테스트

### RED 테스트 재실행

```text
cargo test --test hwpx_to_hwp_adapter task899_business_overview_cell_backgrounds_use_no_pattern -- --nocapture
```

결과:

```text
ok
```

### hatchStyle 매핑 유닛 테스트

```text
cargo test parser::hwpx::utils::tests::test_parse_hatch_style -- --nocapture
```

결과:

```text
ok
```

### HWPX -> HWP 어댑터 전체 테스트

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
31 passed; 0 failed
```

## 6. 주의 사항

`cargo test parser::hwpx::utils::tests::test_parse_hatch_style -- --nocapture` 실행 중 기존 경고가 출력되었다.

이번 변경과 무관한 기존 경고:

- `duplicated attribute`
- `unused_parens`
- `non_snake_case`
- `unused_must_use`

## 7. 판정

Stage 2 GREEN 조건을 만족한다.

`business_overview.hwpx`의 셀 배경 BorderFill 5, 6, 7은 `faceColor`가 있고 `hatchStyle`이 없으므로, IR에서 `pattern_type=-1`로 유지된다.

다음 Stage 3에서는 작업지시자 판정용 HWP 산출물을 `output/` 아래에 생성하고, 한컴/rhwp-studio 시각 검증으로 실제 셀 배경색과 무늬없음 적용 상태를 확인한다.
