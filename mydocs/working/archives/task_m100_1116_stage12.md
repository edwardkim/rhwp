# Task #1116 Stage 12 보고서 — p3 하단 위치 차이 재분류

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 소스 수정 없이 후속 원인 후보 분석 완료

## 1. 승인 범위

작업지시자 지시에 따라 Stage 11 이후 분석을 계속 진행했다.

이번 단계에서는 소스 수정 없이 `sample16` p3/p4의 페이지 분할과 p83 줄 수 차이를 비교했다.

## 2. 핵심 결론

p3 하단이 한컴오피스 기준보다 위로 보이는 현상은 `LINE_SEG.line_spacing` 누락보다 p83의 줄 수 차이가 더 직접적인 후보이다.

기본 HWP5 변환본은 p83이 1줄이다.

```text
samples/hwp3-sample16-hwp5.hwp
p83: h=29.6, vpos=52720
p84: vpos=54800
p85: vpos=56880
p86: vpos=58960
```

한컴 2022 저장본은 같은 위치의 p83이 2줄이다.

```text
samples/hwp3-sample16-hwp5-2022.hwp
p83: h=57.4, vpos=52720..54800
p84: vpos=56880
p85: vpos=58960
p86: vpos=61040
```

따라서 p84~p86은 2022 저장본에서 기본 HWP5 변환본보다 정확히 한 줄 피치인 2080 HU만큼 내려간다.

```text
2080 HU = 27.7px = lh 1300 HU + ls 780 HU
```

이 차이는 사용자가 말한 "전체 내용이 하단까지 내려오지 않음"에 매우 가깝다.

## 3. 비교 결과

기본 HWP5 변환본 p3:

```text
단 0 (items=19, used=874.5px, hwp_used≈813.9px, diff=+60.6px)
p83 h=29.6 vpos=52720
p84 h=29.6 vpos=54800
p85 h=29.6 vpos=56880
p86 h=29.6 vpos=58960
```

한컴 2022 저장본 p3:

```text
단 0 (items=19, used=902.2px, hwp_used≈841.6px, diff=+60.6px)
p83 h=57.4 vpos=52720..54800
p84 h=29.6 vpos=56880
p85 h=29.6 vpos=58960
p86 h=29.6 vpos=61040
```

원본 HWP3 p3:

```text
단 0 (items=20, used=901.5px, hwp_used≈839.1px, diff=+62.5px)
p83 h=29.6 vpos=52529
p84 h=29.6 vpos=54609
p85 h=29.6 vpos=56689
p86 h=29.6 vpos=58769
p87 h=29.6 vpos=60849
```

원본 HWP3은 p83이 1줄이지만 p87 빈 문단까지 p3에 들어간다. 반면 기본 HWP5 변환본은 p87이 p3에 배치되지 않고 p4가 p88부터 시작한다. 2022 저장본은 p83이 2줄이 되면서 p87 빈 문단이 p4 첫 항목으로 밀린다.

## 4. p83 LINE_SEG 직접 비교

기본 HWP5 변환본:

```text
ls[0]: ts=0, vpos=52720, lh=1300, ls=780, cs=4000, sw=46024
```

한컴 2022 저장본:

```text
ls[0]: ts=0,  vpos=52720, lh=1300, ls=780, cs=4000, sw=46024
ls[1]: ts=64, vpos=54800, lh=1300, ls=780, cs=4000, sw=46024
```

원본 HWP3:

```text
ls[0]: ts=0, vpos=52529, lh=1300, ls=780, cs=0, sw=51024
```

기본 HWP5 변환본과 2022 저장본은 p83의 단락 스타일/문단 폭 필드는 거의 같지만, 저장된 `LINE_SEG` 줄 수가 다르다. 즉 한컴 2022에서 해당 문단이 재저장되며 BCP 문장이 2줄로 확정된 것으로 보인다.

## 5. 판단

Stage 9~11에서 확인한 내용:

1. p74~p77의 3줄 문단은 `lh + ls` 기준 2016 HU 피치를 이미 반영한다.
2. `dump-pages` 상세와 단 요약의 `line_spacing` 진단은 정정됐다.
3. p3 기본 HWP5와 2022 저장본의 하단 위치 차이는 p83 1줄/2줄 차이만으로 설명된다.

따라서 다음 구현을 바로 세로 간격 누락 보정으로 가면 중복 보정 위험이 있다.

## 6. 다음 후보

다음 단계에서 안전한 최소 구현 후보:

1. p83 줄 수 차이를 회귀 테스트로 고정해, 기본 HWP5와 2022 저장본의 레이아웃 차이를 명시한다.
2. 2022 저장본의 p83이 2줄이므로 p86이 한 줄 피치만큼 내려가는 사실을 테스트 이름과 문서에 남긴다.
3. 실제 시각 보정은 작업지시자 기준 캡처가 기본 HWP5 변환본인지, 2022 저장본인지 확인한 뒤 별도 승인으로 진행한다.

## 7. 실행 명령

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2022.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 3
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2022.hwp -p 3
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 3
cargo test lineseg_compare --lib -- --nocapture
```

## 8. 승인 요청

다음 단계 소스 수정 승인 요청:

- `tests/issue_1116.rs`에 p83 1줄/2줄 차이와 p86 vpos 차이를 고정하는 진단 테스트를 추가한다.
- 시각 배치 알고리즘은 아직 변경하지 않는다.
