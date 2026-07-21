# PR #2523 검토 - DOM flow-image 그림 색효과

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#2523](https://github.com/edwardkim/rhwp/pull/2523) |
| 작성자 | @planet6897 |
| base / 검토 head | `devel` / `65904a5e7fe3ba0c4ea890a33d638fc66055ff7b` |
| 체리픽 순서 | 3 (`65904a5`) |
| 충돌 | 없음 |
| 검토 시점 원 PR 상태 | `BEHIND`; 기존 head CI 전체 성공 |

## 변경 및 판단

- Canvas2D의 DOM flow-image `<img>` 경로가 WASM canvas 경로와 달리 그림 효과를 잃는 문제를
  `composeImageFilter()`로 보정한다.
- 회색조, 흑백, 밝기, 명암 및 baked watermark 제외 조건은
  `src/renderer/web_canvas.rs::compose_image_filter`의 CSS 의미와 일치한다.
- Studio frontend 변경으로 판정했다.

## 검증

- `npx tsc --noEmit`: 통과
- `npm test`: 456/456 통과
- `npm run build`: 통과. Vite의 500KB chunk 경고만 있으며 새 오류는 없다.
- 변경 전용 코드 검토에서 `grayScale`/`pattern8x8`은 `grayscale(100%)`, `blackWhite`는
  `grayscale(100%) contrast(1000%)`, brightness/contrast는 0이 아닌 경우에만 누적하는 것을
  확인했다.
- PR 작성자의 `pr-149.hwp` headless Chrome 검증 결과(3개 DOM flow image의 filter가
  `none`, `grayscale(100%)`, `grayscale(100%) contrast(1000%)`)와 코드 경로가 일치한다.

## 리스크와 권고

- 현행 `flow-image-clip.test.ts`는 clip 계보만 검사하고 새 `composeImageFilter()`의 효과별
  기대 문자열은 직접 pin하지 않는다. 이번 수정은 작고 기존 Studio 전체 테스트는 통과했으므로 merge
  blocker는 아니지만, 회색조·흑백·brightness/contrast·baked watermark 회귀 단위 테스트를
  후속 보강하는 것을 권고한다.
- 현재 head는 `devel`보다 뒤처져 있으므로 merge 전 최신 `devel` 위 head update와 새 CI가 필요하다.
- 위 조건을 충족하면 **수용 가능**이다.
