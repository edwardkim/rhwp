# Task M100 #6672 Stage 1

## 목적

full renderer의 실제 문단 구성 owner와 제거 후보의 제품 liveness를 구현 전에
확정한다.

## LLDB 증거

`scripts/debug/renderer.py`의 `rhwp-render-flow`를 fresh LLDB에 import하고 다음 native
full-render 경로를 실행했다.

```text
rhwp export-svg samples/76076_regulatory_analysis.hwp -p 34
```

one-shot breakpoint는 `line_breaking.rs:2580`의
`layout_paragraph_in_frame_impl`이 `SpaceMetric`을 확정한 뒤 live tokenizer를 부르는
지점에 정지했다. command는 일반 `Stored` 호출을 계속하고 specialized decision만
잡았으며, decisive 값은 `space_metric=HalfCell`, `english_break_unit=0`,
`korean_break_unit=1`이었다. stack은 `resolve_stored_line_segs_in_frame` →
`recompose_stored_lines_in_frame_with_known_square_band` →
`HeightMeasurer::measure_paragraph` → `DocumentCore::paginate_pass`로 이어졌다. 단순
breakpoint resolve가 아니라 실제 full render의 정지와 frame을 확인했다.

## Liveness 증거

native, wasm32, `native-skia`의 product lib를 각각 `-W dead-code`로 컴파일했다.
세 구성 모두 같은 여섯 callable을 죽은 항목으로 판정했다. 저장소 검색에서는
`split_composed_line_by_width`, `tokenize_paragraph`, `is_line_end_forbidden`만 unit
test가 직접 호출했고, 나머지는 호출자가 없었다.

## 구현 판정

- 무호출 두 함수는 삭제한다.
- test가 직접 검증하는 세 함수와 전용 helper는 `#[cfg(test)]`로 제품에서 제외한다.
- target 전용 함수와 public compatibility API는 보존한다.

초기 closure에 포함했던 `regenerated_half_space_width`는 full binary 재빌드에서
`SpaceMetric::HalfCell`의 실제 호출자로 확인돼 즉시 제품 경계로 복원했다.
