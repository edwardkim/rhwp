# PR #1019 검토 메모 — PageBackground fill mode / RealPic watermark tone

- PR: [#1019](https://github.com/edwardkim/rhwp/pull/1019)
- 제목: `Task #975: Fix PageBackground fill mode and RealPic watermark tone`
- 작성자: `postmelee`
- base: `devel`
- head: `task-975-page-background-fill-mode` (`3257bc6a`)
- 관련 이슈: #975, #976
- 검토일: 2026-05-28

## 1. PR 범위

이 PR은 다음 두 축을 함께 처리한다.

```text
1. PageBackground/BorderFill 이미지 fill_mode(Center 등)가 SVG/Web Canvas에서 무시되고
   페이지 전체로 stretch 렌더링되던 문제
2. effect=RealPic, brightness=-50, contrast=70 색상 워터마크 preset의 tone/opacity 정합
```

추가로 SVG/Web Canvas 경로의 PCX white transparency, non-RealPic watermark opacity,
셀/도형 배경 ImageFill tone 전달까지 함께 정리한다.

## 2. 이전 수정 요청 반영 여부

2026-05-20 메인테이너 요청사항 기준 확인:

| 요청 | 확인 결과 | 비고 |
|---|---|---|
| 최신 devel 기준 rebase | 반영 | PR base는 `5a1c645a`, 현재 로컬 최신 `devel` 위 merge-test도 충돌 없음 |
| RealPic helper를 `image_resolver.rs`로 이동 | 반영 | tone bake helper가 `src/renderer/image_resolver.rs`에 있음 |
| SVG PageBackground watermark/effect/opacity 정합 | 반영 | `render_page_background_image()` 추가 |
| SVG white area transparency 정합 | 반영 | PCX -> PNG 변환이 image resolver 경로로 통합 |
| fixture 명시 | 반영 | `samples/143E433F503322BD33.hwp`, `samples/253E164F57A1BC6934-empty.hwp` 추가 |

## 3. 변경 파일 요약

핵심 구현 파일:

```text
src/renderer/image_resolver.rs
src/renderer/render_tree.rs
src/renderer/style_resolver.rs
src/renderer/layout.rs
src/renderer/layout/shape_layout.rs
src/renderer/layout/table_layout.rs
src/renderer/svg.rs
src/renderer/web_canvas.rs
tests/issue_1019.rs
```

추가 샘플:

```text
samples/143E433F503322BD33.hwp              72K
samples/253E164F57A1BC6934-empty.hwp       370K
```

문서:

```text
mydocs/plans/task_m100_975.md
mydocs/plans/task_m100_975_impl.md
mydocs/working/task_m100_975_stage1.md
mydocs/working/task_m100_975_stage3.md
mydocs/working/task_m100_975_stage4.md
mydocs/report/task_m100_975_report.md
```

## 4. 로컬 검증

최신 `devel` 위 임시 브랜치:

```text
local/pr1019-merge-test
```

검증 결과:

```text
git diff --cached --check
  success

cargo fmt --all -- --check
  success

cargo test --lib renderer::svg::tests
  success, 31 passed

cargo test --test issue_1019 --test issue_514 --test issue_938 --test issue_1156_rowbreak_fragment_fit
  success, 11 passed

cargo check --target wasm32-unknown-unknown --lib
  success

cargo check
  success

cargo test --lib realpic_watermark
  success, 3 passed
```

SVG 판정 후보 산출물:

```text
output/poc/pr1019-page-background/143/143E433F503322BD33.svg
output/poc/pr1019-page-background/253-empty/253E164F57A1BC6934-empty_001.svg
output/poc/pr1019-page-background/253-empty/253E164F57A1BC6934-empty_002.svg
```

## 5. 검토 의견

현재 변경은 이전 보류 사유였던 SVG 경로 정합 누락을 해소한다.

특히 다음 구조가 적절하다.

```text
1. ImageFill tone 속성을 style resolver -> layout -> render tree까지 보존
2. SVG/Web Canvas가 PageBackground에서도 fill_mode helper를 사용
3. RealPic tone bake를 image_resolver에 모아 SVG/Web Canvas가 같은 픽셀 변환을 공유
4. #514/#938/#976 경로를 함께 테스트해 PCX/JPEG watermark 회귀를 가드
```

주의할 점:

```text
1. RealPic tone 값은 공식 스펙이 아니라 샘플 기반 근사값이다.
2. `render_tree.rs`에 tone policy 상수가 추가되었다. 후속 정리 때 image policy 모듈로
   분리할 여지는 있으나, 이번 PR 수용을 막을 수준은 아니다.
3. 최종 수용 전 rhwp-studio WASM 빌드 후 메인테이너 시각 판정이 필요하다.
```

## 6. 메인테이너 시각 판정

```text
2026-05-28 통과
```

판정 대상:

```text
output/poc/pr1019-page-background/143/143E433F503322BD33.svg
output/poc/pr1019-page-background/253-empty/253E164F57A1BC6934-empty_001.svg
output/poc/pr1019-page-background/253-empty/253E164F57A1BC6934-empty_002.svg
rhwp-studio WASM bundle
```

후속 관찰:

```text
이번 샘플은 2단 구성 문서이며, 아직 미구현된 차트 컨트롤의 페이지 내 레이아웃이
한컴 에디터와 다르다.

표 다음 차트가 현재 단 영역에 들어가지 못할 때 한컴은 오른쪽 단으로 넘겨 배치한다.
rhwp-studio는 차트 컨트롤 조판이 아직 구현되지 않아 이 동작을 재현하지 못한다.

이 문제는 #1019의 PageBackground fill_mode/RealPic watermark tone 범위와 독립적인
차트 컨트롤/다단 레이아웃 후속 이슈 #1156으로 분리한다.
```

## 7. 권장 처리

권장안:

```text
수용
```

절차:

```text
1. merge-test 브랜치의 수용 커밋을 local/devel에 병합한다.
2. local/devel을 devel에 병합한다.
3. devel에서 compile/test/WASM 빌드를 확인한다.
4. 통과 시 원격 devel에 push한다.
5. PR #1019와 관련 이슈 #975를 close한다.
```
