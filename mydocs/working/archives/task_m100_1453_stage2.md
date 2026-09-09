# Task M100 #1453 단계별 완료보고서 — 2단계: 회귀 통합 테스트 + 문서 주석

- 이슈: #1453
- 브랜치: `local/task1453`
- 단계: 2/3 (회귀 통합 테스트 + `mod.rs` 범위 주석)
- 작성일: 2026-06-21

## 구현 내용

### (a) 신규 회귀 통합 테스트 `tests/issue_1453_chart_3d_ofpie_routing.rs`

- `issue_1156_chart_column_flow.rs` 의 `render_page_svg` 패턴 재사용
  (`HwpDocument::from_bytes` → `render_page_svg(0)`).
- 7종 × (hwp, hwpx) = **14파일** 루프, 각 page 0 SVG 단언:
  - `!svg.contains("차트 (미지원)")` — placeholder 미발생
  - `svg.contains("hwp-ooxml-chart\"")` — 정상 차트 클래스 존재
  - `!svg.contains("hwp-ooxml-chart-fallback")` — fallback 아님

### (b) `src/ooxml_chart/mod.rs` 지원 범위 주석 갱신

- "지원 범위"에 `bar3DChart`·`pie3DChart`·`ofPieChart` 2D 근사 라우팅(C1a #1453) 명시.
- "범위 외"를 "3D 입체감·ofPie 보조플롯, 영역/산점도, stock(HLC), …" 로 재서술
  (3D 차트 전체 제외 → 입체감·보조플롯만 후속으로 정정).

## 검증 결과

```
$ cargo test --test issue_1453_chart_3d_ofpie_routing
test result: ok. 1 passed; 0 failed   (내부 14파일 단언)

$ cargo test --test issue_1156_chart_column_flow      # 기존 회귀
test result: ok. 2 passed; 0 failed
$ cargo test --test issue_1251_ole_chart_contents     # 기존 회귀
test result: ok. 10 passed; 0 failed

$ cargo clippy --test issue_1453_chart_3d_ofpie_routing
경고 0
```

### 실샘플 end-to-end 확인 (export-svg, 14파일)

| 종류 | hwpx | hwp |
|------|------|-----|
| 3차원묶은세로막대형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 3차원누적세로막대형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 3차원묶은가로막대형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 3차원누적가로막대형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 3차원원형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 원형대원형 | 미지원:0 chart:1 | 미지원:0 chart:1 |
| 원형대가로막대형 | 미지원:0 chart:1 | 미지원:0 chart:1 |

→ 14파일 모두 placeholder 0건 + 정상 차트 렌더. HWP=HWPX 동일.

## 완료 기준 충족

- [x] 신규 통합 테스트 통과 (14파일)
- [x] 기존 차트 테스트 회귀 없음 (`issue_1156`, `issue_1251`)
- [x] clippy 무경고

## 다음 단계

3단계 — 14파일 `export-svg` 산출물(`output/poc/chart_c1a/`) + `pdf/chart/` 정답지 대조표
+ 전체 테스트·빌드·clippy → 작업지시자 시각판정.
