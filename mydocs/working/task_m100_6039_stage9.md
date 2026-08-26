---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 Stage 9 — 돋보기 실그림·슬라이더 트랙 보정

## 문제

통합 배율 버튼의 SVG와 좌우 확대·축소 스프라이트가 모두 18×18px 상자를 사용했지만, 실제 돋보기
그림은 SVG 쪽이 더 크고 굵게 보였다. Stage 8에서 range thumb를 명시적으로 스타일링한 뒤에는 일부
브라우저에서 기본 슬라이더 가로선이 사라졌다.

## 원인

- 18px는 클릭 상자 크기일 뿐 내부 artwork 크기를 보장하지 않는다.
- 스프라이트 원본의 확대·축소 그림은 18px 셀 안에서 비투명 픽셀이 2~15px에 있어 실제 경계가
  14×14px였다.
- 기존 SVG는 2px 선과 16px에 가까운 원·손잡이 경계를 사용해 더 크게 보였다.
- WebKit thumb에 `-webkit-appearance: none`을 적용하면서 기본 range track 렌더링에 의존할 수 없게
  됐지만, 별도 runnable track 규칙이 없었다.

## 수정

- SVG 원을 `cx=6.5`, `cy=6.5`, `r=4.5`, 손잡이를 `9.75→15.25` 좌표로 축소했다.
- 선 굵기를 2px에서 1.5px로 줄여 스프라이트의 14px 실제 그림과 시각 크기를 맞췄다.
- 18×18px SVG 상자와 68px 통합 버튼 폭은 유지했다.
- 슬라이더에 `appearance: none`을 명시하고 WebKit runnable track과 Firefox range track을 각각
  2px 높이와 디자인 토큰 `--ui-border-strong`으로 그렸다.
- 12px thumb는 2px 트랙 중앙에 오도록 WebKit에서 `-5px`에 해당하는 계산식으로 정렬했다.

## Test-first 증거

구현 전 focused test는 새 SVG 내부 좌표와 1.5px 선 계약을 만족하지 못했고, WebKit·Firefox track
규칙도 없어 7개 중 2개가 실패했다. 구현 후 같은 7개가 모두 통과했다.

## 브라우저·스프라이트 검증

- `icon_small_ko.svg`를 1:1 PNG로 렌더하고 배율 아이콘 4개 셀을 분리했다.
- 확대·축소·맞춤 아이콘 모두 18px 셀 내부의 비투명 픽셀 경계가 14×14px였다.
- `http://127.0.0.1:7700/`에서 우측 SVG 실그림이 좌측 확대·축소 돋보기와 같은 시각 크기로 보이고,
  슬라이더의 2px 가로 트랙이 thumb 뒤에 연속해서 표시되는 것을 확인했다.

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-fit.test.ts` | 7/7 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,123 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |
| `cargo fmt --all && cargo fmt --all -- --check` | 소스 브랜치에서 제외하는 `tests/generated/regression_suite_*.rs` 32개가 없어 실행 불가; Rust 변경 없음 |
