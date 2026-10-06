---
kind: report
status: active
last_verified: 2026-09-21
---

# [#7092] 실측 검증 face 두 개를 가운뎃점 신뢰 목록에 올린다

- 대상 SHA: `upstream/devel` `4a29140ff` 위의 한 줄 목록 추가
- 기준: 저장소 `pdf/**` 의 한컴 PDF 전수 + `ttfs/hwp/` 글꼴 파일 실측
- 계측: 정본 PDF 에서 `·` 다음 글자와의 `origin.x` 차 ÷ 글자크기(같은 span·같은 크기만,
  리더 반복 제외). rhwp 는 `export-svg --profile print` 의 글자 x 차.

## 1. 남아 있던 것

`10f2c727f`(#7092)가 메트릭 신뢰의 근거를 **실측 검증 face 목록**으로 옮겼고, 그 목록은
`HY신명조` 하나였다. 그래서 `·` 를 진짜로 갖고 있는 다른 글꼴도 `.notdef` 로 오인돼
`0.3em` 으로 눌렸다. 이슈가 지목한 둘째 문서가 그 예다.

```text
  156347175_속기자료.hwp   제목 줄 face = HY헤드라인M
     수정 전 rhwp 0.299em      정본 0.997em
```

## 2. 글꼴 파일이 말하는 것 — `.notdef` 가 아니다

```text
  ttfs/hwp/H2HDRM.TTF   upm=1024   U+00B7 periodcentered  advance=1024 = 1.000em
                                   윤곽선 bbox (439, 297, 586, 435)
  ttfs/hwp/HYWULM.TTF   upm=1024   U+00B7 periodcentered  advance=1024 = 1.000em
```

빈 상자가 아니라 **점이 그려진 진짜 글리프**다.

## 3. 한/글 정본이 말하는 것

저장소 `pdf/**` 전수(앞 30쪽)에서 이 face 들의 `·` 전진폭이다.

| face | 문서 수 | n | 중앙값 | 범위 | 0.2~0.4em 구간 |
| --- | ---: | ---: | ---: | --- | ---: |
| `HY헤드라인M` (H2hdrM) | **25** | ≈50 | **1.000** | 0.89 ~ 1.06 | **0건** |
| `HY울릉도M` (HYwulM) | 2 | 9 | 0.968 | 0.944 ~ 1.000 | 0건 |

1 미만 값은 자간·justify 가 섞인 것이고 좁힘 갈래(0.3em)와는 자릿수가 다르다.
문서 예: `k-water-rfp` 5종 · `mel-001` 3종 · `aift-2022` · `pr_6528_issue6181_p5` ·
`2022년 국립국어원 업무계획` 2종 · `156689818_kftc_press-hwpx-2020` …

## 4. 수정 전후

```text
  samples/mel-001.hwp 8쪽 제목      HY헤드라인M `·`
     수정 전 0.299em → 수정 후 1.000em      정본 1.000em
  156347175_속기자료.hwp
     HY헤드라인M  0.299 → 1.000 (정본 0.997)
     휴먼명조     0.299 → 0.299 (이 변경의 범위 밖, §6)
```

## 5. 시각 증적 — fresh WASM ↔ 정본, 같은 쪽·같은 줄

`scripts/visual_sweep.py --wasm-pkg`(wasm-pack `--target web` 새 빌드, Chrome
151.0.7922.34, 96dpi) 로 `samples/mel-001.hwp` 8·10쪽을 수정 전후 캡처했다.
`middle-dot-title-vs-oracle-mel-p8.png` 가 8쪽 제목 줄을 셋으로 쌓은 것이다.

```text
  BEFORE   장시간·야간노동 …        점 좌우가 붙고 줄이 짧다
  AFTER    장시간 · 야간노동 …      정본과 같은 간격, 줄 끝 `95)` 도 같은 x
  정본      상시간 · 야간노농 …      (raster 글꼴 차이일 뿐 자리는 같다)
```

## 6. 넣지 않은 것과 그 이유

| face | 정본 | 글꼴 파일 | 판정 |
| --- | --- | --- | --- |
| `HY중고딕` | 5문서 n=41 전각 | 저장소에 없음 | 파일 근거 없음 — 보류 |
| `HY견고딕`·`HY그래픽` | 사실상 1문서 | 있음 (1.000em) | 표본 부족 — 보류 |
| `휴먼명조`·`휴먼고딕` | **22문서 n=72 전각** | `HMKMM.TTF` 512/512 | 아래 참조 |

`휴먼명조` 는 근거가 가장 두꺼운데도 목록에 넣어도 **값이 움직이지 않는다**. 실험으로
확인했다 — 목록에 추가하고 `latin1_table_is_uninformative` 항까지 끈 빌드로
`samples/k-water-rfp.hwp`·`biz_plan.hwp`·`tac-img-02.hwp` 를 다시 렌더해도 `·` 가
`0.299em` 그대로다. 막는 것은 `alt_type == 1 && !substituted` 쪽이다(HFT/TrueType 두
realization 이 갈리는 축). 그 갈래는 이 변경의 범위 밖이므로 **미검증으로 남긴다**.

## 7. 검증

- `cargo nextest run --release --no-fail-fast` — **10120 / 10120 통과**
- 래칫 `body_overflow`·`text_overlap`·`off_canvas`·`overflow_cell`·`oracle_page_count`
  127건 통과 — 갱신한 기준값 **없음**
- Lint 묶음 전부 통과: fmt · clippy native/wasm32/all-targets · workspace build ·
  `rust-test-suite-manifest --check --base-ref 4a29140ff` ·
  `rust-unit-test-tiers --check --base-ref 4a29140ff`(4205, 증가 없음)
- red → green: `src/renderer/style_resolver.rs` 만 되돌리면 새 핀이 실패하고
  반례 핀(`휴먼명조` 유지)은 그때도 통과한다.
