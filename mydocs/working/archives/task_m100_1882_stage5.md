# task_m100_1882 Stage 5 완료 보고서 — 통합 테스트 + 전체 회귀 + 시각검증

- 이슈: #1882 (C1c, #1431 Track C)
- 브랜치: `local/task1882`
- 단계: Stage 5 / 5 — 통합 (`tests/issue_1882_chart_style_gaps.rs` 신규)

## 변경 내용

1. **신규 통합 테스트** `tests/issue_1882_chart_style_gaps.rs` (issue_1431_scatter.rs 관례 미러 —
   샘플 로드 → `render_page_svg(0)` → substring/좌표 assert, hwp+hwpx 각각):
   - `chart_auto_title_rendered` — 막대/라인/원형에서 "차트 제목" 포함 + bold(600) 미포함.
   - `chart_hancom_palette_applied` — 실측 3색 포함 + 구 녹색(#70ad47) 미포함.
   - `chart_axis_headroom_and_sparse_ticks` — 막대 0,2,4,6 존재·3,5 부재 / scatter Y>5<·X 0.5간격.
   - `chart_legend_on_right` — `hwp-chart-legend` 그룹의 텍스트 x가 차트 그룹 내 모든 데이터
     막대의 우측 끝보다 오른쪽 (페이지 배경 rect 오검출 방지 위해 차트 그룹 내부로 한정).
2. **Stage 4 보완(커밋 6d0829d7)**: 좁은 차트(w×0.30 < 50px)에서 `clamp(50, w×0.30)`이
   min>max 패닉하던 결함 — 우측 범례를 폭 충분 시에만 활성화(좁으면 하단 폴백, NaN 폴백)
   + 회귀 테스트. *(커밋 후 자동 보안 리뷰 지적 → 즉시 수정.)*

## 검증

```
cargo test --test issue_1882_chart_style_gaps         → 4 passed, 0 failed
cargo test --lib ooxml_chart                          → 45 passed (기존 32 + C1c 13)
cargo test (전체)                                      → 전부 통과 (exit 0, 실패 0)
cargo clippy --all-targets -- -D warnings              → 경고 0
```

**시각검증** (`output/poc/chart_c1c/all/` — 27종 hwpx 전수 SVG+PNG 산출):

| 확인 종류 | 결과 |
|---|---|
| 묶은세로막대형 | 4갭 전부 정합 — 제목/팔레트/우측 범례/축 0~6 라벨 0,2,4,6 (정답지 일치) |
| 누적세로막대형 | 축 0~14 step 2, 누적 기하 유지, 우측 범례 |
| 백프로기준누적세로막대형 | 0%~100% 20% 간격 (종전과 동일 = 무회귀), 우측 범례 |
| 꺽은선형 | 축 0~6 라벨 0,2,4,6 (정답지 일치), 팔레트/제목/우측 범례 |
| 2차원원형 | 4슬라이스 팔레트(파랑/주황/회색/노랑 실측색), 우측 범례 1~4분기 |
| 표식만있는분산형 | Y 0~5 / X 0~3(0.5 간격) — 정답지 일치 |
| 기타(stock 2종) | placeholder 유지 — **C2 범위**(렌더 커버리지 밖, 계획 명기) |

작업지시자 시각판정용: **WASM 재빌드(docker wasm) + rhwp-studio vite dev 서버 기동** —
studio는 #1456 수정(rawSvgCount, PR #1514) 포함 트리라 차트 첫로드 공백 없음.

## 다음 단계

최종 결과보고서(`mydocs/report/task_m100_1882_report.md`) 승인 → origin push →
upstream devel PR (Refs #1431, #1882).
