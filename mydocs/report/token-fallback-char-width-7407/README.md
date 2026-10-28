# #7407 A2b — 긴 토큰의 글자 단위 폴백이 잰 폭을 쓰게 한 변경의 시각 증적

제품 SHA: `3d18a31ec`. 도입 전 비교 기준은 그 부모 `4cc0ffeed`(= PR #7410 head)다.
두 checkout 을 각각 `cargo build --release --bin rhwp` 로 빌드해 같은 입력에 적용했다.

Windows 11 / Chromium(webfont rasterizer) / Poppler 25.07.0 / 96dpi.
`scripts/visual_sweep.py` 의 compare·overlay·review 를 산출해 직접 판독했다.
중간 SVG·로그·점수 JSON 은 커밋하지 않고 대표 PNG 만 남긴다.

## 1. 대상 — `samples/issue6639/issue6639-reference-input-160.hwpx` 1쪽

기준 PDF `pdf/issue6639/issue6639-reference-160-2020.pdf`. 각 PNG 는
**왼쪽 rhwp / 가운데 한/글 / 오른쪽 overlay** 다.

| 파일 | 상태 | 2px 이웃 관용 내용 실루엣 |
| --- | --- | ---: |
| [issue6639_p1_before.png](issue6639_p1_before.png) | 도입 전 `4cc0ffeed` | 61.40881% |
| [issue6639_p1_after.png](issue6639_p1_after.png) | 수정 후 `3d18a31ec` | 99.84% |

도입 전에는 대상 칸의 긴 문단마다 줄이 하나씩 더 생겨 아래로 밀리고 문단 10 의 줄이
칸 밖으로 잘린다. 수정 후에는 문단별 줄 수가 `[1, 4, 6, 5, 3, 1, 1, 1, 1, 2]` 로
한/글과 같아진다. 점수는 참고값이며 판정은 위 판독이다.

## 2. 코퍼스에서 함께 바뀐 쪽 — 판독 결과

`samples/**` 1,148건을 두 빌드로 `export-svg` 해 산출 해시를 비교하면 5건이 바뀌고
쪽 수가 바뀐 문서는 0건이다. issue6639 두 본을 뺀 나머지 3건이 아래다.

### [task2146_p3_row_pitch.png](task2146_p3_row_pitch.png) — 행 간격이 정본과 일치하게 됐다

`samples/task2146/21761835_jeonjik_exemption_table.hwp` 3쪽, 기준
`pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`.
위부터 **도입 전 rhwp / 수정 후 rhwp / 한/글**이다.

96dpi raster 에서 x 186..350 구간의 잉크 행 위치를 재면:

```text
정본     첫 줄 582 → 다음 행 첫 줄 729   = 147px
도입 전  첫 줄 575 → 다음 행 첫 줄 733   = 158px
수정 후  첫 줄 575 → 다음 행 첫 줄 722   = 147px
```

행 간격 자체는 정본과 같아졌다. 실루엣 점수는 61.61451% → 54.99556% 로 내려가는데,
이 쪽은 그 위에서 이미 7px 이 위로 어긋나 있고(도입 전후 동일) 과대 행 높이가 그 어긋남을
상쇄하고 있었다. **점수 하락을 이 변경의 회귀로 읽지 않는다.** 남은 7px 은 이 변경의
범위 밖이며 미해결이다.

### [counterexample_78494_p54.png](counterexample_78494_p54.png) — 한 칸이 정본보다 한 글자를 더 받는다

`samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` 54쪽, 기준
`pdf/78494-virtual-convergence-industry-decree-2020.pdf`.
위부터 **도입 전 rhwp / 수정 후 rhwp / 한/글**이다.

정본은 `전자문서·` / `전자거래` 로 나누는데 수정 후 rhwp 는 `전자문서·전` / `자거래` 로
낱말 가운데를 가른다. 실루엣은 84.75065% → 84.52491% (−0.23%p) 다.

근인은 이 변경이 아니라 **칸 안 여백 축소**이며 [#7413](https://github.com/edwardkim/rhwp/issues/7413) 로 분리했다.
같은 쪽의 다른 칸 8개는 안 여백 13.60px 을 지키는데 이 칸만 2.00px 로 깎여 가용 너비가
84.90px 가 된다. 선언 안 여백을 지킨 73.30px(5497 HWPUNIT)이면 6번째 글자(누적 6261
HWPUNIT)가 거부돼 정본과 같아진다. 도입 전에 이 칸이 정본과 같았던 것은 폴백이
`·`(U+00B7)를 실측 3.49px 대신 8.0px 로 과다 계상했기 때문이며, 두 오류가 상쇄 중이었다.
`src/renderer/composer.rs` 는 PR #7410 과 이 변경 모두 건드리지 않는다.

### `samples/issue5169_viewtext_changetracking.hwp` 12쪽 — 미검증

한 칸에서 글자 하나가 윗줄로 옮겨간다(`연구위원`/`교수` → `연구위원교`/`수`). 줄 수는
그대로다. 이 문서에는 대응하는 한/글 출력 PDF 가 없어 **판정하지 않는다.**

## 이 증적이 말하지 않는 것

- B(표 높이·가로선 위치, `#7407`)는 여기서 다루지 않는다.
- WASM 병행 캡처는 실행하지 않았다. 위 수치는 Native 빌드 기준이다.
- 쪽 수가 바뀐 문서는 없으나(원장 594문서 회귀 0) 이는 쪽 수 계약이지 시각 일치가 아니다.
