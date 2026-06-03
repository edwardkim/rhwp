# Task #1116 Stage 10 보고서 — p3 세로 위치 후속 분석

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 소스 수정 전 원인 재분류 완료

## 1. 승인 범위

작업지시자 승인에 따라 Stage 10 분석을 진행했다. 이번 단계에서는 소스 수정 없이 p3 세로 위치 원인을 다시 분류했다.

## 2. Stage 9 이후 확인

Stage 9에서 `dump-pages`의 문단 상세 출력은 `line_spacing` 포함으로 정정했다.

p3의 3줄 문단 p74~p77은 다음처럼 계산된다.

```text
lh=1300 HU * 3 = 3900 HU = 52.0px
ls=716 HU * 3 = 2148 HU = 28.6px
lines = 6048 HU = 80.6px
spacing_before = 568 HU = 3.8px
total = 84.4px
```

따라서 `LINE_SEG.vpos` 피치 2016 HU는 실제 측정/배치에 이미 반영되어 있다.

## 3. 남은 차이의 정체

`RHWP_TYPESET_DRIFT=1`로 p69~p86을 다시 보면 모든 문단에서 `diff`가 해당 문단의 `spacing_before`와 일치한다.

예:

```text
pi=74 sb=3.8 lines=3 lh_sum=52.0 ls_sum=28.6 fmt_total=84.4 vpos_h=80.6 diff=+3.8
pi=81 sb=1.9 lines=1 lh_sum=17.3 ls_sum=10.4 fmt_total=29.6 vpos_h=27.7 diff=+1.9
```

즉 p3 내부에서 `used`가 `LINE_SEG.vpos + lh + ls` 흐름보다 큰 차이는 누락된 `ls`가 아니라 문단 앞 간격의 별도 가산이다.

## 4. 단 요약 `hwp_used`의 남은 진단 문제

단 요약은 아직 `compute_hwp_used_height()`가 마지막 줄의 `line_spacing`을 제외해 계산한다.

현재 p3:

```text
used=874.5px
hwp_used≈803.5px
diff=+71.0px
```

마지막 항목 p86 기준:

```text
vpos + lh      = 58960 + 1300 = 60260 HU = 803.5px
vpos + lh + ls = 58960 + 1300 + 780 = 61040 HU = 813.9px
```

`hwp_used` 요약에도 `ls`를 포함하면 diff는 약 +60.6px로 줄고, 이 값은 p69~p86의 `spacing_before` 누적과 거의 같다.

따라서 단 요약도 Stage 9와 같은 방향으로 정정해야 한다. 이건 시각 배치 변경이 아니라 진단 도구 정정이다.

## 5. p3가 여전히 한컴보다 위로 보이는 경우의 후보

작업지시자 피드백은 “3페이지 내용이 한컴오피스보다 위로 올라가 보임”이었다. 이 전제라면 다음 점이 중요하다.

1. `spacing_before`를 제거하는 보정은 p3를 더 위로 올리므로 이 피드백과 반대 방향이다.
2. p3의 `LINE_SEG.ls`는 이미 반영되고 있으므로, 추가로 `ls`를 더하면 중복이다.
3. p3의 실제 끝 항목은 pi=86이고, pi=87 빈 문단은 p3에 배치되지 않는다.
4. p4는 pi=88부터 시작한다. 한컴 캡처가 p3에서 pi=87 또는 pi=88 근처까지 보이는지 여부를 확인해야 한다.
5. 한컴보다 전체가 낮아야 한다면 body 시작 y, 페이지 상단 여백, 또는 페이지 분할 허용 높이 쪽이 더 가까운 후보이다.

## 6. 다음 구현 후보

소스 수정 승인 후의 최소 후보:

1. `compute_hwp_used_height()`가 마지막 줄의 `line_spacing`을 포함하도록 정정한다.
2. `dump-pages` 단 요약 명칭을 `vpos_used`처럼 바꿔, 실제 pagination `used`와 문서 `LINE_SEG` 흐름 비교라는 의미를 분명히 한다.
3. p3 시각 보정은 한컴 3mm 캡처에서 `pi=86`, `pi=87`, `pi=88` 중 어디까지 같은 페이지에 보이는지 확인한 뒤 별도 구현한다.

## 7. 실행 명령

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
RHWP_TYPESET_DRIFT=1 target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 3
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 87
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 88
```

## 8. 승인 요청

다음 단계에서는 먼저 진단 도구 정정만 진행하는 것이 안전하다.

승인 요청:

- `compute_hwp_used_height()`의 마지막 줄 기준을 `vpos + lh + ls`로 정정.
- `tests/issue_1116.rs`에 p3 단 요약이 `ls` 포함 기준으로 출력되는지 가드 추가.
- 시각 배치 변경은 아직 하지 않음.
