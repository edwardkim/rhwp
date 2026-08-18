---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5515 검토 - 디코드 불가 그림의 그림-없음 표시

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5515](https://github.com/edwardkim/rhwp/pull/5515) / planet6897 |
| base / contributor head | devel / `4962c0dd8a743170c17f4b4542a7463361cb32ae` |
| 가시성 branch | `review/planet6897-20260818` |
| local cherry-pick | `6ae24756a` |
| 선행 체리픽 | #5483, local `54510eb99` |
| 원격 상태 | OPEN, 비 draft, MERGEABLE, BLOCKED |
| 검토 기준 | `upstream/devel@e5ef2620bd469aa2d0118097c4d04f63cfdacdc3` 위에 #5483 후 #5515 누적 |

## 변경 범위

디코드할 수 없는 텍스트 EPS/AI 등 그림 바이트가 SVG에 PostScript data URI로 남아
브라우저·resvg·Skia에서 빈칸이 되던 경로를 `MissingPicture`로 바꿨다.
`is_displayable_image_data`와 공통 unusable 판정을 추가하고, 본문 그림 경로에서는 같은
bbox의 placeholder 노드로 교체해 캡션·테두리·후속 흐름을 보존한다. SVG에는 점선 테두리와
그림-없음 아이콘을 추가하고 WebCanvas의 점선 표현과 맞췄다.

적용된 주요 파일은 `src/renderer/image_resolver.rs`, `src/renderer/layout/utils.rs`,
`src/renderer/layout/picture_footnote.rs`, `src/renderer/svg.rs`,
`src/renderer/svg_layer.rs`, `src/renderer/web_canvas.rs`이다.

## 체리픽 및 충돌

- #5483을 먼저 적용한 동일한 `review/planet6897-20260818` branch에 #5515 source head를 적용했다.
- #5515 적용 과정에 충돌은 없었다.
- contributor source history는 rewrite하지 않았고, 통합 결과만 로컬에서 검증했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare` 실행 후 `--check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `git diff --check upstream/devel...HEAD` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 실행해 **7300 passed, 38 skipped, 8 slow**을 확인했다. 전체 실행 시간은 452.801초였고 release-test 빌드 포함 종료 시간은 15분 11초였다.

비공개 기준 HWP fixture를 현재 release-test binary로 직접 렌더링했다.

- screen: `target/pr-review/release-test/rhwp export-svg ... --page 0 --profile screen`
- print: `target/pr-review/release-test/rhwp export-svg ... --page 0 --profile print`
- 두 결과 모두 3쪽 문서의 1쪽을 생성했다.
- screen SVG에는 `stroke-dasharray="2 2"`, 점선 테두리, 그림-없음 표시가 있었고
  `data:application/postscript`는 남지 않았다.
- print SVG에는 placeholder 표시가 없었고 `data:application/postscript`도 남지 않았다.
- 원시 결과는 `/tmp/pr5515-screen-20260818/`와 `/tmp/pr5515-print-20260818/`에 생성했다.

이는 한글 2022의 편집 화면 동작인 점선·아이콘과 인쇄 경로의 빈칸 동작을 각각 확인한
결과다. 실제 그림 바이트를 복원하지 않고, 디코드 불가 그림의 표시 정책만 검증했다.

## 남은 범위와 판정

차단 결함은 발견하지 못했다. 이슈 [#5513](https://github.com/edwardkim/rhwp/issues/5513)은
아직 OPEN이며, malformed PNG/JPEG처럼 헤더는 유효하지만 실제 디코드가 실패하는 바이트를
추가 변환 없이 판정하지 않는 것은 PR 본문에 적힌 성능·범위 제한이다.

또한 `is_displayable_image_data`를 직접 호출하는 단위 테스트와 비공개 텍스트 EPS fixture를
자동 회귀 fixture로 고정한 테스트는 별도 보완 여지가 있다. 현재 통합 branch의 전체 nextest와
실제 fixture 렌더링은 통과했으므로 이 항목은 보류 결함이 아니라 테스트 커버리지 후속 범위로
기록했다.

GitHub에서는 이 시점에 두 source branch 모두 required check가 보고되지 않았고 상태가
BLOCKED였다. 따라서 이 문서는 로컬 통합 검토 기록이며, 원 PR 승인·병합은 수행하지 않았다.
