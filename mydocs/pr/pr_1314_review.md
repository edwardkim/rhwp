# PR #1314 검토 — fix(equation): 적분기호 글리프 확대 및 상·하한 밀착 (#1313)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1314
- 작성자: `planet6897`
- 상태: open / draft 아님
- base: `devel`
- head: `task1313-pr` (`fef16435`)
- 이슈: #1313

## 2. 변경 요약

`samples/3-10월_교육_통합_2022.hwp` 9페이지 등에서 적분기호의 상·하한이 기호와 떨어지고,
적분기호가 한컴 2022 기준보다 작게 렌더링되는 문제를 수정한다.

코드 변경:

- `src/renderer/equation/layout.rs`
  - `INTEGRAL_SCALE = 2.5` 추가
  - 적분 `MathSymbol`의 글리프 크기를 `BIG_OP_SCALE`에서 분리
  - `layout_subsup()` 적분 분기에서 상·하한 위치를 글리프 상·하단부에 밀착
- `src/renderer/equation/svg_render.rs`
- `src/renderer/equation/canvas_render.rs`
- `src/renderer/skia/equation_conv.rs`
  - SVG / Canvas / Skia BigOp 경로에서도 적분 전용 스케일 사용

문서 변경:

- `mydocs/plans/task_m100_1313.md`
- `mydocs/plans/task_m100_1313_impl.md`
- `mydocs/working/task_m100_1313_stage1.md`
- `mydocs/working/task_m100_1313_stage2.md`
- `mydocs/report/task_m100_1313_report.md`

## 3. GitHub 상태

- 2026-06-07 확인: 컨트리뷰터가 PR head를 `cad416e3` → `fef16435`로 force-push/rebase했다.
- 최신 base는 현재 `devel`(`054d72fc`)이다.
- 변경 파일 목록과 실제 PR patch 내용은 기존 커밋과 동일하다.
- 리뷰 코멘트: 없음
- PR 코멘트: 없음
- GitHub checks:
  - Build & Test: pass
  - Canvas visual diff: pass
  - CodeQL: pass
  - Rust/JS/Python CodeQL analysis: pass
  - WASM Build: skipping

## 4. 로컬 적용 방식

초기 확인 시점의 PR 브랜치는 현재 `devel`보다 오래된 기준점을 포함했다.
따라서 초기에는 직접 merge하면 최근 release/security/문서 변경을 되돌리는 diff가 생겼다.

이후 컨트리뷰터가 PR을 최신 `devel` 위로 rebase했다.
현재 `local/pr1314-upstream`은 `devel` 바로 위의 1커밋 PR 상태이다.

초기 검토 브랜치 처리:

1. 현재 `devel`에서 `local/pr1314-integration` 생성
2. PR의 실제 커밋 `cad416e3` 1개만 cherry-pick
3. 충돌 없음

현재 upstream 상태:

- `local/pr1314-upstream`: `fef16435`
- `cad416e3`와 `fef16435`의 PR 대상 9개 파일 내용 차이 없음
- `HEAD..local/pr1314-upstream` diff 없음

현재 로컬 검토 브랜치:

```text
local/pr1314-integration
```

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --release --lib equation
cargo build --release
./target/release/rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8 -o output/poc/pr1314-integral
docker compose --env-file .env.docker run --rm wasm
cargo build --release --features native-skia
./target/release/rhwp export-png samples/3-10월_교육_통합_2022.hwp -p 8 -o output/poc/pr1314-integral-png
```

결과:

- `cargo fmt --all -- --check`: pass
- `cargo test --release --lib equation`: 151 passed, 0 failed
- `cargo build --release`: pass
- Docker WASM build: pass
- `pkg/` → `rhwp-studio/public/` WASM 동기화 완료
- `cargo build --release --features native-skia`: pass
- SVG 산출물:
  - `output/poc/pr1314-integral/3-10월_교육_통합_2022_009.svg`
- PNG 산출물:
  - `output/poc/pr1314-integral-png/3-10월_교육_통합_2022.png`

## 6. 검토 의견

수식 파서상 적분은 주로 `MathSymbol(∫) + SubSup` 경로를 타므로, 실제 핵심 수정은
`layout_subsup()`의 적분 분기다. PR은 여기에 적분 전용 스케일과 상·하한 오프셋을 적용한다.

SVG / Canvas / Skia의 `BigOp` 적분 분기에도 같은 스케일을 적용했는데, 현재 주 경로는 아니더라도
렌더러 간 상수 불일치를 막는 방어적 정리로 볼 수 있다.

주의점:

- Canvas/WASM 경로에서는 시각적으로 개선됐지만, SVG export에서는 적분기호 상·하한 배치가 오히려 이상해졌다.
- SVG에서 적분은 `<text font-size="30">∫</text>`로 출력되고, 상·하한은 hard-coded offset으로 배치된다.
  실제 SVG 렌더러가 어떤 수식 폰트 메트릭을 적용하느냐에 따라 적분 글리프의 보이는 bbox가 달라지므로
  Canvas와 SVG가 같은 상·하한 위치로 보장되지 않는다.
- PNG(native Skia) 산출물도 별도 시각 판정 대상이다.
- 적분 모양은 시각 피델리티 이슈이므로, 메인테이너가 SVG/PNG/WASM을 정답 PDF와 직접 비교해 최종 판정해야 한다.

## 7. 권장 처리

권장: **현재 상태로는 보류.**

이유:

- PR은 최신 `devel` 기준으로 rebase되어 merge 구조 문제는 해소됐다.
- 하지만 SVG export 피델리티가 악화되어 renderer 공통 정합 기준을 만족하지 못한다.
- 적분 전용 스케일/offset을 Canvas 기준 magic ratio로 조정하는 방식은 SVG/Skia/Canvas의 폰트 메트릭 차이에 취약하다.

권장 개선 방향:

- 적분기호의 visual bbox를 layout에서 명시적으로 다루거나,
- SVG 경로에서 적분기호를 폰트 `<text>`가 아닌 안정적인 path/shape로 렌더링하거나,
- 최소한 SVG export에서 동일 수식 폰트가 강제/임베딩되는 조건으로 상·하한 offset을 재측정해야 한다.

다음 단계:

1. 메인테이너 시각 판정
   - `output/poc/pr1314-integral/3-10월_교육_통합_2022_009.svg`
   - `output/poc/pr1314-integral-png/3-10월_교육_통합_2022.png`
2. SVG 판정 실패 유지 시 PR에 수정 요청 코멘트 작성
3. 컨트리뷰터 수정 후 재검증
