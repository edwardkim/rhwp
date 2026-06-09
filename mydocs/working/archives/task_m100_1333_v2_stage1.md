# v2 구현+검증 보고서 — 단 구분선 길이 정정 (M100 #1333)

계획서: `mydocs/plans/task_m100_1333_v2.md`

## 1. 배경

Stage 2 는 이중 렌더 두 후보 중 **page-level(body 전체높이 고정)** 을 남겼으나,
PDF 픽셀 측정 결과 정답은 **zone emit(콘텐츠 높이)** 였다. (페이지 3·8·23 등 부분
페이지에서 page-level 이 과도하게 길었음.)

## 2. 수정 (`src/renderer/layout.rs`)

1. **Stage 2 게이트 되돌림** — `build_columns` 의 zone emit 조건에서
   `has_zone_specific_layout &&` 및 진입부 술어 계산 제거. 단일 다단 페이지
   (zone_layout=None → unwrap_or(layout))도 콘텐츠 높이로 emit.
2. **page-level 호출 제거** — 페이지 빌더의 `build_column_separators` 호출 + 가드 삭제,
   미사용 `build_column_separators` 함수 삭제. 단 구분선 = zone emit 단일 경로.
3. **y_end body 하단 캡** — `emit_zone_column_separators` 에서
   `y_end = y_end.min(body_area.y + body_area.height)`. 꽉 찬 페이지에서 prev_zone_y_end
   가 trailing 간격 등으로 body 를 초과해 구분선이 페이지 밖까지 그려지던 결함(p22 105%)
   정정. 부분 페이지·sub-page zone 은 콘텐츠가 body 하단보다 위라 영향 없음.

## 3. 검증

### 3.1 대상 문서 (HWP) — 단 구분선 끝 위치 vs PDF (한글 2022)

전 23쪽 **단일 구분선**, 끝 위치 PDF ±1.5% 이내, 페이지 밖 초과 없음:

| 페이지 | rhwp end% | PDF end% | diff |
|--------|-----------|----------|------|
| 1 | 89.4 | 90.3 | -0.9 |
| 2 | 94.6 | 95.5 | -0.9 |
| 3 | 73.2 | 74.2 | -1.0 (짧음 재현) |
| 4 | 93.5 | 95.0 | -1.5 |
| 5 | 93.0 | 94.5 | -1.5 |
| 6 | 97.3 | 96.4 | +0.9 (캡) |
| 8 | 63.9 | 64.8 | -0.9 |
| 17 | 97.3 | 96.3 | +1.0 (캡) |
| 19 | 97.3 | 96.7 | +0.6 (캡) |
| 22 | 97.3 | 96.9 | +0.4 (캡, 이전 105%) |
| 23 | 53.5 | 52.8 | +0.7 |

페이지 3 시각 확인: 구분선이 ~73% 에서 멈춰 PDF 와 일치.

### 3.2 회귀

- shortcut.hwp(#874): 9개 구분선(1,1,2,1,1,2,1) 전수 보존.
- exam_kor / exam_math / exam_social / interview / hwp-multi-001: 단 구분선 중복 0건.

### 3.3 테스트 / clippy

- `cargo test --release`: **2107 passed, 0 failed**.
- `cargo clippy --release`: 경고/에러 없음.

## 4. 결론

단 구분선을 **zone emit 단일 경로(콘텐츠 높이, body 하단 캡)** 로 통일하여
이중 렌더 제거 + 부분 페이지 짧은 구분선(PDF 정합)을 동시에 달성. 회귀 없음.
