# Task #297 최종 결과보고서

## 이슈 요약

**원 보고 제목**: exam_math.hwp 12쪽 우측 컬럼 단락 높이 과대 — 동전 그림 위치 어긋남 #297

**실제 증상**: `samples/exam_math.hwp` 12쪽(및 16, 20쪽)에서 하단 "* 확인 사항" 박스가 PDF 대비 약 147 px 아래로 밀려 본문 하단 경계를 침범.

## 원인

`src/renderer/layout/table_layout.rs` `compute_table_y_position`이 `VertRelTo::Page`와 `VertRelTo::Paper`를 동일하게 `(ref_y=0, ref_h=page_h_approx)`로 처리. HWP 스펙상 두 값은 서로 다른 기준 영역을 가리킴:

- **Paper**: 용지 전체 (절대 좌표)
- **Page**: 쪽 본문 영역 (body area, 머리말/꼬리말/여백 제외)

이 구분이 없어 `vert=Page/141, valign=Bottom` 푸터 표가 용지 바닥 기준으로 계산되어 아래로 밀림.

### 수치 검증

exam_math.hwp 문단 1.22 표 (size=419.5×134.7 px, `v_offset=141 HU=1.9px`, `valign=Bottom`):

| 경우 | 공식 | 결과 |
|------|------|------|
| 수정 전 | `0 + 1508.1 − 134.7 − 1.9` | 1371.5 px |
| 수정 후 | `147.4 + 1213.3 − 134.7 − 1.9` | **1224.1 px** |
| PDF 기준 | `pdftotext bbox` 텍스트 TOP 1231.5 px에서 역산 | ~1226.5 px ✓ |

## 수정 내용

**파일**: `src/renderer/layout/table_layout.rs:986-992`

```diff
+            // Task #297: Page는 본문 영역(body area) 기준, Paper는 용지 전체 기준
+            // (HWP 스펙: Page=쪽 본문, Paper=용지 전체). 바탕쪽 문맥에서는
+            // col_area = paper_area이므로 두 경로 결과가 동일하여 회귀 없음.
             let (ref_y, ref_h) = match vert_rel_to {
-                crate::model::shape::VertRelTo::Page => (0.0, page_h_approx),
+                crate::model::shape::VertRelTo::Page => (col_area.y, col_area.height),
                 crate::model::shape::VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)),
                 crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
             };
```

## 검증 결과

### 단위 검증
- exam_math.hwp 12쪽 "* 확인 사항" 텍스트 y좌표: 1385.47 → **1238.08** (PDF 1242.5 ± 4 일치)
- 시각 비교 (PDF vs SVG 병렬): 박스 위치 및 내용 일치

### 회귀 검증

**본문 `VertRelTo::Page` 표 스캔** (145개 샘플 중 13건):

| 샘플 | 패턴 | 결과 |
|------|------|------|
| exam_math.hwp (pi=22, 55, 86) | Page/141 Bottom TopAndBottom | 수정 반영 (pages 12/16/20) |
| exam_math_no.hwp (동일 3건) | 동일 | 동일 수정 반영 |
| tac-case-001..005.hwp | Page/0 Top TopAndBottom | diff=0 (무회귀) |
| exam_social.hwp (p=61) | Page/477 Bottom TopAndBottom | -1 byte (수치 포맷 미차) |
| exam_eng.hwp (p=102) | Page/0 Bottom InFrontOfText | -1 byte (수치 포맷 미차) |

**바탕쪽 `VertRelTo::Page` 표** (5건): 모두 `col_area = paper_area`이므로 수학적 동치 → **diff=0**

**exam_math.hwp 전 20페이지**: 12/16/20만 변경(각 섹션 마지막 페이지), 나머지 17페이지 byte-identical

### 테스트
- `cargo test --release --lib`: **988 passed / 0 failed**
- `cargo clippy --release --lib`: 경고 없음

## 영향 범위

**영향 받음**:
- exam_math.hwp, exam_math_no.hwp — 각 섹션 마지막 페이지의 "* 확인 사항" 박스 위치 정상화

**영향 없음**:
- 본문 `VertRelTo::Paper` 표
- 본문 `VertRelTo::Para` 표
- 바탕쪽 모든 표 (col_area=paper_area 조건)
- HWP 표 외의 모든 요소 (문단/그림/수식)

## 조사 과정에서의 교훈

### 가설 정정

초기 원인 가설(1단계): 바탕쪽(master page) 표가 `vert=Paper/101954 HU, valign=Top`으로 저장되어 있고 한컴이 이를 표 BOTTOM 기준으로 해석 → bottom-anchoring 수정 필요.

**이 가설은 틀렸음**. 3단계 진입 후 실제 수정 적용 결과 시각 변화 없음을 발견. 재조사로 판명:
- 바탕쪽 1x3 표는 빈 표였고 내용 없음 → 시각 영향 무관
- 실제 "* 확인 사항" 박스는 **본문 pi=22의 `VertRelTo::Page` 표**
- 근본 원인은 Page/Paper 기준점 미구분

**교훈**: 가설을 수립한 뒤 **가장 빠른 검증 방법**(본 케이스는 실제 수정 + SVG 비교)으로 조기에 반증하는 것이 중요. 1단계에서 dump로 바탕쪽 표만 보고 본문 pi=22를 놓친 것이 1차 원인.

### 범위 의식

원래 이슈 제목("동전 그림 위치 어긋남")은 **증상 오인**이었음. 실제 동전(pi=33) y좌표는 PDF와 일치(≈497 px). 드리프트는 전혀 다른 본문 영역 표(pi=22)에서 발생. 이슈 제목만 따라가지 않고 **실측으로 증상을 재확인**한 후 범위를 재정의한 판단이 옳았음.

## 산출물

- `mydocs/plans/task_m100_297.md` (수행계획서)
- `mydocs/plans/task_m100_297_impl.md` (구현계획서, v1 폐기 → v2 교체)
- `mydocs/working/task_m100_297_stage1.md` (원인 확정 — 초기 가설, 부분 오류 포함)
- `mydocs/working/task_m100_297_stage3.md` (수정 + 단위 검증)
- `mydocs/report/task_m100_297_report.md` (본 문서)
- 소스: `src/renderer/layout/table_layout.rs` (6줄 변경, 주석 포함)

## 후속 과제 후보 (범위 외)

- **pi=28/30/32 문단 높이 계산 불일치**: dump-pages의 h값이 실제 렌더와 차이 (~30-40 px). 현재는 vpos 보정으로 y좌표는 맞지만 pagination 입력값 부정확. 잠재적 오버플로 원인 가능 — 별도 이슈 후보.
- **`compute_table_y_position`에서 page_h_approx 정확도**: `col_area.y * 2.0 + col_area.height`는 상하 여백이 다른 경우(exam_math: 위 39mm, 아래 34mm) 부정확. 실제 paper_h를 사용할 수 있도록 리팩터링 검토.

## 이슈 종료 조건

작업지시자 승인 후 `gh issue close 297` 실행.
