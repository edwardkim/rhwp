# Task #347 수행 계획서 — 표 HorzRelTo::Page / HorzAlign::Right 좌표 계산 오류

## 1. 배경

`samples/exam_eng.hwp` 페이지 2 SVG 출력이 PDF 원본과 상이.

- **증상 A** — 좌측 단 하단 안내 박스("이제 듣기 문제가 끝났습니다…")가 우하단으로 잘못 배치
- **증상 B** — 우측 단 상단에 불필요한 여백

두 증상은 동일 표(pi=102, 1×1, 글앞으로)의 절대 좌표 계산 오류에 기인함.

## 2. 원인

`src/renderer/layout/table_layout.rs`:

| 위치 | 현재 코드 | 문제 |
|------|---------|------|
| L942 | `HorzRelTo::Page => (col_area.x, col_area.width)` | Page 기준을 컬럼으로 처리 (그림은 body_area) |
| L949 | `HorzAlign::Right => ref_x + (ref_w - tw) + h_offset` | h_offset 부호 반대 (그림은 `-`) |
| L994 | `VertRelTo::Page => (col_area.y, col_area.height)` | 주석은 body 기준이라 했으나 코드는 col_area |

대조군: `src/renderer/layout/picture_footnote.rs` L175–203 — 그림/각주 경로는 정상 처리.

## 3. 검증 데이터

- `samples/exam_eng.hwp` p.2 — 회귀 검증용 핵심 샘플
- `samples/exam_eng.pdf` — 시각 비교 정답
- pi=102 dump:
  - common: `vert=Page(0)`, `horz=Page(34865=123.0mm)`, `valign=Bottom`, `halign=Right`, `wrap=글앞으로`, size=112×16.1mm
  - 정답 위치: x≈31mm(좌측 본문 시작), y≈345mm(좌측 단 하단)
  - 현재 출력: x≈159mm, y≈345mm (컬럼 1 영역)

## 4. 단계 구성 (3단계)

### 단계 1: 회귀 재현 + 본문 영역 파라미터 통로 마련
- `compute_table_x_position` / `compute_table_y_position` 시그니처에 `body_area: &LayoutRect` 추가
- 호출부 (table 본문 배치, TAC 표 등) 모두에 `body_area` 전달
- 이 단계에서는 동작 변경 없음 — 단순 파라미터 통로만 확보 (refactor)
- 검증: `cargo build`, `cargo test` 그대로 통과

### 단계 2: HorzRelTo::Page / VertRelTo::Page를 body_area 기준으로 정정 + Right 부호 수정
- L942: `HorzRelTo::Page => (body_area.x, body_area.width)`
- L949: `HorzAlign::Right => ref_x + ref_w - table_width - h_offset`
- L994: `VertRelTo::Page => (body_area.y, body_area.height)`
- 검증:
  - `rhwp export-svg samples/exam_eng.hwp` → p.2 박스 위치, 우측 단 상단 여백 시각 확인
  - 기존 회귀 테스트 (`re_sample_gen`, `cargo test`) 통과
  - `dump-pages` / `ir-diff`로 다른 샘플 (특히 단단 문서) 회귀 없음 확인

### 단계 3: 회귀 샘플 추가 + 최종 보고
- `output/re/` 재현 검증용 SVG 생성 (exam_eng p.2)
- `mydocs/working/task_347_stage{N}.md` 단계별 보고
- `mydocs/report/task_347_report.md` 최종 보고
- `mydocs/orders/20260426.md` 갱신

## 5. 회귀 위험

- `HorzRelTo::Page` + 표는 다단 문서에서만 컬럼≠본문이라 영향. 단단 문서는 col_area=body_area라 변화 없음
- TAC(글자처럼) 표는 다른 분기를 타므로 영향 없음
- `picture_footnote` 경로와 시맨틱이 일치되어 향후 유지보수 단순화

## 6. 산출물

- 코드 수정: `src/renderer/layout/table_layout.rs` (+ 호출부)
- 문서: `mydocs/plans/task_347.md`, `mydocs/plans/task_347_impl.md`,
  `mydocs/working/task_347_stage{1,2,3}.md`, `mydocs/report/task_347_report.md`
- 회귀 샘플: `output/re/exam_eng_p2.svg` (필요 시)
