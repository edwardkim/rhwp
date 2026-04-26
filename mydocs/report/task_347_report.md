# Task #347 최종 결과 보고서

## 이슈

[GitHub #347](https://github.com/edwardkim/rhwp/issues/347) — 표 HorzRelTo::Page / HorzAlign::Right 좌표 계산 오류 (다단 문서)

## 발견 경로

`samples/exam_eng.hwp`의 SVG 출력이 PDF 원본과 상이하다는 작업지시자 보고로 시작. 페이지 2/4의 박스형 안내문 콘텐츠가 잘못된 위치·누락 상태로 렌더링됨을 확인.

## 문제 정리 (6건, 단일 이슈로 통합)

| # | 위치 | 증상 | 원인 |
|---|------|------|------|
| 1 | p2 좌측 단 하단 "이제 듣기..." 박스 | 우하단으로 잘못 배치 | `HorzRelTo::Page`가 `col_area`(컬럼) 기준 |
| 2 | p2 박스 우측 정렬 | 페이지 우측 벗어남 | `HorzAlign::Right`에서 `+ h_offset` (그림 코드는 `-`) |
| 3 | (잠재) `VertRelTo::Page` 표 | 컬럼 기준 위치 | 주석은 body 기준이나 코드는 col_area |
| 4 | p2 우측 박스 내부 텍스트 | 70px 아래로 밀림 | 글뒤로 그림이 `y_offset += pic_height` 적용 |
| 5 | p4 우측 Q27 박스 | 프레임 누락 + 내용 겹침 | TAC 그림 미렌더 + InFrontOfText 표 push-down |
| 6 | p4 우측 Q28 박스 | 프레임이 표 텍스트 덮음 | `[표,그림]` 순서 컨트롤에서 z-order 보존 누락 |

## 수정 파일

- `src/renderer/layout.rs` — `current_body_area` 필드 + 페이지 진입 시 set, `layout_shape_item`에 TAC 그림 직접 렌더 분기
- `src/renderer/layout/table_layout.rs` — `HorzRelTo::Page`/`VertRelTo::Page`를 body_area 기준으로, `HorzAlign::Right` 부호 정정, InFrontOfText/BehindText push-down 미적용
- `src/renderer/layout/picture_footnote.rs` — 글뒤로/글앞으로 그림 y_offset 진행 차단

## 단계별 진행

| 단계 | 내용 | 보고서 |
|------|------|--------|
| 1 | body_area 통로 추가 (refactor) | `working/task_347_stage1.md` |
| 2 | HorzRelTo::Page / VertRelTo::Page / Right 부호 정정 | `working/task_347_stage2.md` |
| 3 | (스코프 확장) TAC 그림 렌더 + InFrontOfText push-down 정정 | `working/task_347_stage3.md` |

## 시각 검증 산출물

- `mydocs/working/task_347_exam_eng_p2_after.png` — p2 박스 위치 + 박스 내부 텍스트 정상
- `mydocs/working/task_347_exam_eng_p4_after.png` — p4 Q27 City Pass Card 박스 정상
- `mydocs/working/task_347_exam_eng_p4_q28_after.png` — p4 Q28 Lockwood Snow Festival 박스 정상 (z-order 수정 후)

## 회귀 검증

- `cargo test --release`: **1047+ passed, 0 failed**
- 시각 회귀: `samples/exam_eng.hwp` 8페이지 전수 확인 + 추가 샘플 (`biz_plan.hwp`, `aift.hwp`, `equation-lim.hwp`) export-svg 빌드 회귀 없음

## 영향 매트릭스

| 케이스 | 변화 | 회귀 |
|--------|------|------|
| 단단 문서 / 모든 표 | col_area = body_area라 동작 동일 | 없음 |
| 다단 / HorzRelTo::Column | 분기 미변경 | 없음 |
| 다단 / HorzRelTo::Page (Left/Center/Right) | body_area 기준 + Right 부호 정정 | 의도된 정정 |
| 글뒤로/글앞으로 그림 + Para | y_offset 진행 차단 | 의도된 정정 |
| TAC 그림 + 텍스트 없는 빈 문단 | 직접 렌더 (신규) | 의도된 신규 동작 |
| TAC 그림 + 텍스트 있는 문단 | paragraph_layout 처리 (기존) | 없음 |
| InFrontOfText/BehindText Para 표 | push-down 미적용 | 의도된 정정 |
| TopAndBottom Para 표 | push-down 유지 | 없음 |
| TAC 표 / 중첩 표 / Paper 기준 | 미변경 | 없음 |
| 같은 문단의 [TAC 그림, InFrontOfText 표] 컨트롤 순서 | 그림이 표 앞으로 z-order 보존 | 의도된 정정 |

## 결론

5건의 절대 좌표 계산 결함을 단일 이슈에서 통합 수정. picture_footnote(그림/각주) 경로의 시맨틱(`HorzRelTo::Page → body_area`, `Right → -h_offset`, push-down 미적용)을 정답으로 삼아 표·그림 경로를 통일. body_area는 페이지별 1회 set되는 `Cell` 상태로 관리하여 기존 호출 시그니처 변경 회피.

이슈 클로즈는 작업지시자 승인 후 수행.
