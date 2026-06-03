# Task #1116 Stage 9 보고서 — p3 3줄 문단 LINE_SEG 간격 재확인

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: p3 3줄 문단의 `LINE_SEG.ls` 반영 여부 진단 출력 정정

## 1. 작업지시자 새 단서

p3의 3줄 문단들은 `LINE_SEG.vpos` 간격이 2016 HU이다.

```text
lh=1300 HU
ls=716 HU
lh + ls = 2016 HU
```

그런데 기존 `dump-pages` 출력은 p74~p77 문단을 다음처럼 보여줬다.

```text
h=55.8 (sb=3.8 lines=52.0 sa=0.0)
```

이 값은 3줄의 `lh=1300 HU`만 합산한 52.0px라서, `ls=716 HU`가 빠진 것처럼 보였다.

## 2. 확인 결과

실제 typeset/layout 경로는 p74~p77의 줄간격을 이미 반영하고 있었다.

- `lh_sum=52.0px`
- `ls_sum=28.6px`
- `lines=80.6px`
- `spacing_before=3.8px`
- `total=84.4px`

SVG의 p74 첫 줄에서 p75 첫 줄까지의 y 간격도 약 80.64px로, `3 * (1300 + 716) HU`와 일치한다.

따라서 이번 단서는 실제 배치 로직 누락보다 진단 출력이 `line_heights`만 보여 오판을 유도한 문제로 정리했다.

## 3. 구현

수정 파일:

```text
src/document_core/queries/rendering.rs
src/renderer/typeset.rs
tests/issue_1116.rs
```

변경:

1. `dump-pages`의 FullParagraph 높이 표시를 `line_height + line_spacing` 기준으로 정정했다.
2. 출력에 `lh`와 `ls` 분해값을 함께 표시해, p3 3줄 문단이 어떤 값으로 계산되는지 바로 보이게 했다.
3. `RHWP_TYPESET_DRIFT`의 `vpos_h` 비교식을 `last.vpos + last.lh + last.ls - first.vpos`로 바꿔 현재 pagination/layout 모델과 맞췄다.
4. `tests/issue_1116.rs`에 p74 진단 출력이 `h=84.4`, `lines=80.6`, `lh=52.0`, `ls=28.6`을 포함하는지 가드를 추가했다.

## 4. 수정 후 출력

```text
FullParagraph  pi=74  h=84.4 (sb=3.8 lines=80.6 lh=52.0 ls=28.6 sa=0.0)
FullParagraph  pi=75  h=84.4 (sb=3.8 lines=80.6 lh=52.0 ls=28.6 sa=0.0)
FullParagraph  pi=76  h=84.4 (sb=3.8 lines=80.6 lh=52.0 ls=28.6 sa=0.0)
FullParagraph  pi=77  h=84.4 (sb=3.8 lines=80.6 lh=52.0 ls=28.6 sa=0.0)
```

`RHWP_TYPESET_DRIFT=1` 기준:

```text
TYPESET_DRIFT_PI: pi=74 ... fmt_total=84.4 vpos_h=80.6 diff=+3.8
```

`diff=+3.8`은 해당 문단의 `spacing_before`이다.

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1116 -- --nocapture
cargo build --bin rhwp
git diff --check
```

## 6. 다음 판단

p3가 여전히 한컴보다 위에 보인다면, p74~p77의 `ls=716 HU` 누락이 아니라 다음 항목을 봐야 한다.

1. p3 페이지 body 영역 시작 y 또는 상/하 여백 차이.
2. `spacing_before`가 vpos에 이미 흡수된 문단에서 중복 또는 부족하게 처리되는지.
3. Web Canvas와 SVG의 실제 glyph baseline 차이.
4. 한컴 캡처 기준의 마지막 보이는 항목이 rhwp와 같은 문단 범위인지.
