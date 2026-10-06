---
kind: working
status: done
canonical: mydocs/plans/archives/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 Stage 7 — 통합 배율 SVG 아이콘 교체

## 문제

Stage 6에서 우측 통합 배율 버튼의 돋보기를 18×18px로 맞췄지만, 원과 손잡이를 별도의 CSS
의사 요소로 그린 결과 실제 화면에서 영문 `Q`처럼 보였다. 왼쪽 축소·확대 스프라이트와 외곽 크기는
같아졌지만 돋보기 형태와 선 연결이 안정적이지 않았다.

## 원인

- 원은 `border-radius`를 적용한 사각형, 손잡이는 회전한 `border-top`으로 각각 렌더됐다.
- 서로 다른 의사 요소의 서브픽셀 배치와 겹침이 브라우저 배율에 따라 달라질 수 있었다.
- 원 안쪽에서 손잡이가 시작해, 별도 도형의 겹침이 돋보기보다 `Q`의 꼬리처럼 읽혔다.

## 수정

- CSS 의사 요소 조합을 `viewBox="0 0 18 18"` 인라인 SVG로 교체했다.
- 원과 손잡이는 `currentColor`, 2px 선, 둥근 선 끝·모서리를 공유한다.
- SVG는 `flex: 0 0 18px`로 고정해 확대·축소 스프라이트와 같은 실제 상자를 유지한다.
- 배율 텍스트 36px와 통합 버튼 68px 고정 폭은 변경하지 않았다.

## Test-first 증거

구현 전에 기존 CSS 의사 요소를 금지하고 18×18px SVG 구조·선 스타일을 요구하는 focused test를
추가했다. `.stb-zoom-menu-icon` 규칙과 SVG 마크업이 없어 6개 중 1개가 실패하는 것을 확인했다.
구현 후 같은 테스트 6개가 모두 통과했다.

## 브라우저 검증

`http://127.0.0.1:7700/`에서 복구 안내를 닫고 새 문서의 상황 선을 확인했다.

- 왼쪽 축소 스프라이트: 18×18px, `y=700.5px`
- 우측 통합 배율 SVG: 18×18px, `y=700.5px`
- SVG: `stroke-width=2px`, `stroke-linecap=round`, `flex=0 0 18px`
- 통합 버튼: 68×20px

원과 손잡이가 하나의 SVG 좌표계에서 렌더되어 `Q`처럼 보이던 형태가 사라지고, 좌측 확대·축소
돋보기와 같은 크기와 수직 기준선을 유지했다.

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-fit.test.ts` | 6/6 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,122 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |
| `cargo fmt --all && cargo fmt --all -- --check` | 소스 브랜치에서 제외하는 `tests/generated/regression_suite_*.rs` 32개가 없어 실행 중단; Rust 변경 없음 |
