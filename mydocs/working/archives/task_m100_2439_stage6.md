# Task M100-2439 Stage 6 — 저장 flow·들여쓰기 구현과 최종 검증

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-20
- 코드 커밋: `64138965`

> 정정(2026-07-20): 2~3쪽 확대 비교에서 축퇴 헤더 폭, fragment outer margin, padding을
> 제외한 orphan guard 문제가 추가로 확인됐다. 아래 Stage 6의 “최종” 판정은
> [Stage 7](task_m100_2439_stage7.md)의 구현·10쪽 재검증으로 대체한다.

## 1. 구현 결과

### 단일 positive-offset 빈 host RowBreak 표

native HWP5 구조 증거가 일치하는 단일 표 경로에서 실제 painted bottom, outer bottom,
저장 LineSeg 진행량을 flow에 포함했다. 다음 일반 문단에는 strict fit을 한 번만 전달하고,
표 fragment는 실제로 그려진 행 하단을 기준으로 이월한다.

### native HWP5 두 표 visible host

같은 visible host에 있는 zero-offset/positive-offset 두 표를 순차 flow로 소비한다. 두 번째
표의 outer top/bottom과 host LineSeg 간격을 typeset과 layout 양쪽에 반영해 표 그룹 뒤
서명문이 마지막 표 아래에서 시작하게 했다.

### native HWP5 저장 LineSeg 들여쓰기

비합성 full-width 일반 본문 줄의 저장 `LineSeg.column_start`를 권위 시작점으로 적용했다.
표 셀, wrap/control, 번호 control, 합성 LineSeg, HWP3/HWPX 경로는 제외했다.

- 제목 줄: `1900HU` → 63.09px ≈ 47.32pt
- 번호 줄글: `10320HU` → 175.36px = 131.52pt

## 2. 페이지 흐름 검증

최종 `dump-pages`는 정답지와 같은 10쪽이다.

- 4쪽은 `pi=19`에서 끝난다.
- 5쪽은 `pi=20`에서 시작한다.
- 10쪽에는 `pi=90`만 있고
  `5.응급 및 긴급한 상황시 7920으로 연락한다.`를 렌더한다.

페이지 수를 맞추는 하드코딩은 추가하지 않았다. 저장 표·LineSeg flow를 복원한 결과로
마지막 한 줄이 10쪽에 자연스럽게 배치됐다.

## 3. 코드 검증

- focused: 16개 대상, 60 tests, 0 failed, 0 ignored
- `cargo fmt --all -- --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- 전체 CI 성격 검증은 수행하지 않음

## 4. PDF visual sweep

- 결과 루트: `/private/tmp/rhwp-issue2439-sweep-final-20260720`
- 대상: 정답 PDF와 rhwp 1~10쪽
- 페이지 수: 10/10
- 자동 후보: 0/10
- 평균 `pixel_match_percent`: 89.15839%
- 평균 `visual_accuracy_proxy_percent`: 6.21195%
- 최저 `visual_accuracy_proxy_percent`: 2.6602% (3쪽)

실행 환경은 macOS Darwin 25.5.0 arm64이며 sweep에 `--font-path`를 전달하지 않았다.
fontconfig의 `MS바탕`/`바탕` 조회는 Verdana fallback을 반환했고, 한국어 시스템 폰트로
Apple SD Gothic Neo·PCMyungjo 등이 확인됐다. 따라서 아래 자동 잉크 지표에는 한컴 전용
폰트 부재의 영향이 포함되며, 호환성 점수로 해석하지 않는다.

| 쪽 | compare | overlay | review | visual_accuracy_proxy_percent |
|---:|---|---|---|---:|
| 1 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_001.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_001.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_001.png` | 4.45000 |
| 2 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_002.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_002.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_002.png` | 3.72551 |
| 3 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_003.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_003.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_003.png` | 2.66020 |
| 4 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_004.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_004.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_004.png` | 3.01742 |
| 5 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_005.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_005.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_005.png` | 5.84580 |
| 6 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_006.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_006.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_006.png` | 5.62240 |
| 7 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_007.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_007.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_007.png` | 5.57155 |
| 8 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_008.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_008.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_008.png` | 5.59558 |
| 9 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_009.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_009.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_009.png` | 5.34630 |
| 10 | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/compare/compare_010.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/overlay/overlay_010.png` | `/private/tmp/rhwp-issue2439-sweep-final-20260720/issue2439-answer/review/review_010.png` | 20.28470 |

사람이 `review_001.png`부터 `review_010.png`까지 직접 확인했다. 표 하단과 서명문이
겹치지 않고, 번호 줄글 들여쓰기가 적용되며, 10쪽의 마지막 문장도 존재한다.

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 평균 약 6.21%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

## 5. Studio 검증 환경

- 기존 개발 서버 PID: `81399`
- cwd: `rhwp-studio`
- 주소: `http://127.0.0.1:7700`
- HTTP 응답: 200

## 6. 범위

로컬 구현, focused 검증, WASM build, PDF sweep, Studio 서버 확인까지 완료했다. 전체 CI,
remote push, PR 생성은 수행하지 않았다.
