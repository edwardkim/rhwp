---
kind: working
status: active
canonical: mydocs/plans/task_m100_3591.md
last_verified: 2026-07-30
---

# Task #3591 Stage 1~3 보고 — 팬 정책 구현·잔차 규명·검증

## Stage 1 — 팬 정책 구현

`virtual-scroll.ts`:

- 상수 `PAN_SPACE_RATIO=0.25` / `MIN_PAN_SPACE=80` / `MAX_PAN_SPACE=240` 도입.
- `horizontalPanSpace(viewportWidth, contentWidth)` 신설:
  `contentWidth <= viewportWidth ? 0 : clamp(viewportWidth × 0.25, 80, 240)`.
- `applyHorizontalPanSpace` 가 이 값을 쓰도록 교체. 팬 0 이면 `pageLefts`(단일 열 −1 =
  CSS 중앙 정렬)와 `totalWidth` 를 그대로 두어 **브라우저 자연 중앙 정렬이 회복**된다.

계약 테스트(`tests/virtual-scroll-horizontal-pan.test.ts`): 콘텐츠가 창에 들어갈 때 팬 0·
가로 스크롤 없음, clamp 경계(상한 240 / 비율 225·150 / 하한 80), 4K 최대화에서 스크롤
영역이 문서폭+480 으로 수렴, 그리드 자체 중앙 유지.

### 파생 발견 — 앵커 오버슈트 (같은 커밋에서 정정)

팬이 얇아지자 줌 앵커 계산이 스크롤 가능 범위를 넘는 것이 드러났다(실측 632 vs 최대
550). 종전에는 두꺼운 팬이 이를 흡수했다. 브라우저가 `scrollLeft` 대입 시 클램프하므로
화면은 안전하나 이후 계산이 어긋난 값을 근거로 삼는 것을 막기 위해
`CanvasView.clampScrollLeft()` 를 추가하고 두 대입 지점에 적용했다.
`tests/zoom-anchor.test.ts` 의 소스 계약도 함께 갱신.

## Stage 2 — 잔차 −7.5px 규명: **측정식 오류였다 (수정 없음)**

Stage 1 후 단일 열에서 첫 페이지 중심이 −7.5px 로 일정하게 관측됐다. 세 지표로 분리한
결과:

| 지표 | 값 | 의미 |
|---|---:|---|
| `pageInContent` | 0.1 | scroll-content 안에서는 완벽 중앙 |
| `pageInClient` (clientWidth 기준) | **1.0** | 실제 표시 영역 기준 중앙 정상 |
| `pageInViewport` (rect.width 기준) | −6.5 | 스크롤바 15px 포함 폭으로 잰 값 |

즉 −7.5px 는 **하니스가 뷰포트 중앙을 `getBoundingClientRect().width`(세로 스크롤바
포함)로 계산한 아티팩트**다. 스크롤바 폭의 절반이 그대로 관측됐다. 작업지시자 육안
판정("창 가운데 위치")이 맞았고 계측이 틀렸다. 코드 수정 대상 없음 — 하니스 측정식을
`clientWidth` 기준으로 교정했다.

## Stage 3 — 검증

교정 하니스(`task3591_verify.mjs`)로 zoom 100% → 줌아웃 4단계 → 줌인 4단계 → 100% 복귀
전 구간 측정:

| 검증 | 결과 |
| --- | --- |
| `slErr`(중앙 대비 scrollLeft 오차) | 전 구간 **≤ 1.3px** (종전 −546 ~ −99) |
| 단일 열 `pageOff` | 전 구간 **≤ 1.5px** |
| 팬 0 구간 | `hasHScroll=false`, `contentRatio=100%`, `left:50%`(CSS 중앙 정렬 회복) |
| 팬 적용 구간 | `left` 명시값이 전 페이지 동일 — #3377 좌표 동기화 무회귀 |
| 하니스 자동 판정 | **VERDICT: PASS, violations 0** |
| studio `npm test` | 681/681 |
| studio build / 확장 build.mjs | 성공 |
| `cargo test --profile release-test --tests` + fmt | 373 바이너리 ok · 통과 (Rust 비접촉) |

### 첫 로드 before/after (dev 7700, 창 1229, SO-SUEOP)

| 항목 | 전 | 후 |
|---|---:|---:|
| 콘텐츠 총폭 | 3291.7px | **833.7px** |
| 스크롤 영역 대비 내용 | 25% | **100%** |
| 가로 스크롤바 | 있음 | **없음** |

작업지시자 dev 육안 확인: 가로 스크롤바 소멸, 창 가운데 정렬, **축소 다중 페이지
모드에서도 중앙 정렬 정상**.

## 남은 것 (Stage 4)

작업지시자 4K 실기동 최종 시각 판정 — 확장 재빌드 완료(`rhwp-chrome/dist`).

## Stage 4 — 실환경 판정에서 발견한 추가 결함 (그리드 팬 오적용)

작업지시자 4K 실기동 1차 판정: ①첫 로드 수평 스크롤 소멸 OK ③400% 줌인 후 가로 이동
가능·답답하지 않음 OK — 그러나 **②줌아웃 시 정렬 불안정(중앙에서 왼쪽 이동)** 이
보고됐다(관측 지점: 25%).

### 근인

`layoutGrid` 의 base 는 `max(gridWidth + marginLeft*2, viewportWidth)` 라 **그리드에서는
항상 창 폭 이상**이다. 그래서 팬 조건 `contentWidth <= viewportWidth` 가 그리드 첫
진입(zoom 0.5)에서 거짓이 되어 팬 240×2 가 붙었고, 스크롤 여지 482px 이 생겨 문서가
중앙에서 밀렸다.

| zoom | 수정 전 totalW | 스크롤 여지 | 수정 후 |
|---|---:|---:|---:|
| 0.5 (그리드 진입) | 1711 | **482px** | 1231 (2px) |
| 0.35 / 0.25 | 1229 | 0 | 1229 |

### 수정

`horizontalPanSpace` 첫 줄에 `if (this.gridMode) return 0;` — 그리드는 자체 marginLeft 가
중앙을 잡으므로 팬이 불필요하다. 회귀 가드 추가:
`grid mode never receives pan space at any zoom` (0.5~0.25 전 구간 스크롤 여지 없음).

### 판정

- studio `npm test` 682/682, studio·확장 빌드 갱신.
- **작업지시자 실환경 재판정: 통과** (2026-07-30) — "문제가 해결되었습니다".

### 범위 밖으로 남긴 것

25% 에서 광폭 페이지가 열 안에서 좌측에 치우쳐 보이는 것은 그리드가 **최대 페이지 폭
기준 균일 열**로 배치하는 의도된 설계다(표 형태 정렬). 결함이 아니므로 수정하지 않았다.
셀 내부 중앙 정렬로 바꾸려면 별도 이슈로 다룬다.
