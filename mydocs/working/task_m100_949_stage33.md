# Task m100 #949 Stage 33 - HWPX lineSegArray vpos 보존 구현

## 1. 목표

Stage 33은 Stage 32에서 확인한 첫 divergence를 직접 수정하는 단계다.

```text
문제:
  HWPX XML에는 paragraph 18 lineSegArray.vertpos=68258이 존재한다.
  그러나 DocumentCore::from_bytes() 이후 61435로 덮어써져 page sequence가 어긋난다.

수정:
  line segment를 실제로 합성/보정한 경우에만 section vpos를 재계산한다.
  명시 lineSegArray가 있는 문서에서는 source vpos를 보존한다.
```

## 2. 수정 내용

수정 파일:

```text
src/document_core/commands/document.rs
tests/hwpx_to_hwp_adapter.rs
```

`reflow_zero_height_paragraphs()`의 section vpos 재계산 조건을 바꿨다.

기존에는 비-TAC `TopAndBottom` table/picture가 있거나 TAC table raw control data가 없으면
section vpos를 넓게 다시 계산했다. 이 조건은 HWPX가 이미 계산해 저장한 `lineSegArray.vertpos`까지
덮어쓰는 부작용이 있었다.

변경 후에는 다음 경우에만 vpos를 재계산한다.

```text
1. `needs_line_seg_reflow(para)`가 true라서 line segment를 새로 합성한 경우
2. TAC table 때문에 첫 line segment height를 실제로 키운 경우
```

즉 “object가 존재한다”는 이유가 아니라 “본문 line segment가 실제로 변경되었다”는 사실을 기준으로
후속 문단 vpos를 재계산한다.

## 3. 자동 검증

실행:

```text
cargo test --test hwpx_to_hwp_adapter task949_stage33_hwpx_h03_explicit_lineseg_vpos -- --nocapture
```

결과:

```text
test task949_stage33_hwpx_h03_explicit_lineseg_vpos_preserved_on_load ... ok
test task949_stage33_hwpx_h03_explicit_lineseg_vpos_survives_adapter_export_reload ... ok
```

추가 guard:

```text
cargo test --test hwpx_to_hwp_adapter stage5_all_three_samples_recover_via_unified_entry_point -- --nocapture
cargo test --test hwpx_to_hwp_adapter stage6_verify_recovered_for_all_three_samples -- --nocapture
cargo fmt --check
cargo build
```

결과:

```text
모두 성공
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hy-001.hwp
```

파일 크기와 rhwp 재로드:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 374,784 bytes | ok, sections=2, pages=9 |
| `hwpx-h-02.hwp` | 32,256 bytes | ok, sections=2, pages=10 |
| `hwpx-h-03.hwp` | 38,400 bytes | ok, sections=2, pages=9 |
| `hy-001.hwp` | 88,576 bytes | ok, sections=1, pages=2 |

## 5. h03 vpos 확인

생성된 `hwpx-h-03.hwp`의 paragraph 18:

```text
ls[0]: ts=0, vpos=68258, lh=600, th=600, bl=510, ls=272, cs=0, sw=48188
```

`ir-diff` 확인:

```text
rhwp ir-diff samples/hwpx/hwpx-h-03.hwpx \
  output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-03.hwp \
  -s 0 -p 18

결과:
차이 0건
```

`dump-pages` 확인:

```text
paragraph 18: page 1 마지막 표
paragraph 19: page 2 시작
```

Stage 32에서 관찰한 `paragraph 19가 page 1에 남는 현상`은 rhwp 내부 페이지네이션 기준으로 해소됐다.

## 6. 수동 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-03.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 글상자 전까지만 출력, 실패 | 성공 | target |
| `output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hy-001.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 마지막 표전까지만 출력, 실패 | 정상 | #974 guard |

## 7. 해석

이번 단계에서 닫힌 것은 `hwpx-h-03`의 rhwp 페이지네이션 첫 divergence다.

```text
닫힌 문제:
  HWPX explicit lineSegArray.vertpos 보존
  paragraph 18/19 page boundary 복원

아직 한컴 수동 판정이 필요한 문제:
  h03 한컴 파일손상이 이 vpos 보존으로 같이 해소되는지
  남는다면 #825/#832/#836 SHAPE_COMPONENT 또는 drawText 주변 contract가 별도 원인인지
```

따라서 다음 판단은 Stage 33 산출물의 한컴 에디터 판정 결과를 기준으로 한다.

## 8. 수동 판정 해석

작업지시자 판정으로 Stage 33의 의미가 분리됐다.

```text
1. hwpx-h-01/hwpx-h-02 guard는 한컴과 rhwp-studio 모두 성공했다.
2. hwpx-h-03은 이번 stage에서 처음으로 rhwp-studio 정상 조판에 도달했다.
3. 따라서 Stage 32에서 추적한 lineSegArray.vertpos 보존 문제는 실제로 rhwp 페이지네이션 실패의
   원인이었다.
4. 그러나 hwpx-h-03과 hy-001은 한컴 에디터에서 여전히 파일손상이다.
```

중요한 결론:

```text
rhwp-studio 조판 실패와 한컴 파일손상은 같은 원인이 아니었다.
```

Stage 33으로 닫힌 축:

```text
HWPX explicit lineSegArray.vertpos 보존
paragraph 18/19 page boundary
hwpx-h-03 rhwp-studio 정상 조판
```

남은 축:

```text
한컴 HWP5 loader가 2페이지 글상자 또는 그 직전 record에서 요구하는 binary contract
```

특히 `hy-001`은 #974에서 다뤘던 글상자 내부 그림 케이스이고, 이번 산출물에서도 한컴 파일손상이
발생했다. 따라서 다음 단계는 `hwpx-h-03`과 `hy-001`의 공통 실패 지점인 drawText/text-box 계열
HWP5 record contract를 비교해야 한다.

다음 단계에서 금지할 해석:

```text
1. rhwp-studio에서 정상 조판하므로 한컴 파일손상도 해결됐다고 보는 것
2. table-axis 또는 lineSegArray 축을 다시 probe하는 것
3. h01/h02 guard가 이미 성공한 상태에서 넓은 adapter 변경을 넣는 것
```

다음 단계의 우선 질문:

```text
한컴 에디터가 drawText/text-box 또는 그 내부 picture control을 로드할 때 요구하지만,
rhwp-studio는 관대하게 렌더링하는 HWP5 record contract는 무엇인가?
```
