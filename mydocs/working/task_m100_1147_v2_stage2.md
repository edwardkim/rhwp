# Task #1147 v2 Stage 2 보고서 — 본 페이지 + 회귀 검증

- **수행계획서**: [task_m100_1147_v2.md](../plans/task_m100_1147_v2.md)
- **구현계획서**: [task_m100_1147_v2_impl.md](../plans/task_m100_1147_v2_impl.md)
- **Stage 1 보고서**: [task_m100_1147_v2_stage1.md](task_m100_1147_v2_stage1.md)

## 1. 본 페이지 (HWPX, section 0 page_num=5)

### SVG 박스 좌표 비교 (`output/svg/task1147_v2/_008.svg`)

| 항목 | Stage 1 v1 (수정 전) | Stage 1 v2 (수정 후) | 변화 |
|------|---------------------|---------------------|------|
| 표 9×18 하단 abs_y | 1000.03 | 1000.03 | (불변) |
| "※ 추진일정은…" 박스 상단 abs_y | 1018.05 | 1006.45 | **-11.6 px** |
| 표↔문단 간격 | 18.0 px | **6.4 px** | **-11.6 px** |

권위 PDF (한컴 2022 편집기 출력 — `pdf/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서-2022.pdf` 페이지 8) 와 시각 정합. 작업지시자가 지적한 "테이블과 문단 사이의 간격이 넓음" 해소.

### 페이지네이션 회귀 없음

`dump-pages -p 7` 출력:
```
=== 페이지 8 (global_idx=7, section=0, page_num=5) ===
  body_area: x=75.6 y=105.8 w=642.5 h=941.1
  단 0 (items=8, used=931.5px, hwp_used≈943.9px, diff=-12.4px)
    FullParagraph  pi=120 ... (이하 8 개 항목)
    ...
    FullParagraph  pi=127  h=31.7  vpos=68712  "※ 추진일정은 ..."
```

items=8 / used=931.5 px — Stage 1 v1 결과 (페이지네이션) 와 동일하게 유지.

## 2. cargo test 결과

### lib 전수

- **1411 passed, 0 failed, 6 ignored** — finished in 38.75s

### integration (`cargo test --tests`)

- 모든 integration 테스트 통과 (FAILED 0)
- svg_snapshot 8 케이스 (form-002, issue-147, issue-157, issue-267, issue-617, issue-677, table-text, table-text-page-0) 전부 통과 → **golden SVG 회귀 없음**

## 3. 다른 HWPX 회귀 (aift.hwpx)

`dump-pages` drift 분포 정상 범위 (대부분 0~-10 px, 본 변경으로 인한 비정상 페이지 없음).

| 페이지 | items | used | hwp_used | diff |
|--------|-------|------|----------|------|
| p4 | 44 | 929.5 | 929.5 | +0.0 |
| p5 | 13 | 261.2 | 261.2 | +0.0 |
| p6 | 18 | 815.1 | 816.8 | -1.7 |
| p7 | 18 | 959.4 | 962.3 | -2.9 |
| p16 | 37 | 951.5 | 951.5 | +0.0 |
| p18 | 15 | 979.4 | 957.6 | +21.8 |

(p18 +21.8 은 다른 케이스로 본 변경과 무관 — Stage 1 v1 보고서의 "남은/후속 이슈" 카테고리)

## 4. HWP5/HWP3 회귀

본 변경은 `self.is_hwpx_source.get() == true` 조건에서만 발동하므로 HWP5/HWP3 는 영향 없음 (Stage 1 v1 와 동일한 격리 패턴).

## 5. 결론

- 본 페이지 시각 간격 11.6 px 감소, 권위 PDF 정합 ✓
- 페이지네이션 회귀 없음 (items=8 유지) ✓
- cargo test 전수 통과 (lib 1411 + integration 전수) ✓
- golden SVG 회귀 없음 ✓
- HWP5/HWP3 영향 없음 (트리거 격리) ✓

## 6. 다음 단계

Stage 3: 최종 보고서 (`task_m100_1147_v2_report.md`) 작성 + Stage 1/2/최종 보고서 + 산출물 커밋. 작업지시자 승인 요청.
