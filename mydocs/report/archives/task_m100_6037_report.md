---
kind: report
status: done
canonical: mydocs/report/archives/task_m100_6037_report.md
last_verified: 2026-08-25
---

# #6037 보고서 — 차트 계열 가드의 술어를 파손 기준으로 좁힌다

- **Issue**: [#6037](https://github.com/edwardkim/rhwp/issues/6037) (#3683 Track B 곁가지)
- **브랜치**: `task6037` · **기준 커밋**: `upstream/devel = 385e93b2c`
- **계획서**: [`task_m100_6037.md`](../../plans/archives/task_m100_6037.md)

## 결론 한 줄

**파손의 술어는 계열 수가 아니라 「그리기 장치와 짝이 맞는가」였다.** 한컴 2022 로 21 판정 단위를
144DPI 래스터로 갈라 확정했고, 가드를 그 술어로 좁혔다 — 원형 가드는 제거하고(파손이 아니다),
주식형은 `c:upDownBars` 의 **첫·끝 계열**이 바뀔 때만 막는다. 엔진 산출을 한컴이 다시 판정해
**원형 5종은 그림이 픽셀 0.00% 로 불변**이고 **주식형 통과분은 전건 정상**임을 확인했다.

## 1. 왜 손댔나

B2-엔진(#5652)이 넣은 가드 둘은 `#5447` 스파이크 실측 위에 서 있었는데, 그 스파이크가 잰 것은
**rhwp 가 꼬리 복제로 만든 산출**이었고 **한컴 편집기 자신의 산출**은 재지 않았다. 작업지시자가
"한컴에서는 원형·주식형도 계열 증감이 된다"고 관측해 그 빈칸을 실측으로 메웠다.

결과는 양쪽 다 뒤집었다 — 한컴은 **하나도 거부하지 않고**, 그러면서 **자기 산출을 여러 건 깨뜨린다**.
즉 "한컴이 하니까 우리도 한다"도 "한컴이 깨지니까 다 막는다"도 답이 아니었다.

## 2. 실측 — 한컴 편집기 산출 13 판정 단위

원장 `samples/issue6037/MANIFEST.json`. 대조군은 코퍼스 정답지 `pdf/chart/**-2022.pdf`.

### 갈림 ① 값이 빈 계열이냐

| 문서 | 빈 계열일 때 | 값 채웠을 때 |
|---|---|---|
| 2차원원형 계열추가 | 조각 불변, 제목만 밀림 | 원본 `판매` 가 화면에서 밀려남 |
| 원형대원형(ofPie) 계열추가 | **파이가 통째로 사라짐** | **정상** |
| OHLC 계열추가 | **검은 박스** | **정상** |

빈 계열이 파손을 만든다. 그런데 **rhwp 는 빈 계열을 만들 수 없다** —
`is_number`(`chart.rs:283`)가 빈 문자열을 `notANumber` 로 거부한다. 이 파손은 rhwp 에서 재현
불가능하다.

### 갈림 ② 어느 계열을 건드리나 — 캔들 짝

`c:upDownBars` 는 **첫 계열과 끝 계열**을 몸통으로 삼는다.

| 편집 | 첫/끝 | 렌더 |
|---|---|---|
| 중간 삽입 (`…저가,새계열,종가`) | 유지 | 정상 |
| 중간 삭제 (`저가`) | 유지 | 정상 |
| **꼬리 삽입** | 끝 바뀜 | **검은 박스** |
| **꼬리 삭제**(`종가`) | 끝 바뀜 | **검은 박스** |
| **첫 계열 삭제**(`시가`) | 첫 바뀜 | **검은 박스** |
| HLC 꼬리 삽입 | 끝 바뀜, **장치 없음** | **정상** |

HLC 꼬리 삽입이 음성 대조군이다 — 끝이 바뀌었는데도 정상이다. 따라서 조건은 "끝이 바뀜"이 아니라
**`upDownBars` 가 있을 때의 양끝 변경**이다. `upDownBars` 잔존 자체는 원인이 아니다(정상 3건도
잔존했다).

## 3. 무엇을 바꿨나

### S1 — 편집 스캐너가 캔들 장치를 본다

`ChartData.has_up_down_bars`(`crates/rhwp-ooxml-chart/src/data.rs`). plot 의 자식이라 계열 밖에서
나오므로 **차트 단위**로 둔다. 렌더 쪽 모델의 동명 필드(`lib.rs:75`, C2a #2277)와 같은 토큰이지만
이쪽은 편집 검증용이다. 코퍼스 28건 중 OHLC 1건만 `true`.

### S2 — 가드를 술어로 좁혔다

- `pieSeriesCountFixed` **제거**. 파손이 아니고, rhwp 는 꼬리에 붙이므로 원본이 `idx=0` 에 남아
  계속 그려진다(한컴은 맨 앞에 끼워 원본을 밀어낸다 — 이 지점은 rhwp 가 더 안전하다). 원형에
  계열이 여럿인 것은 OOXML 규격상 유효하다.
- `stockSeriesCountFixed` → **`candleAnchorBroken`**. `upDownBars` 가 있을 때 **첫 또는 끝** 계열이
  바뀌면 거부한다. 개수 변경 자체는 막지 않는다.

**「그대로라고 볼 근거」는 이름·값 둘 중 하나**로 판정한다. 한쪽만 보면 각각 오탐이 난다 —
이름만 보면 "개명 + 개수 변경"이, 값만 보면 "그 계열 값 편집 + 개수 변경"이 헛걸린다.

> **작업 중 잡은 것** — 최초 술어는 **끝만** 봤다. 기존 회귀 시험이 `series.remove(0)`(시가 삭제)로
> 거부를 단언하고 있어 드러났다. 첫 계열 변경은 그때 실측이 없어 장치 동작에서 추론해 fail-closed
> 로 함께 막았고, **S5 에서 한컴 실측이 그 추론을 확정했다**(첫계열삭제 = 검은 박스).

### S3 — 읽기 봉투에 `plot`·`hasUpDownBars`

`chart_data_json` 에 두 필드를 additive 로 더했다. 무효과(원형에 계열을 더해도 안 보인다)를
**코어가 막는 대신 표면이 알린다**. B2-UI 가 계획했던 dryRun 선탐침이 이 필드로 불필요해진다.

### S4 — 표면·문서

MCP `hwp_csv_to_chart` 설명, `cli_commands.md`, `agent_knowledge_map.md` 를 새 계약으로 갱신.

> 초안에 `hwp_get_chart_data` 를 참조했는데 **그런 도구가 없다.** 읽기 봉투는 코어·WASM 경로에만
> 있어 문구에서 뺐다.

## 4. 한컴 재판정 — 엔진 산출

원장 `samples/issue6037/MANIFEST-engine.json`. 생성기
`generate_issue6037_judgment_bundle`(`#[ignore]`, 산출마다 재개방·①==②·③④ 바이트 불변 자가검증).
**통과시키기로 한 편집만** 담는다 — 거부되는 것은 산출이 없어 판정 대상이 아니다.

| 변종 | 대조군 대비 | 판정 |
|---|---|---|
| 2차원원형·**3차원원형**·원형대원형·**원형대가로막대형**·**쪼개진원형** 계열추가 | **0.00%** | 미반영(의도한 결과) |
| 고가저가종가(HLC) 꼬리계열추가 | 0.49% | 반영 |
| 시가고가저가종가 중간계열추가 | 0.59% | 반영 |
| 시가고가저가종가 중간계열삭제 | 0.13% | 반영 |

- **원형 5종 전건 픽셀 0.00%** — 가드를 풀어도 그림이 전혀 바뀌지 않는다. 굵은 3종은 **첫 실측**이다.
- **중간계열삭제는 한컴 편집기가 같은 편집을 한 산출과 래스터 0.02%** — 엔진이 한컴과 같은 그림을
  만든다. 이보다 강한 증거는 없다.
- 대조군 PDF 는 코퍼스 정답지와 **0.00% 동일**함을 확인해 중복 보관하지 않는다.
- 이번엔 두 포맷 모두 PDF 를 받아 `raster_equal` 불변식 8건으로 **HWP↔HWPX 렌더 동일성**을
  고정했다(한컴 세트 원장에 남은 `provenance_gap` 이 엔진 세트에서는 없다).

## 5. 수용 기준

| # | 기준 | 판정 | 근거 |
|---|---|---|---|
| 1 | 주식형 양끝이 바뀌는 편집은 거부, `invalid[]` + exit 2 | 충족 | `stock_candle_anchor_is_guarded_at_both_ends`(꼬리삭제·첫계열삭제·꼬리삽입 × 2포맷) · CLI `stock_tail_series_add_via_csv_exits_two_with_candle_guard` |
| 2 | 양끝이 유지되는 계열 증감과 HLC 는 통과 | 충족 | `stock_middle_series_edits_are_allowed`(끝이 `종가` 유지 단언) · `hlc_without_candle_allows_tail_series_add` |
| 3 | 원형 계열 추가 통과 + 원본이 `idx=0` 에 남는다 | 충족 | `pie_series_add_is_allowed_and_keeps_the_original_first`(3종 × 2포맷) · CLI `pie_series_add_via_csv_is_allowed`(되읽은 CSV 첫 열 대조) |
| 4 | 봉투에 `plot` 이 실리고 3면 자기서술이 일치 | 충족 | 드리프트 가드 `capabilities_and_mcp_declare_the_command_consistently` 등 green |
| 5 | 편집하지 않은 차트 blob 원형 유지 · B1/B2 무회귀 | 충족 | `issue_3546_chart_preserved_on_save` · `issue_4100_chart_data_edit` 전건 · #5652 판정 트립와이어 green |
| 6 | 한컴이 새 산출을 정상 개봉·렌더 | 충족 | §4 — 16 산출 전건 개봉, 원형 0.00%, 주식형 통과분 정상 |

## 6. 검증

변경 범위 = Rust core + 크레이트 스캐너 + CLI/MCP 자기서술 + 문서. **렌더러·studio 무변경**이라
Native Skia·wasm 은 대상 밖이다(`local_validation.md` §4.3).

```bash
rustfmt --edition 2021 --check <변경 .rs>      # CRLF 체크아웃이라 LF 로 바꿔 검사
node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel
cargo clippy --locked --all-targets --profile release-test --target-dir target/pr-review -- -D warnings
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast
python tools/hancom_chart_judgment_verify.py --manifest <절대경로>/samples/issue6037/MANIFEST.json
python tools/hancom_chart_judgment_verify.py --manifest <절대경로>/samples/issue6037/MANIFEST-engine.json
```

- `rust-unit-test-tiers`: **4221 tests 불변** — 크레이트 `#[cfg(test)]` 을 늘리지 않았다(신규 시험은
  전부 통합 시험 쪽).
- `rust-test-suite-manifest --check`: **`새 test source 없음`** 확인. 뒤따르는
  `Cargo.toml generated test target block drift` 는 문서가 예고한 **기여자 checkout 의 위양성**이다
  (생성 블록은 review worktree·CI 전용, `git status` 에 `Cargo.toml` 변경 없음).
- 판정 원장 재계산: 한컴 세트 **117건**, 엔진 세트 **156건** 전건 일치.

## 7. 알려진 한계

- **빈 계열은 계속 만들 수 없다.** `is_number` 가 빈 값을 거부한다. 한컴이 파손을 낸 케이스가
  정확히 그것이므로 계속 막는 것이 맞다.
- **`c:upDownBars` 를 편집에 맞춰 다시 쓰지 않는다.** 그래서 OHLC 양끝을 바꾸는 편집은 여전히
  거부다 — 그것을 지원하려면 캔들 장치 재작성이 필요하고 B3 소관이다.
- **`plot` 은 첫 계열 기준**이다(`axis` 와 같은 방식). 코퍼스 전건이 단일 plot 이라 실제 문제는
  관측되지 않았다.
- **한컴 세트 원장의 `provenance_gap`** — 그 13 변종은 변종당 PDF 가 1개라 어느 포맷에서 저장했는지
  기록이 없다. 포맷 간 렌더 동일성은 엔진 세트 원장으로만 판정한다.

## 8. 후속

- **B2-UI** — 이 이슈와 직렬이 아니다. S3 의 `plot` 노출로 UI 의 dryRun 선탐침 설계가 단순해진다.
  계획은 별도 서브이슈로 등록한다.
- **차트 시각 회귀 게이트** — #3938 종료 코멘트가 지목한 렌더 축의 공백. 여전히 이슈 미등록이다.
