# task_m100_1882 Stage 4 완료 보고서 — 갭③ 범례 우측 배치

- 이슈: #1882 (C1c, #1431 Track C)
- 브랜치: `local/task1882`
- 단계: Stage 4 / 5 — 범례 위치 (`src/ooxml_chart/{mod,parser,renderer}.rs`)

## 변경 내용

1. **모델** (`mod.rs`): `enum LegendPos { Bottom(default), Right, Left, Top }` +
   `OoxmlChart.legend_pos`. **default=Bottom** — `c:legend`/`legendPos` 미존재 시 현행 하단
   배치 유지 (모델을 직접 구성하는 기존 단위 테스트 보호, 구현계획서 승인 사항).
2. **파서** (`parser.rs`): `b"legendPos"` arm — r→Right, l→Left, t→Top, 그 외→Bottom.
   legendPos는 `c:legend` 안에서만 등장하므로 상태 플래그 불요.
3. **렌더러** (`renderer.rs`):
   - `render_legend`를 `legend_items(chart)`(pie=카테고리별/일반=시리즈별 분기 추출) +
     `push_legend_swatch`(라인=선/그 외=10×10 사각형) + 가로 배치(기존) +
     신규 `render_legend_right`(행 16px 세로 스택, 플롯 세로 중앙 정렬)로 분해.
   - 레이아웃: Right이면 `legend_h=0` 대신 `legend_w = clamp(최장라벨×10 + 26, 50, w×0.30)`
     확보, `plot_w -= legend_w`. pie 경로 동일 분기. Left/Top은 하단 폴백(코퍼스 전 샘플 r).
   - 두 배치 모두 `<g class="hwp-chart-legend">` 래핑 (통합 테스트 위치 검증용).
   - `data_bar_xs` 기하 헬퍼의 범례 제외 필터(10×10)는 세로 배치에서도 유효 — 무회귀.

근거: 코퍼스 27종 전부 XML에 `<c:legendPos val="r"/>` 명시 (유일하게 XML 파싱으로
해결되는 갭), 정답지 PDF 전 샘플 우측 세로 스택.

## 테스트 (TDD — 구현 전 실패 확인)

- parser 2: `test_parse_legend_pos_right`(r→Right), `test_parse_legend_pos_default_bottom`.
- renderer 2: `test_render_legend_right_vertical`(범례 텍스트 x>260 우측 + y<250 세로 중앙 —
  `hwp-chart-legend` 그룹 내 좌표 파싱), `test_render_legend_bottom_default_unchanged`(y>270 하단 유지).

## 검증

```
cargo test --lib ooxml_chart                              → 44 passed, 0 failed
cargo test --test issue_1431_scatter                      → 1 passed
cargo test --test issue_1453_chart_3d_ofpie_routing       → 2 passed
```

**시각 확인** (`output/poc/chart_c1c/stage4/` ↔ 정답지): 묶은세로막대형·2차원원형 모두
범례가 우측 세로 스택(플롯 세로 중앙)으로 이동 — **4갭 전부 반영된 렌더가 정답지와 정합**
(제목 "차트 제목" / 파랑·주황·회색·노랑 팔레트 / 우측 범례 / 축 0~6 라벨 0,2,4,6).

## 다음 단계

Stage 5 — 통합 테스트(`tests/issue_1882_chart_style_gaps.rs`) + `cargo test` 전체 + clippy +
27종 시각검증(`output/poc/chart_c1c/`) → 최종보고서.
