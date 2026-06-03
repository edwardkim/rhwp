# Task #1116 Stage 14 보고서 — 3mm 격자 정답지 기준 p3 세로 배치 재수정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 한컴 PDF 정답지 기준 재수정 및 검증 완료

> 후속: 작업지시자가 한컴오피스 원본 HWP3 3mm 격자 화면을 추가 제공했다.
> Stage 15에서 HWP3 원본도 같은 문단 흐름 `spacing_before` 보정 경로를 타도록 확장했다.

## 1. 재확인

작업지시자가 제공한 3mm 격자 스크린샷과 `pdf/hwp3-sample16-hwp5-2022.pdf`를 기준으로 다시 비교했다.

Stage 13의 p83 2줄 합성은 `hwp3-sample16-hwp5-2022.hwp`의 raw `LINE_SEG`만 본 판단이었다. 그러나 실제 정답지 PDF에서는 p83 BCP 항목이 단일 시각 줄에 가깝게 렌더링된다. 따라서 p83 합성은 철회했다.

## 2. 실제 원인

p74~p77의 내부 3줄 줄간격은 한컴 PDF와 거의 같다.

문제는 3줄 문단이 끝난 뒤 다음 문단으로 넘어가는 문단 앞 간격이다.

기존 RHWP:

```text
p74 첫 줄 -> p75 첫 줄: 80.6px
```

한컴 PDF 정답지:

```text
p74 첫 줄 -> p75 첫 줄: 약 88.3px
```

차이는 약 7.6px이며, HWP3-origin 변환본의 `spacing_before=568 HU`와 일치한다.

## 3. 수정

수정 파일:

```text
src/renderer/mod.rs
src/renderer/height_measurer.rs
src/renderer/layout.rs
src/renderer/layout/paragraph_layout.rs
tests/issue_1116.rs
```

변경:

1. `style_resolver`의 기존 HWP3-origin spacing `/2` 정책은 유지했다. 전역 제거는 `hwp3-sample16-hwp5-2018/2022/2024` 페이지 수를 65페이지로 밀어 회귀가 컸다.
2. 대신 본문 흐름 배치와 높이 측정에서 사용하는 `spacing_before`만 `is_hwp3_variant`일 때 원래 값으로 복원했다.
2. vpos 보정에서 현재 문단의 `spacing_before`를 사전 차감하지 않도록 했다.
3. Stage 13의 p83 BCP 2줄 합성은 제거했다.
4. `tests/issue_1116.rs`를 3mm 격자 PDF 기준 좌표로 갱신했다.

## 4. 좌표 비교

PDF 정답지 좌표는 `pdftotext -bbox-layout`의 `yMin`을 SVG 좌표계로 환산했다.

| 항목 | 한컴 PDF 환산 | RHWP 수정 후 |
| --- | ---: | ---: |
| `2. 추진방향` | 약 340.8px | 337.2px |
| `3. 주요 추진내용` | 약 752.9px | 749.3px |
| p74 첫 줄 | 약 382.4px | 375.5px |
| p75 첫 줄 | 약 470.8px | 463.7px |
| p86 첫 줄 | 약 982.9px | 975.9px |

폰트 bbox/브라우저 렌더 차이를 감안하면 세로 흐름은 3mm 격자 기준으로 크게 개선됐다.

## 5. 산출물

재생성:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm/hwp3-sample16-hwp5_003.svg
output/poc/render-spacing/visual-compare-stage14/hancom2022-p3-03.png
output/poc/render-spacing/visual-compare-stage14/rhwp-hwp5-p3-grid-3mm.png
output/poc/render-spacing/visual-compare-stage14/hancom2022-vs-rhwp-p3-grid-3mm-side-by-side.png
```

## 6. 검증

완료:

1. `cargo fmt --all -- --check`
2. `cargo test --test issue_1116 -- --nocapture`
3. `cargo test --test issue_1035_alignment -- --nocapture`
4. `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm -p 2 --show-grid=3mm`
5. `cargo test --test issue_1086 -- --nocapture`
6. `cargo build --bin rhwp`
7. `git diff --check`
