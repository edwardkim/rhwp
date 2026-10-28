# Stage 136 - 2025 편람 HWPX Q&A p286 시각 증적 분석

## 목표

Stage 135에서 Q5 saved-frame tail 및 Q7/Q8 경계를 PDF와 같은 p283~p285 흐름으로 고정했다. 이번 Stage는 연속 Q&A 구간의 다음 physical page인 p286을 PDF, native HWP, HWPX로 대조해 첫 남은 시각적 divergence를 찾는다.

## 기준 자료

- PDF oracle: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽, 재생성 금지)
- native 입력: `samples/2025 행정업무운영 편람(최종).hwp`
- HWPX 입력: `samples/2025 행정업무운영 편람(최종).hwpx`
- 선행 커밋: `fd8bf8c5f` (`fix: HWPX Q&A 저장 frame 줄 소유를 보존한다`)

## 분석 범위

1. PDF p286과 native HWP/HWPX p286 SVG를 같은 96 DPI raster로 대조한다.
2. Q8 continuation 또는 다음 Q&A 표의 page owner, table border, body baseline과 footer 여백을 비교한다.
3. HWPX source와 HWP 저장-재로드의 p286 render tree가 같은지 확인한다.
4. 차이가 있으면 raw lineSeg, row-cut, renderer paint 중 원인 계층을 고정한 뒤에만 코드를 수정한다.

## 보존 계약

- Stage 134~135의 HWPX source/저장-재로드 383쪽 및 p283~p285 owner를 후퇴시키지 않는다.
- native HWP의 383쪽과 PDF oracle을 변경하지 않는다.
- fixture 경로, physical page 번호, paragraph index만으로 구현을 분기하지 않는다.

## 완료 기준

1. p286의 PDF/native/HWPX 비교 근거와 first visual divergence 판정을 남긴다.
2. 구현이 필요하면 source topology로 한정한 회귀를 추가한다.
3. source와 저장-재로드의 383쪽 계약을 유지한 검증 결과를 기록한다.

## 분석 결과

### p286, p287 visual sweep

PDF p286과 p287을 기존 96 DPI oracle에서 rasterize하고, 같은 physical page의 native HWP 및 HWPX SVG를 rasterize해 비교했다. p286의 Q9 표제·3개 bullet response와 p287의 Q10/Q11 표제·border·footer 여백에서 native HWP와 HWPX 사이의 owner 또는 baseline drift를 발견하지 못했다.

- p286: Q9 `보조기관, 보좌기관, 합의제행정기관의 의미`가 세 출력 모두 같은 첫 표 fragment에 있다.
- p287: Q10 `공문서 작성시 연·월·일의 정확한 표기방법은 무엇입니까?`와 Q11 표제가 세 출력에서 같은 physical page에 있다.
- p284~p285의 저장 frame 보정이 p286 이후 Q&A 표의 source 흐름을 다시 밀지 않았음을 확인했다.

이 범위에서는 renderer 변경이 필요하지 않았다. 시각 검증 결과를 source 및 HWP 저장-재로드 page-tree 회귀로 고정한다.

## 회귀 및 검증 결과

`tests/issue_3930_hwpx_hwp_save_layout.rs`에 p286 Q9와 p287 Q10 title owner를 추가하고, 저장 HWP 재로드의 p286/p287 render tree가 HWPX source tree와 byte-for-byte 같은지 확인한다. 기존 p283~p285 Q&A 경계, p30/p144/p145 바탕쪽·표 owner와 383쪽 contract도 함께 유지한다.

```text
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

Stage 완료 시 위 명령의 최종 summary를 기록한다. PDF oracle은 `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`를 그대로 사용하며 재생성하지 않는다.

실행 결과:

```text
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s
```
