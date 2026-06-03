# Task #1116 Stage 11 보고서 — dump-pages 단 요약 vpos 사용량 정정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 진단 도구 정정 및 회귀 테스트 완료

## 1. 승인 범위

작업지시자 승인에 따라 Stage 10에서 제안한 진단 도구 정정만 적용했다.

이번 단계에서는 p3 시각 배치 자체를 변경하지 않았다.

## 2. 원인

Stage 9에서 문단 상세 출력은 `line_spacing` 포함으로 정정했지만, `dump-pages` 단 요약의 `hwp_used`는 여전히 마지막 줄의 `line_spacing`을 제외했다.

기존 p3 요약:

```text
used=874.5px, hwp_used≈803.5px, diff=+71.0px
```

이 `hwp_used`는 p86 마지막 줄을 다음처럼 계산한 값이다.

```text
vpos + lh = 58960 + 1300 = 60260 HU = 803.5px
```

하지만 현재 renderer/typeset의 줄 진행 모델은 `lh + ls`이다.

```text
vpos + lh + ls = 58960 + 1300 + 780 = 61040 HU = 813.9px
```

## 3. 구현

수정 파일:

```text
src/document_core/queries/rendering.rs
tests/issue_1116.rs
```

변경:

1. `compute_hwp_used_height()`의 일반 마지막 줄 기준을 `vpos + lh + ls`로 정정했다.
2. vpos-reset 직전 줄 기준도 `prev.vpos + prev.lh + prev.ls`로 정정했다.
3. `tests/issue_1116.rs`에 p3 단 요약이 `hwp_used≈813.9px`, `diff=+60.6px`를 출력하는지 가드를 추가했다.

## 4. 수정 후 출력

```text
단 0 (items=19, used=874.5px, hwp_used≈813.9px, diff=+60.6px)
```

이제 단 요약의 diff도 Stage 10 분석처럼 문단앞 간격 누적에 가까운 값으로 읽힌다.

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1116 -- --nocapture
cargo build --bin rhwp
git diff --check
```

확인:

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
```

출력:

```text
used=874.5px, hwp_used≈813.9px, diff=+60.6px
```

## 6. 다음 판단

p3 시각 배치가 여전히 한컴보다 위로 보이면, 이제 `LINE_SEG.ls` 누락이나 `hwp_used` 진단식 문제는 제외하고 봐야 한다.

다음 후보는 한컴 3mm 캡처에서 p3 하단에 실제로 어느 문단까지 보이는지 확인한 뒤, 페이지 분할 경계 또는 body 시작 y를 비교하는 것이다.
