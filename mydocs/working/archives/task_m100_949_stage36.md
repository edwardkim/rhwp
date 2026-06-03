# Task M100-949 Stage 36 작업 기록

## 1. 목적

Stage 35 이후에도 `hwpx-h-03`과 `hy-001`의 일부 `SHAPE_COMPONENT` payload hash가 정답
HWP와 달랐다. Stage 36은 남은 차이를 byte offset 단위로 추적해 `renderingInfo` matrix
정밀도 계약을 구현한다.

## 2. 핵심 발견

남은 차이는 record 구조, tail, count 문제가 아니었다. 모두 `SHAPE_COMPONENT`의 rendering
matrix block 안에 있는 scale matrix double 값의 하위 바이트 차이였다.

예시:

```text
HWPX value: 0.723629
rhwp 기존 저장: f64(0.723629)
한컴 정답 저장: f64(f32(0.723629))
```

즉 한컴 HWPX -> HWP 저장 경로는 `renderingInfo`의 fractional matrix 값을 그대로 `f64`로
저장하지 않고, `f32` 정밀도로 한 번 양자화한 뒤 HWP5 `SHAPE_COMPONENT`의 8바이트 double
slot에 기록한다.

## 3. 구현 내용

`src/parser/hwpx/section.rs`의 `parse_rendering_info()`에서 XML matrix 숫자를 읽을 때 다음
규칙을 적용했다.

```text
- 정수값: 기존 값 유지
- 소수값: f32로 양자화한 뒤 f64로 승격
```

이 규칙은 HWP5 raw rendering payload 생성에만 적용된다.

추가 테스트:

```text
parser::hwpx::section::tests::test_rendering_info_quantizes_fractional_matrix_values_like_hwp5
```

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hy-001.hwp
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

## 6. rhwp 재로드 확인

```text
hwpx-h-01.hwp: 9 pages
hwpx-h-02.hwp: 10 pages
hwpx-h-03.hwp: 9 pages
hy-001.hwp: 2 pages
```

## 7. 실행한 검증

```text
cargo fmt --check
cargo test parser::hwpx::section::tests::test_rendering_info_quantizes_fractional_matrix_values_like_hwp5 -- --nocapture
cargo build
target/debug/rhwp dump-pages output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-01.hwp
target/debug/rhwp dump-pages output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-02.hwp
target/debug/rhwp dump-pages output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-03.hwp
target/debug/rhwp dump-pages output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hy-001.hwp
```

## 8. 한컴/시각 판정

다음 파일을 한컴 에디터에서 확인한다.

```text
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hy-001.hwp
```

판정표:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hwpx-h-03.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | target |
| `output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/hy-001.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | #974 guard |

## 9. 판정 해석

Stage 36 판정으로 #949의 주 대상이었던 한컴 에디터 파일손상/중단 문제는 해소되었다.

```text
- hwpx-h-03: 파일손상 해소, 2페이지 이후 출력 성공
- hy-001: 파일손상 해소, 2페이지 이후 출력 성공
- h01/h02 guard: 한컴/rhwp-studio 모두 성공 유지
```

이 결과는 정답 HWP byte 비교 결과와 일치한다. 특히 `hwpx-h-03`, `hy-001`의 문제 구간
text-box/picture bundle이 정답 HWP와 byte-equal이 되었고, 한컴 에디터 판정도 성공으로
전환되었다.

## 10. 잔여 관찰

한컴 로딩/저장 contract는 통과했지만, rhwp-studio 조판에는 다음 fidelity 차이가 남아 있다.

```text
- hwpx-h-03: rhwp-studio에서 2페이지 문단과 표 사이의 간격이 한컴 에디터보다 넓다.
- hy-001: rhwp-studio에서 2페이지 문단 다음 엔터 두 번 후 배치되는 표가 한컴보다 더 아래에 배치된다.
```

이 항목은 HWPX -> HWP 저장 contract 실패가 아니라, rhwp-studio의 페이지/문단/표 간격 조판
fidelity 문제로 분리한다. #949의 파일손상 해소와 같은 축으로 섞지 않는다.
