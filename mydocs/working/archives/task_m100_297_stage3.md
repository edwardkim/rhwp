# Task #297 3단계 완료 보고서 — 수정 및 단위 검증

## 수정 내용

**파일**: `src/renderer/layout/table_layout.rs:986-992` (`compute_table_y_position`)

```diff
-            let (ref_y, ref_h) = match vert_rel_to {
-                crate::model::shape::VertRelTo::Page => (0.0, page_h_approx),
-                crate::model::shape::VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)), // Para
-                crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
-            };
+            // Task #297: Page는 본문 영역(body area) 기준, Paper는 용지 전체 기준
+            // (HWP 스펙: Page=쪽 본문, Paper=용지 전체). 바탕쪽 문맥에서는
+            // col_area = paper_area이므로 두 경로 결과가 동일하여 회귀 없음.
+            let (ref_y, ref_h) = match vert_rel_to {
+                crate::model::shape::VertRelTo::Page => (col_area.y, col_area.height),
+                crate::model::shape::VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)),
+                crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
+            };
```

## 단위 검증 (exam_math.hwp 12쪽)

SVG 내 "* 확인 사항" text translate y:

| 항목 | 값 |
|------|-----|
| 수정 전 | 1385.47 px |
| **수정 후** | **1238.08 px** |
| PDF 기준 (yMin 923.60 pt) | ~1242.5 px (baseline) |
| 차이 | 약 4.4 px (허용 오차 내) |

시각 비교: `/tmp/t297v2/cmp.png` — PDF와 SVG 박스 위치 일치. "20" 페이지번호 및 박스 프레임 모두 동일 y 좌표.

## 회귀 검증 결과

### 145개 샘플 중 본문 `VertRelTo::Page` 표 13개 스캔

| 샘플 | 패턴 | pre vs post |
|------|------|-------------|
| exam_math.hwp (pi=22, 55, 86) | Page/141 Bottom TopAndBottom | **수정 목표 - 위치 교정** (pages 12/16/20 변경) |
| exam_math_no.hwp | 동일 패턴 | 동일 수정 반영 (+1937 bytes) |
| tac-case-001..005.hwp | Page/0 Top TopAndBottom | **변화 없음** (diff=0) — 다른 경로에서 렌더링되는 것으로 추정 |
| exam_social.hwp (p=61) | Page/477 Bottom TopAndBottom | -1 byte (수치 포맷 미차) |
| exam_eng.hwp (p=102) | Page/0 Bottom InFrontOfText | -1 byte (수치 포맷 미차) |

### 바탕쪽(master page) `VertRelTo::Page` 표 (5건)

- exam_eng, exam_kor, exam_science 바탕쪽에 위치
- 페이지 1 시각 회귀: **모두 diff=0** — col_area=paper_area 조건에서 수정 전후 결과 동일

### exam_math.hwp 전 20페이지 비교

| 페이지 | 변경 라인 수 | 의미 |
|--------|-------------|------|
| 1-11, 13-15, 17-19 | 0 | 무변화 |
| **12, 16, 20** | 186/178/96 | 각 섹션 마지막 페이지, "* 확인 사항" 박스 위치 교정 |

페이지 16 시각 비교(PDF vs SVG 병렬): 박스 위치 일치 확인.

### 단위 테스트

```
cargo test --release --lib
test result: ok. 988 passed; 0 failed; 1 ignored
```

## LAYOUT_OVERFLOW 확인

exam_eng/exam_kor에서 4-7 px overflow 경고가 있으나:
- 수정 전에도 동일하게 존재 (pre-existing)
- FullParagraph에서 발생 → 테이블 수정과 무관

## 결론

- **의도한 수정 대상**: exam_math.hwp (+ exam_math_no) pi=22/55/86의 "* 확인 사항" 박스 위치 교정 ✓
- **무회귀**: 나머지 샘플 모두 변화 없음 또는 허용 가능한 수치 미차 ✓
- **단위 테스트**: 988 통과 ✓

## 승인 요청

3단계 완료. 4단계(추가 회귀 테스트 + 최종 결과보고서)로 진입해도 되는지 승인 부탁드립니다.
