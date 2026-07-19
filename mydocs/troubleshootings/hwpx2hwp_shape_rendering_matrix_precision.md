---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# HWPX to HWP SHAPE_COMPONENT rendering matrix precision

## 1. 요약

HWPX `renderingInfo`를 HWP5 `SHAPE_COMPONENT`의 rendering matrix block으로 저장할 때,
소수 matrix 값을 XML 문자열에서 파싱한 `f64` 그대로 저장하면 한컴 에디터에서 파일손상으로
이어질 수 있다.

한컴 HWP oracle에서 확인한 규칙은 다음이다.

```text
HWPX XML value: 0.723629
rhwp 기존 저장: f64(0.723629)
한컴 정답 저장: f64(f32(0.723629))
```

HWP5 record 슬롯은 8바이트 double이지만, 한컴 HWPX -> HWP 저장 경로는 fractional matrix
값을 먼저 `f32` 정밀도로 양자화한 뒤 `f64`로 승격해 double slot에 기록한다.

## 2. 발생 증상

Task M100-949에서 다음 증상이 확인되었다.

```text
- HWPX 원본은 rhwp-studio에서 정상 렌더링된다.
- rhwp가 HWPX -> HWP로 저장한 파일도 rhwp-studio에서는 열리거나 조판된다.
- 한컴 에디터에서는 특정 지점에서 파일손상 판정을 내리고 이후 페이지 출력을 중단한다.
```

특히 `hwpx-h-03.hwp`는 2페이지 글상자 전후에서 파일손상 판정이 발생했다. `hy-001.hwp`도
비슷하게 2페이지 후반부에서 한컴 에디터가 중단했다.

## 3. 대상 record

문제는 다음 HWP5 record tuple에서 확인되었다.

```text
CTRL_HEADER
SHAPE_COMPONENT
LIST_HEADER
PARA_HEADER
CTRL_HEADER
SHAPE_COMPONENT
SHAPE_PICTURE
CTRL_HEADER
SHAPE_COMPONENT
SHAPE_PICTURE
SHAPE_RECTANGLE
```

여기서 마지막까지 남은 차이는 `SHAPE_COMPONENT` 안의 rendering matrix block이었다.

HWP5 `SHAPE_COMPONENT`의 해당 영역은 다음 구조다.

```text
rendering_count: u16
translation matrix: double[6]
repeat rendering_count times:
  scale matrix: double[6]
  rotation matrix: double[6]
```

스펙상 각 matrix 원소는 double로 저장된다. 그러나 한컴 oracle은 XML 소수값을 그대로
`f64`로 기록하지 않았다.

## 4. 원인

`SHAPE_COMPONENT` 필드 디코딩 결과는 사람 눈에는 모두 같아 보였다.

예:

```text
scale[0] oracle   [0.723629, 0, 310, 0, 0.723636, 0]
scale[0] generated[0.723629, 0, 310, 0, 0.723636, 0]
```

하지만 byte-level diff에서는 하위 바이트가 달랐다.

```text
oracle:    f64(f32(0.723629)) = 0x1.727f800000000p-1
generated: f64(0.723629)      = 0x1.727f8012dfd69p-1
```

즉 값 표시를 6자리 정도로 하면 같아 보이지만, HWP5 payload hash는 달라진다.

한컴 에디터는 이 차이를 단순 조판 오차로 처리하지 않고, 해당 `SHAPE_COMPONENT` subtree에서
파일손상 또는 출력 중단으로 판정할 수 있다.

## 5. 확정 규칙

HWPX `renderingInfo`를 HWP5 `raw_rendering`으로 materialize할 때 다음 규칙을 적용한다.

```text
- 정수 matrix 값: 그대로 저장한다.
- 소수 matrix 값: f32로 한 번 양자화한 뒤 f64로 승격해 저장한다.
```

예:

```text
1        -> f64(1)
310      -> f64(310)
0.723629 -> f64(f32(0.723629))
1.287342 -> f64(f32(1.287342))
```

이 규칙은 HWP5 저장용 `raw_rendering` payload에 대한 규칙이다. 화면 렌더러의 내부 계산이나
IR의 의미값 전체에 무차별 적용하면 안 된다.

## 6. 구현 위치

현재 구현 위치:

```text
src/parser/hwpx/section.rs
parse_rendering_info()
```

관련 테스트:

```text
parser::hwpx::section::tests::test_rendering_info_quantizes_fractional_matrix_values_like_hwp5
```

Stage 36 구현은 `parse_rendering_info()`에서 XML matrix 값을 읽을 때 HWP5 raw rendering
payload로 저장될 값을 한컴 oracle 정밀도에 맞춘다.

## 7. 검증 결과

Stage 36에서 다음 문제 구간이 정답 HWP와 byte-equal이 되었다.

### `hwpx-h-03`

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

### `hy-001`

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

작업지시자 한컴 에디터 판정:

```text
hwpx-h-01: 성공
hwpx-h-02: 성공
hwpx-h-03: 성공
hy-001: 성공
```

따라서 Task M100-949의 한컴 파일손상/중단 문제는 이 규칙까지 반영했을 때 해소되었다.

## 8. 진단 절차

비슷한 문제가 발생하면 다음 순서로 확인한다.

```text
1. 한컴에서 마지막으로 정상 출력된 위치를 기록한다.
2. 해당 지점 직후의 HWP5 record tuple을 oracle/generated로 추출한다.
3. tag, level, size, count, tail, reference가 같은지 확인한다.
4. 필드 표시상 같은데 payload hash가 다르면 byte-level diff를 수행한다.
5. diff offset이 SHAPE_COMPONENT rendering matrix block이면 f32 -> f64 양자화 여부를 확인한다.
6. 정답 HWP와 byte-equal이 되는지 확인한 뒤 한컴 에디터 판정을 받는다.
```

주의:

```text
필드 디코더가 반올림해서 보여주는 값만 믿지 않는다.
payload hash가 다르면 반드시 byte-level diff를 확인한다.
```

## 9. 관련 문서

```text
mydocs/working/task_m100_949_stage34.md
mydocs/working/task_m100_949_stage35.md
mydocs/working/task_m100_949_stage36.md
mydocs/troubleshootings/hwpx2hwp-rule.md
```
