# Task m100 #949 Stage 33 계획 - HWPX 명시 lineSegArray vpos 보존

## 1. 배경

Stage 32에서 `hwpx-h-03`의 rhwp 페이지네이션 첫 divergence가 `SHAPE_COMPONENT`가 아니라
`paragraph 18`의 `PARA_LINE_SEG.vertical_pos`임을 확인했다.

```text
oracle/source HWPX: paragraph 18 vpos = 68258
generated load path: paragraph 18 vpos = 61435
```

중요한 점은 `samples/hwpx/hwpx-h-03.hwpx`의 XML에 이미 `vertpos="68258"`가 존재한다는 것이다.
따라서 이 단계의 목표는 새로운 shape 계약을 추정하는 것이 아니라, HWPX가 제공한 명시
`lineSegArray.vertpos`를 `DocumentCore::from_bytes()`가 덮어쓰지 않도록 막는 것이다.

## 2. 구현 원칙

```text
1. 명시 lineSegArray가 있는 문서의 page-computed vpos는 source 값을 우선 보존한다.
2. line segment를 실제로 합성하거나 높이를 보정한 경우에만 section vpos를 재계산한다.
3. 비-TAC TopAndBottom table/picture가 있다는 이유만으로 section 전체 vpos를 재계산하지 않는다.
4. Stage 17/18에서 성공한 table-axis guard는 유지한다.
```

## 3. 수정 대상

```text
src/document_core/commands/document.rs
tests/hwpx_to_hwp_adapter.rs
```

## 4. 검증 대상

생성 경로:

```text
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/
```

수동 판정 파일:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 5. 자동 검증

```text
cargo test --test hwpx_to_hwp_adapter task949_stage33_hwpx_h03_explicit_lineseg_vpos -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage5_all_three_samples_recover_via_unified_entry_point -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage6_verify_recovered_for_all_three_samples -- --nocapture
cargo fmt --check
cargo build
```

## 6. 판정 기준

성공:

```text
1. `hwpx-h-03.hwpx` 로드 직후 paragraph 18 vpos가 68258로 유지된다.
2. HWPX -> HWP 저장 후 재로드해도 paragraph 18 vpos가 68258로 유지된다.
3. `hwpx-h-03.hwp`의 rhwp 페이지네이션에서 paragraph 19가 page 2로 이동한다.
4. h01/h02/hy guard가 한컴 또는 rhwp-studio에서 회귀하지 않는다.
```

실패:

```text
1. `hwpx-h-03`이 여전히 paragraph 19를 page 1에 남긴다.
2. h01/h02 table-axis guard가 깨진다.
3. hy-001 #974 guard가 깨진다.
```
