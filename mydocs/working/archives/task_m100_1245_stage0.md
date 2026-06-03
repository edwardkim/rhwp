# Task M100 #1245 Stage0 — 착수 및 원인 분석

## 작업 지시

- 2026-06-02: 작업지시자가 [#1245](https://github.com/edwardkim/rhwp/issues/1245) 해결 착수를 지시했다.
- 대상은 `3-09월_교육_통합_2022.hwp` 7쪽 하단 `문25)`/`문26)` 주변 그림/도형 객체와 문단 흐름 불일치다.

## 현재 상태

- 기준 브랜치: `upstream/devel` `914ee139`
- 작업 브랜치: `local/task_m100_1245`
- 오늘할일: `mydocs/orders/20260602.md`에 #1245 진행 항목 추가
- 수행 계획서: `mydocs/plans/task_m100_1245.md`
- 구현 계획서: `mydocs/plans/task_m100_1245_impl.md`

## 열린 PR 확인

착수 시점의 열린 PR:

- #1242 `Task #1201: HWPX masterpage idRef 연결 보강`
- #1241 `미주 연속 인라인 수식 다행 병합 해소`
- #1240 `미주(해설) 다줄 문단 줄간격 간헐적 좁음 수정`
- #1235 `수식 큰 연산자(Σ/∏/∫) 피연산자 간격 추가`
- #1234 `폰트 충실도: 한컴 돋움 폴백을 Noto Sans KR ExtraLight로`
- #1170 `chore: harden repo hygiene`

레이아웃/미주/수식 관련 PR과의 회귀 가능성을 이후 검증에서 고려한다.

## 초기 판단

이번 문제는 `Square`/`TopAndBottom` 그림 흐름, `LINE_SEG.vertical_pos`, 페이지 하단 overflow 판정의 결합 문제일 가능성이 있다.

Stage0에서는 소스 수정 없이 다음을 먼저 확인한다.

1. `dump-pages`로 7쪽 page item 목록과 하단 항목 위치 확인
2. `dump`로 `문25)`/`문26)` 주변 paragraph, line segment, picture/shape common attr 확인
3. `export-svg`와 `rsvg-convert`로 7쪽 비교 PNG 생성
4. PDF 7쪽과 RHWP 결과의 하단 객체 위치 차이 기록

## Stage0 확인 결과

### 7쪽 덤프

명령:

```bash
cargo run --bin rhwp -- dump-pages samples/3-09월_교육_통합_2022.hwp -p 6
```

확인된 주요 문단:

- 왼쪽 단 `문25)` host: `pi=386`
  - 그림: `ci=11`, `wrap=Square/어울림`, `tac=false`, `vert=문단`, `align=Right`
  - line segment:
    - `ls[0].vertical_pos=31648`, `sw=26788`
    - `ls[1].vertical_pos=34470`, `sw=26788`
    - `ls[2].vertical_pos=36227`, `sw=26788`
    - `ls[3].vertical_pos=37804`, `sw=13166`
- 오른쪽 단 `문28)` host: `pi=420`
  - 그림: `ci=9`, `wrap=Square/어울림`, `tac=false`, `vert=문단`, `align=Right`
  - line segment 후반부에서 좁은 줄이 시작한다.

### 시각 비교 자료

RHWP SVG/PNG:

```bash
cargo run --bin rhwp -- export-svg samples/3-09월_교육_통합_2022.hwp -p 6 -o output/task1245_stage0 --show-control-codes --debug-overlay
rsvg-convert output/task1245_stage0/3-09월_교육_통합_2022_007.svg -o output/task1245_stage0/3-09월_교육_통합_2022_007.png
```

PDF 보조 비교:

```bash
pdftoppm -f 7 -l 7 -png -r 144 pdf/3-09월_교육_통합_2022.pdf output/task1245_stage0/pdf/page
```

비교 결과:

- PDF 7쪽에서는 `문25)` 타원 그림이 `문25)` 본문 옆 중간 높이에 배치된다.
- RHWP debug PNG에서는 같은 `pi=386 ci=11` 그림이 페이지 하단 밖으로 내려간다.
- `pi=420 ci=9` 그림도 페이지 하단 쪽으로 과하게 내려가며, 오른쪽 단 하단 흐름이 PDF와 달라진다.

## 원인 후보

`src/renderer/layout.rs`의 `square_wrap_first_narrow_line_vpos_px`는 `Square/어울림` 그림이 문단 중간부터 본문을 감싸는 경우, 처음 좁아지는 `LINE_SEG.vertical_pos`를 그림 상단 보정값으로 사용한다.

현재 구현은 좁아지는 줄의 `vertical_pos` 값을 그대로 px로 변환해 `para_base_y`에 더한다.

하지만 `3-09월_교육_통합_2022.hwp` 7쪽 `pi=386`에서는 `LINE_SEG.vertical_pos`가 문단 내부 상대값이 아니라 이미 페이지/구역 흐름 기준 값처럼 저장되어 있다. 이 값을 `para_base_y`에 그대로 더하면 `para_base_y + absolute_vpos`가 되어 그림이 페이지 하단 밖으로 밀린다.

따라서 후보 수정은 다음이다.

- `Square/어울림` 그림 상단 보정은 `first_narrow.vertical_pos` 자체가 아니라 `first_narrow.vertical_pos - first_line.vertical_pos`를 우선 사용한다.
- 다만 기존 task 1209의 page 8 `문29)` 회귀 가드처럼 `first_line.vertical_pos=0`인 문서는 현재 동작을 유지해야 하므로, 기준 line segment를 명확히 분리한다.
- 수정 후 `issue_1209_2022_page8_question29_square_picture_starts_at_wrap_line`, `issue_1209_2022_sep_page13_question19_square_picture_wraps_following_text`, 이번 #1245 신규 가드가 함께 통과해야 한다.

## 승인 대기

2026-06-02: 작업지시자가 Stage0 분석 후 수정 진행을 승인했다.

이후 구현/검증 기록은 `mydocs/working/task_m100_1245_stage1.md`에 남긴다.
