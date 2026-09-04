# PR #1314 검토 — fix(equation): 적분기호 path 렌더링 정합 (#1313, #1317)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1314
- 작성자: `planet6897`
- 상태: open / draft 아님
- base: `devel`
- head: `task1313-pr` (`5e725e55`)
- 이슈: #1313, #1317

## 2. 변경 요약

1차 검토에서 메인테이너가 지적한 문제:

- Canvas/WASM에서는 적분기호 크기와 상·하한 배치가 개선됐지만, SVG export에서는 상·하한 위치가 어긋남.
- 기존 구현은 `MathSymbol(∫) + SubSup`에서 적분기호를 폰트 `<text>`로 렌더링하고, 상·하한은 Canvas 폰트 메트릭 기준 offset으로 배치했다.
- SVG viewer/renderer가 다른 폰트를 대체하면 적분 글리프 bbox가 달라져 SVG/Canvas/Skia 간 피델리티가 깨질 수 있었다.

컨트리뷰터 수정 내용:

- 적분기호를 폰트 글리프가 아닌 stroke path로 렌더링.
- `src/renderer/equation/layout.rs`에 `IntegralGeom` / `integral_geom()`을 추가하여 적분 path의 visual bbox와 상·하한 attach point를 SSOT로 공유.
- `src/renderer/equation/svg_render.rs`, `canvas_render.rs`, `src/renderer/skia/equation_conv.rs`가 같은 geometry를 사용.
- ∑/∏ 계열은 기존 text 렌더링 경로 유지.

## 3. GitHub 상태

- PR head: `5e725e55557a6468c683a34e04b2e6003d0349b5`
- PR base SHA: `931423669097ef4d3653bf28f5cbe37007f74dc4`
- 현재 로컬 `devel`: `292ec8f0`
- PR upstream branch는 현재 `devel`보다 오래된 base 위에 있으므로, 그대로 비교하면 최근 #1319/#1320/#1133 변경을 되돌리는 큰 diff가 생긴다.
- 실제 검토는 현재 `local/devel`에서 `local/pr1314-v2-integration` 브랜치를 만들고 PR 단일 커밋을 cherry-pick하여 진행했다.

GitHub Actions:

- CI: success (`27083478672`)
- Render Diff: success (`27083478670`)
- CodeQL: success (`27083478680`)

## 4. 로컬 적용 방식

검토 브랜치:

```text
local/pr1314-v2-integration
```

적용 커밋:

```text
68b78f16 fix(equation): 적분기호(∫)를 path로 렌더하여 SVG/Canvas/Skia 상·하한 정합 (#1313, #1317)
```

현재 `local/devel` 기준 실제 변경 범위:

- 문서 12개 추가
- 수식 렌더러 4개 파일 수정
  - `src/renderer/equation/layout.rs`
  - `src/renderer/equation/svg_render.rs`
  - `src/renderer/equation/canvas_render.rs`
  - `src/renderer/skia/equation_conv.rs`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --release --lib equation
cargo build --release
./target/release/rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8 -o output/poc/pr1314-v2-integral-svg
cargo build --release --features native-skia
./target/release/rhwp export-png samples/3-10월_교육_통합_2022.hwp -p 8 -o output/poc/pr1314-v2-integral-png
```

결과:

- `cargo fmt --all -- --check`: pass
- `cargo test --release --lib equation`: 151 passed, 0 failed
- `cargo build --release`: pass
- `cargo build --release --features native-skia`: pass
- SVG 산출물:
  - `output/poc/pr1314-v2-integral-svg/3-10월_교육_통합_2022_009.svg`
- PNG 산출물:
  - `output/poc/pr1314-v2-integral-png/3-10월_교육_통합_2022.png`

SVG 구조 확인:

- 산출물에서 적분기호 텍스트 출력 `>∫<`는 발견되지 않음.
- 적분기호는 `stroke-linecap="round"`가 있는 `<path>`로 출력됨.

## 6. 검토 의견

1차 리뷰에서 지적했던 핵심 문제는 “적분기호의 visual bbox와 상·하한 attach point가 renderer별 폰트 메트릭에 의존한다”는 점이었다.

이번 수정은 적분기호를 path로 고정하고, layout/SVG/Canvas/Skia가 같은 `integral_geom()`을 공유하도록 바꾸었다. 따라서 구조적으로는 1차 리뷰에서 요구한 “SVG와 native PNG/Skia가 같은 기준을 사용해야 한다”는 조건을 만족하는 방향이다.

주의점:

- PR upstream branch는 현재 `devel`보다 오래된 base이므로, merge 전에 최신 `devel` 기준 rebase 또는 maintainer-side integration merge가 필요하다.
- PR에는 contributor 작업 문서가 다수 포함되어 있으므로, 최종 수용 시 문서 포함 여부를 유지할지 판단이 필요하다.

## 7. 권장 처리

권장: **수용 가능**.

이유:

- 코드 리뷰상 1차 보류 사유였던 SVG 폰트 메트릭 의존 문제가 제거됐다.
- 수식 단위 테스트와 일반/native-skia 빌드가 통과했다.
- GitHub Actions도 success 상태다.
- 메인테이너 SVG/PNG/WASM 시각 판정이 통과됐다.

메인테이너 시각 판정:

- `output/poc/pr1314-v2-integral-svg/3-10월_교육_통합_2022_009.svg`
- `output/poc/pr1314-v2-integral-png/3-10월_교육_통합_2022.png`
- WASM/rhwp-studio: 통과

## 8. PR 코멘트 초안

```markdown
2차 검토 완료했습니다.

1차 리뷰에서 지적했던 SVG export의 적분기호 상·하한 불일치 문제는 이번 수정에서 적분기호를 path 기반으로 렌더링하고, layout/SVG/Canvas/Skia가 `integral_geom()`을 공유하도록 정리하면서 구조적으로 해소된 것으로 확인했습니다.

로컬 검증:

- `cargo fmt --all -- --check`
- `cargo test --release --lib equation`
- `cargo build --release`
- `cargo build --release --features native-skia`
- `samples/3-10월_교육_통합_2022.hwp` 9페이지 SVG/PNG export

GitHub Actions도 CI / Render Diff / CodeQL 모두 success 상태임을 확인했습니다.

메인테이너 SVG/PNG/WASM 시각 판정도 통과했습니다. 수용 절차로 진행하겠습니다. 감사합니다.
```
