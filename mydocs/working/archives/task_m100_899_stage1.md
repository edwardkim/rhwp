# Task m100 #899 Stage 1 - RED 테스트 확정

## 1. 목적

`business_overview.hwpx`에서 셀 배경색이 지정된 BorderFill이 HWP 저장 시 `무늬없음`으로 정규화되지 않는 문제를 RED 테스트로 고정한다.

Stage 1에서는 production 코드를 수정하지 않는다.

## 2. 추가한 테스트

파일:

```text
tests/hwpx_to_hwp_adapter.rs
```

테스트:

```text
task899_business_overview_cell_backgrounds_use_no_pattern
```

검사 대상:

```text
samples/hwpx/business_overview.hwpx
```

검사 BorderFill:

```text
5, 6, 7
```

이 BorderFill들은 실제 표 셀에서 사용되며 `faceColor`가 있는 셀 배경이다.

## 3. 기대값

HWPX `winBrush`가 다음 조건일 때:

- `faceColor` 있음
- `hatchColor` 있음
- `hatchStyle` 없음

HWP 저장용 IR의 `SolidFill.pattern_type`은 다음이어야 한다.

```text
-1
```

의미:

- 한컴 HWP에서 `무늬없음`

## 4. RED 결과

실행:

```text
cargo test --test hwpx_to_hwp_adapter task899_business_overview_cell_backgrounds_use_no_pattern -- --nocapture
```

결과:

```text
FAILED
```

실패 메시지:

```text
assertion `left == right` failed:
BorderFill #5 has faceColor but no hatchStyle; HWP save must encode no-pattern as -1
left: 0
right: -1
```

## 5. 판정

RED가 의도대로 성립했다.

현재 HWPX 파서는 `winBrush`에서 `hatchStyle`이 없는 경우 `SolidFill::default()`의 `pattern_type=0`을 그대로 유지한다.

다음 Stage 2에서는 `src/parser/hwpx/header.rs`의 `winBrush` 파싱에서 무늬 종류 속성이 없을 때 `pattern_type=-1`로 초기화하는 방향을 우선 검토한다.

## 6. 현재 변경 상태

추적 필요 파일:

```text
samples/hwpx/business_overview.hwpx
tests/hwpx_to_hwp_adapter.rs
mydocs/working/task_m100_899_stage0.md
mydocs/working/task_m100_899_stage1.md
```
