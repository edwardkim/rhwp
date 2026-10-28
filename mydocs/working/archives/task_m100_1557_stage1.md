# Task #1557 Stage 1 완료보고서 — root-cause element 확정

## 목표
header.xml 내부 bisection 으로 한글 페이지 붕괴 유발 element 확정 + serializer 코드 매핑.

## 결과 — 원인은 `<hh:head secCnt>` 불일치

### 격리 (36382669 PASS, 8→1)
| 실험 | 한글 페이지 |
|------|-----------|
| 원본 | 8 |
| 저장본(rt) | 1 |
| rt + 원본 section0.xml | 1 (section0 원인 아님) |
| rt + 원본 header.xml | 8 (header 가 원인) |
| **rt + header 의 `secCnt="1"`→`"3"` 만 치환** | **8 (완전 복원)** ✅ |

다중 케이스 교차:
| 파일 | rt section*.xml 수 | 원본 | rt(secCnt=1) | rt(secCnt=3) |
|------|----:|----:|----:|----:|
| 36382669 | 3 | 8 | 1 | **8 (완전 복원)** |
| 36384160 | 3 | 29 | 1 | **3** (1→3 개선, 잔여 별도 요인) |

### 진단
- 저장본은 `Contents/section0..2.xml` **3개**를 쓰면서 `header.xml` 에는 **`secCnt="1"`** 기록.
- 한글은 `secCnt` 만큼만 구역을 로드 → 구역 1·2 무시 → 페이지 붕괴.
- 36384160 은 secCnt 교정으로 1→3 회복(완전 29 아님 — 잔여는 본문 내 다른 요인, 본 타스크 범위 외 가능).

### 코드 지점
`src/serializer/hwpx/header.rs:37`
```rust
let sec_cnt = doc.doc_properties.section_count.max(1).to_string();
```
- 섹션 **파일**은 `doc.sections`(=3) 기준으로 방출되나, `secCnt` 는 `doc.doc_properties.section_count`(=1, 파서가 갱신 안 한 stale 값)에서 가져와 **불일치**.
- `doc_properties.section_count` 가 실제 섹션 수와 어긋나는 것이 근인.

## 수정 방향 (Stage 2)
`secCnt` 를 실제 직렬화 대상인 **`doc.sections.len()`** 으로 산출(섹션 파일 수와 항상 일치).

## 부수 관찰 (별개·범위 외)
header.xml 의 기타 차이(페이지 붕괴와 무관): `imgBrush` mode TOTAL→FIT, `gradation` colorNum 누락,
`strikeout` 3D→NONE, `shadow` DROP→CONTINUOUS, TabDef switch(HwpUnitChar) 래퍼 미방출.
→ 시각 충실도 후속 이슈 후보(본 타스크는 secCnt 만 다룸).

## 다음
Stage 2 — `header.rs:37` 수정 + 대표 케이스 한글 페이지 복원 검증.
