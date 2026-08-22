# task_m100_5863 처리결과 — 다음 쪽 표 조각의 현재 쪽 중복 방출을 막는다

- 이슈: [#5863](https://github.com/edwardkim/rhwp/issues/5863)
- 분리한 형제 이슈: [#5862](https://github.com/edwardkim/rhwp/issues/5862) (같은 쪽의 **눈에 보이는** 절단·소실 — 이 PR 범위 밖)
- 기준: `0f9ceeb19` (devel)
- 재현 문서·정답지 모두 저장소 안: `samples/hwpx_sample2.hwp` · `pdf/hwpx_sample2-2024.pdf`

## 1. 결함

`samples/hwpx_sample2.hwp` 8쪽 SVG 에 9쪽 표(`구분/조회방법` 순위확인서 발급 안내)의 글자가
**한 벌 더** 실렸다. 같은 표가 9쪽에 온전히 그려지고 한글 2024 정본도 9쪽에 두므로 순수한
중복이다. 조각은 본문 하한(1,084.7px)을 넘어 1,129.9px 까지 뻗어 용지(1,122.5px) 밖까지 갔다.

```
p8  본인확인 1회 · 내역확인 1회 · 한국부동산원 1회   ← 중복 (128자)
p9  본인확인 1회 · 내역확인 1회 · 한국부동산원 1회   ← 정상
detect_table_clipping:  CLIP 1/29p  max_overflow=45.1px
```

## 2. 원인 — 억제 창이 테두리 안티에일리어싱 폭(6px)뿐이었다

`suppress_future_nested_table_border_residue` 는 잘린 셀의 clip 바닥 **바로 아래 6px**
안에서 시작하는 표만 숨겼다(`NESTED_FRAGMENT_RESIDUAL_BORDER_PX`). 이 조각은 34.4px 아래에서
시작해 창을 벗어난다.

```
Table pi=74   y= 226.6 bot= 986.3
  Cell        y= 226.6 bot= 985.7           ← 잘린 셀(clip)
    Table pi=21 y=1020.1 h=109.8 bot=1129.9 ← clip 바닥 아래에서 시작
```

clip 바닥 **아래에서 시작하는** 표는 그 셀 안에서 보일 수 있는 부분이 없다 — 정의상 다음 쪽
조각이다. 그래서 창의 상한을 없앴다. 함수의 기존 주석이 이미 근거를 적어 두고 있다:
"A separate page render owns that table with a fresh viewport."

## 3. 시각 증적 — 정직하게

이 잔여물은 본문 clip 이 잘라내므로 **래스터에는 나타나지 않는다.** 전/후 렌더는 픽셀 동일하다
([`render_before_after.png`](edit_demo_5863/render_before_after.png)). 이 PR 이 고치는 것은
**SVG DOM·텍스트 추출 오염과 클리핑 게이트 오탐**이다
([`svgdom_before_after.png`](edit_demo_5863/svgdom_before_after.png)).

같은 쪽에서 **눈에 보이는** 절단·소실(마지막 줄 가로 절단 + 다음 줄 통째 소실)은 셀 clip 이
자기 내용보다 43px 짧은 별개 결함이고, 원인이 높이 측정 코어라 이 PR 에 넣지 않았다 —
[#5862](https://github.com/edwardkim/rhwp/issues/5862) 로 분리하고 정답지 대조 이미지를 붙였다
([`cellclip_p8_rhwp_vs_oracle.png`](edit_demo_5863/cellclip_p8_rhwp_vs_oracle.png)).

## 4. 검증

### red → green

```
수정 제거:  next_page_table_fragment_is_not_duplicated_on_the_current_page ... FAILED
            8쪽에 9쪽 표의 글자 "본인확인" 가 중복으로 실렸다 (#5863)
            nothing_but_the_footer_is_drawn_below_the_body_on_page_eight  ... FAILED
복원:       3/3 통과
```

`the_owning_page_still_draws_the_whole_table` 는 전후 모두 통과한다 — 억제가 진짜 내용을
지우지 않는지 지키는 가드다.

### 회귀 없음 (전/후 바이너리 직접 대조)

수정 전 바이너리를 따로 빌드해 같은 문서들을 돌렸다.

| 검사 | 결과 |
|---|---|
| `export-text` 총 글자수 65개 문서 | **전부 동일** (감소 0 · 증가 0) |
| `export-svg` 전 쪽 글자수 16개 문서 | `hwpx_sample2.hwp` 8쪽만 −128자, **나머지 15개 문서 동일** |
| 8쪽 래스터 픽셀 diff | **bbox=None (완전 동일)** |
| 259문서 쪽수 게이트 | 245/259 — 전/후 동일, 회귀 0 |
| `detect_table_clipping` (해당 문서) | CLIP 1/29p → **클리핑 0** |

`render_page_samples.tsv` 의 `press_release_topbottom_float.hwpx` 행은 baseline 열이 3인데
실측이 2다. 전/후 바이너리 모두 2라 **이 PR 과 무관한 기존 개선**이며, 픽스처 열이 낡았다.

### 실행한 것

| 명령 | 결과 |
|---|---|
| `cargo test --test regression_suite_014 issue_5863` | 3 passed |
| `cargo test --lib -p rhwp` | 3,893 passed / 13 ignored |
| `rustfmt --edition 2021` 변경 파일 | 정리 완료 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4,225 (src 기준선 유지) |
