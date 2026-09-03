# Task M100 #6672 최종 보고서

## 결과

full renderer의 live paragraph line-flow 경계와 분리된 renderer callable을 제품
graph에서 제거했다.

- 호출자 0개인 `inject_footnote_markers`와
  `missing_lineseg_legacy_bullet_requires_regenerated_space_metric`은 삭제했다.
- 기존 unit test가 직접 검증하는 `tokenize_paragraph`, `is_line_end_forbidden`,
  `split_composed_line_by_width`, `estimate_regenerated_line_text_width`는
  `#[cfg(test)]` 경계로 제한했다.
- target 전용 renderer 함수와 public legacy API는 건드리지 않았다.
- 이슈 피드백 뒤 renderer 전체로 확대해 56개 test-owned callable을
  `#[cfg(test)]`로 제한하고 저장소 전체에서 호출자도 없는 43개 callable은 삭제했다. 최종 세
  target 공통 dead-code 진단에는 함수·메서드가 0건이고, 비-callable
  field/type/constant 34건만 남는다.

코드 동작은 바꾸지 않았다. `exam_science.hwp` 1쪽 SVG의 변경 전후 SHA-256은 모두
`0b4275739388c41cd663c749b9841181e18a89fe68998b2003e07886a028c252`였고
byte comparison도 일치했다.

## Debugger 증거

`scripts/debug/renderer.py`는 LLDB command `rhwp-render-flow` 하나를 제공한다.
일반 `SpaceMetric::Stored` 호출은 계속하고, full renderer가 frame-aware
`SpaceMetric::HalfCell`을 고른 decision-bearing divergence에서만 정지한다.

fresh LLDB에서 다음 경로를 실행했다.

```text
rhwp export-svg samples/76076_regulatory_analysis.hwp -p 34
```

실제 정지는 `layout_paragraph_in_frame_impl`의 live tokenizer 호출점이었다.
값은 `HalfCell`, EBU 0, KBU 1이었고, stack은
`resolve_stored_line_segs_in_frame` → `recompose_stored_lines_in_frame...` →
`HeightMeasurer::measure_paragraph` → `DocumentCore::paginate_pass`였다.
이 세션에는 lldb-mcp가 노출되지 않아 shell LLDB fallback을 사용했다.

## 검증

- Gestell: initial candidate 및 aggressive 99-callable 최종 candidate PASS
- native, wasm32, native-skia `-W dead-code`: renderer callable 교집합 0건
- Rust unit-tier policy: 4,205 tests, cfg support items 85, PASS
- Rust lint 묶음: fmt, native Clippy, wasm32 Clippy, workspace build,
  workspace all-target Clippy, generated manifest check 모두 PASS
- release-test Nextest: 8,986 passed, 46 skipped
- Native Skia lib: 3,930 passed, 13 ignored
- Native Skia missing-picture gate: 2/2 PASS
- Native Skia direct-PDF gate: 4/4 PASS
- locked wasm-pack web release build PASS
- Studio renderer contract: 61/61 PASS
- Studio production TypeScript/Vite build PASS
- debugger script Python compile 및 72-column 검사 PASS
- `git diff --check` PASS

## 계획 대비 교정

초기 후보였던 `regenerated_half_space_width`는 full binary 재빌드에서
`SpaceMetric::HalfCell`의 실제 호출자로 확인됐다. 후보에서 즉시 제외하고 제품 함수로
복원했다. 또한 첫 두 표본이 specialized line-flow 결정을 만들지 않아 LLDB breakpoint가
정지하지 않았으며, production 주석이 지정한 `76076_regulatory_analysis.hwp`로 입력을
바꿔 실제 `HalfCell` 정지를 확보했다.

## 산출물

- Branch: `renderer/full-render-dead-code`
- Base: `origin/devel` `b6b9384ed`
- Merge는 수행하지 않는다.
