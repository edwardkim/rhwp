# 최종 결과보고서 — Task M100 #1892

## 이슈

[#1892 대법원 서식 hwp 계열 HWP5 라운드트립 IR_DIFF — 페이지 내 초대형 변위 (최대 5449px, 표본 198중 15 비-PASS)](https://github.com/edwardkim/rhwp/issues/1892)

## 요약

대법원 행정예규 서식 hwp 는 실체가 **HWP3 V3.00** 이며, 결함 축은 이슈가 추정한
"HWP5 직렬화기 fidelity(IR_DIFF)"가 아니라 **HWP3 파스 → export_hwp_with_adapter
→ 재파스 경로의 4중 결함 체인**이었다. 4곳을 수정하여 대표 2912309 의
5449px STRUCT 가 0.00px PASS 로 수렴했고, 20k 검사 비-PASS .hwp 전건 재측정에서
악화 0 으로 광범위 개선을 확인했다.

## 진단 노트 — IR_DIFF=1 단서의 정체

이슈 단서였던 `hwp5-roundtrip` IR_DIFF=1 은 **어댑터 미경유 plain serialize**
(`serialize_document`) 경로에서 HWP3 문서의 SectionDef(PAGE_DEF)가 통째로
소실되는 도구-경로 전용 현상(SectionPageDef diff)이었다 (#1893 함정 노트 (1)
동형 — 진단 도구와 제품 경로의 직렬화 상태 불일치). 제품 경로
(`export_hwp_with_adapter`)는 `convert_if_hwpx_source` 가 HWP3 에도 SectionDef
컨트롤을 삽입하므로 페이지 기하는 보존된다. 실제 렌더 변위의 축은 별개의 4건:

## 근본 원인 (4중 결함 체인)

### ① 빈 묶음('$con' children=0) 재파스 사각형 오분류

HWP3 그리기 트리의 컨테이너(object_type 0)는 자식 없이 존재할 수 있다.
파서의 중첩 Group 판별(`parse_container_children`)과 최상위 GSO 판별이
"CONTAINER 태그 ∨ 하위 SHAPE_COMPONENT 존재"만 보므로 빈 묶음이 `_ =>
Rectangle` 폴백으로 오분류 → 렌더 트리 `Group: 8→4 / Rect: 0→4` 구조 분기.

**수정**: `tags::SHAPE_CONTAINER_ID`('$con') 신설, 두 판별에 component
ctrl_id 검사 추가 (`src/parser/control/shape.rs`, `src/parser/tags.rs`).

### ② rendering matrix 폴백의 offset 승격 → 그룹 자식 이동 (최대 10485px)

`write_shape_component_base` 의 행렬 폴백(raw_rendering 없음·회전 없음·explicit
행렬 없음)이 translation 을 `offset_x/y` 로 합성했다. 그룹 자식 위치의 단일
권위는 `render_tx/ty` 이고(렌더러 `layout_group_child_*` 는 offset 을 읽지
않음, object_ops 그룹 생성도 render_tx 명시), 위치를 의도한 IR 은 항상
explicit 경로로 간다. 폴백에 오는 offset 은 위치가 아닌 메타데이터(HWP3
relative_pos)인데 행렬로 살아나 재파스 자식이 relative_pos 만큼 이동했다.

**수정**: 폴백 translation 을 identity 로 (`src/serializer/control.rs`).

### ③ HWP3 attr 크기 기준 비트 누락 → 종이비례 팽창 (5449px 의 실체)

HWP3 `build_common_obj_attr` 가 크기 기준 비트(15-17 너비 / 18-19 높이)를
누락해 0 = **Paper(퍼센트)** 로 저장. 재파스 렌더의 `resolve_object_size` 가
개체 크기를 종이×(HWPUNIT 크기/10000)배로 해석 — 실측 w×5.95 h×8.42
(=종이 59528/84188 HU ÷ 10000) 팽창이 Group bbox 5449px 변위의 실체.
IR 필드(`width_criterion=Absolute`)와 attr 비트가 불일치하는 stale-pair 결함.

**수정**: 파서 디코드와 동일 매핑으로 비트 기록 (`src/parser/hwp3/mod.rs`).

### ④ 탭 "데이터 없음" 마커의 IR 유입 → 탭 무폭화 (130~478px)

직렬화기는 tab_extended 없는 탭에 `[0,...,0,0x0009]` 8유닛 마커를 방출한다
(HWP5 포맷 필수). 파서가 이를 tab_extended 로 실으면 레이아웃이 `ext[0]=0`
을 탭 결과 위치로 해석해 탭이 무폭이 된다. 한컴 실측 탭 확장은 ext[2]
고바이트 = 종류 enum+1 (≥256) 이라 전부-0 실데이터는 존재하지 않음 —
마커 판별은 안전하다.

**수정**: null 마커는 tab_extended 미적재 (`src/parser/body_text.rs`).

**#1244 계약 정제**: 기존 `issue_1244_tab_extended_fallback` 테스트 2건은 재파스
IR 에 마커 항목이 "존재"할 것을 단언했으나, #1244 의 실계약은 바이트 레벨
(한컴이 ext[6]=0x0009 를 요구)이다. 마커 방출은 불변이고 파서 정규화만 추가됐다.
단언을 "탭 문자 보존 + tab_extended 빈 상태"로 갱신 — 직렬화기가 마커를 누락
(ext[6]=0)하면 파서가 null 마커로 인식하지 못해 ext[6]=0 항목이 유입되므로
갱신된 단언이 #1244 회귀도 그대로 잡는다. 재파스 IR 이 삽입 직후 IR 과
동일해지므로(빈 tab_extended) 저장/재열기 렌더 불변성은 오히려 강화.

## 검증

### 대법원 15건 (이슈 표본)

| 파일 | 사전 | 사후 |
|---|---|---|
| 2912309 (제적등본2) | STRUCT 5449.16 | **PASS 0.00** |
| 2912311 (제적초본) | STRUCT 5039.64 | **PASS 0.00** |
| 3050521 (소송구조) | OVER 391.73 | **PASS 0.00** |
| 2952505 (문서건명부) | OVER 130.0 | **PASS 0.00** |
| 2912821 (송달료납부) | OVER 4.0 | **PASS 0.00** |
| 2955381 (기일입찰표) | STRUCT 1913.76 | STRUCT 275.0 (개선) |
| 2955295/2955311/2955331 | OVER 398/478/465 | OVER 231/382/376 (개선) |
| 2955219/2955227 | OVER 104/104 | OVER 103/103 |
| 2912179/2912237/2912875/2957829 | PAGE/OVER 소폭 | 불변 |

PASS 0/15 → **5/15**, 악화 0.

### 20k 검사 비-PASS .hwp 전건 재측정 (37건)

**FIXED(→PASS) 17 / BETTER 5 / SAME 12 / WORSE 0** (잔여 3건은 선존 LOAD_FAIL,
불변). 대법원 밖 정부 서식(부산·인천·도봉 등 별지 서식 OVER 1323/1656/1747px)도
전환 — 결함이 HWP3 변환 서식류 전반에 걸친 계열 결함이었음을 확인.

특기: 20k 검사의 TIMEOUT 클러스터 중 8건이 PASS 0.00 전환 — ③ 종이비례 팽창이
레이아웃 폭주(렌더 타임아웃)의 원인이기도 했다.

### 회귀 게이트

- `tests/issue_1892.rs` 4건 (렌더 자기정합 2 + IR 핀 2) PASS
- cargo test 전 스위트: **PASS** (GitHub Actions 최신 head 기준 `Build default-feature tests` /
  `Build & Test` 성공, maintainer 로컬 `cargo test --profile release-test --tests` 통과)
- big_hwp 2,500 A/B (origin/devel base exe vs fix exe): **회귀 0 / 개선 2**
  (admrul_0646 OVER 247→PASS, admrul_0645 465→323). 비-PASS 6→5, STRUCT/PAGE/LOAD_FAIL 0 유지.
- big_hwpx 2,500 A/B: **완전 동일** (PASS 2483/STRUCT 9/OVER 8 양측 일치, 파일별
  diff 0) — HWPX 경로 무영향 확인.

## 남는 축 (후속)

1. **char_shapes start_pos 단위공간 전사** — HWP3 IR 의 char-index 공간
   start_pos 를 직렬화기가 HWP5 code-unit 공간으로 전사하지 않음. 잔여
   103~382px TextRun 축(2955219/227/295/311/331/381)의 원인. #1773 "HWP3↔HWP5
   재파스 컴포저 조성 불변성" 클래스 — 별도 설계 사안.
2. **2912179 PAGE_MISMATCH** (1→2쪽, TextLine −1) — 1과 동류 여부 후속 판별.
3. **hwp5-roundtrip 단일 모드의 HWP3 입력** — 어댑터 미경유라 SectionPageDef
   소실이 IR_DIFF 로 표면화(배치 모드는 HWP3 사전 제외). 도구 일관성 축.

## 산출물

- 수정 5파일: parser/control/shape.rs, parser/tags.rs, parser/body_text.rs,
  parser/hwp3/mod.rs, serializer/control.rs
- 픽스처 2: samples/issue1892_hwp3_drawing_group_roundtrip.hwp (2912309),
  samples/issue1892_hwp3_tab_roundtrip.hwp (2952505)
- 테스트: tests/issue_1892.rs
- 문서: plans/task_m100_1892.md, plans/task_m100_1892_impl.md, 본 보고서
