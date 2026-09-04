# Task M100 #1453 단계별 완료보고서 — 6단계: 누적 회귀 테스트 + 시각판정 + 전체 검증

- 이슈: #1453 (Part B 막대 누적 보정)
- 브랜치: `local/task1453`
- 단계: 6/6
- 작성일: 2026-06-21

## 구현 내용

### (a) 렌더러 기하 유닛 테스트 (`renderer.rs` tests)

합성 차트(시리즈 name 비움 → 범례 swatch 제외)로 데이터 막대 x 좌표를 추출해 단언:

| 테스트 | 단언 |
|--------|------|
| `test_stacked_bars_share_x_per_category` | stacked → 카테고리(2)당 단일 x = **2** (시리즈 같은 x 공유) |
| `test_clustered_bars_distinct_x` | clustered → 2×3 = **6** 분리 x (무회귀 가드) |
| `test_percent_stacked_axis_and_single_column` | `100%`/`0%` 축 라벨 + 단일 x = 2 |

`data_bar_xs` 헬퍼: `fill="#..."` + stroke 없음 + `10×10`(범례 swatch) 제외 → 데이터 막대만.

### (b) 실샘플 통합 가드 (`tests/issue_1453_chart_3d_ofpie_routing.rs`)

`chart_stacked_bars_render_with_percent_axis` — 누적 막대 6종(hwpx):
- 3차원누적세로/가로 + 누적세로/가로 + 백프로기준누적세로/가로.
- 전부 정상 렌더(fallback 0, chart 클래스).
- percentStacked 2종(백프로기준누적)만 `100%` 축 라벨 보유, 일반 stacked 4종은 미보유.

## 정답지 대조 (시각판정)

`output/poc/chart_c1a/{종류}/rhwp.png` ↔ `hancom.png`:

| 종류 | 5단계 전 (grouped 왜곡) | 6단계 (누적 보정) | 정답지 기하 일치 |
|------|----------|----------|:---:|
| 3차원누적세로막대형 | 묶은 막대(Y 0~5) | **누적 막대(Y 0~14)** | ✅ |
| 3차원누적가로막대형 | 묶은 막대 | **왼→오 누적** | ✅ |
| 누적세로막대형 (2D) | 묶은 막대 | **누적 막대** | ✅ |
| 누적가로막대형 (2D) | 묶은 막대 | **누적 막대** | ✅ |
| 백프로기준누적세로막대형 (2D) | 묶은 막대(Y 0~5) | **100% 꽉 찬 막대(0%~100%)** | ✅ |
| 백프로기준누적가로막대형 (2D) | 묶은 막대 | **100% 꽉 찬 막대** | ✅ |

남은 차이 = 알려진 스타일 4갭(제목·팔레트·범례·축, C1c)뿐. **누적 기하는 정답지에 수렴.**
(백프로 항목별 시리즈 비율이 한컴 PDF와 일치 확인.)

## 전체 검증

```
$ cargo test            → 2477 passed; 0 failed (RC=0)  [이전 2469 + 신규 8]
$ cargo clippy --all-targets → ooxml_chart/issue_1453 무경고
```

## 완료 기준 충족

- [x] 누적 기하 가드(렌더러 3 + 통합 1) 통과
- [x] 6종 시각판정 자료 산출 + 정답지 기하 일치
- [x] 전체 스위트 통과 / clippy 무경고

## 다음 단계 (Step 4 전 게이트)

작업지시자 **wasm 개발 서버 육안 확인** 후 최종 보고서 작성 → `local/devel` 통합.
