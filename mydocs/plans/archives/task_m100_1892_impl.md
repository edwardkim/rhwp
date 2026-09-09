# 구현 계획서 — Task M100 #1892

## 근본 원인 (4중 결함 체인)

대법원 서식 hwp = **HWP3 V3.00**. 경로: HWP3 파스 → `export_hwp_with_adapter`
(HWP5 직렬화) → 재파스. IR은 보존되는데 렌더가 갈라지는 지점 4곳:

### ① 빈 묶음 재파스 사각형 오분류 (parser/control/shape.rs)

HWP3 그리기 트리의 컨테이너(object_type 0)가 자식 없이 존재(children=0).
직렬화기는 '$con' SHAPE_COMPONENT 로 올바르게 방출하지만, 파서의 중첩 Group
판별이 "CONTAINER 태그 존재 ∨ 하위 SHAPE_COMPONENT 존재"만 보므로 빈 묶음이
`_ => Rectangle` 폴백으로 떨어짐 → 렌더 트리 Group→Rect 구조 분기 (STRUCT).
최상위 GSO 판별(`is_container`)에도 같은 구멍.

### ② rendering matrix 폴백의 offset 승격 (serializer/control.rs)

`write_shape_component_base` 폴백(raw_rendering 없음 + 회전 없음 + explicit
행렬 없음)이 translation 을 `offset_x/y` 로 합성. 그룹 자식 위치의 단일 권위는
`render_tx/ty` 이며(렌더러 `layout_group_child_*`, object_ops 그룹 생성 모두),
위치를 의도한 작성자는 항상 render_tx 를 명시 → explicit 경로로 감. 폴백에
도달하는 offset 은 위치가 아닌 메타데이터(HWP3 relative_pos)인데 행렬로 살아나
재파스 그룹 자식이 relative_pos 만큼 이동 (최대 10485px).

### ③ HWP3 크기 기준 비트 누락 (parser/hwp3/mod.rs)

`build_common_obj_attr` 가 attr 비트필드 조립 시 크기 기준(bit 15-17 너비 /
18-19 높이)을 누락 → 0 = Paper(퍼센트) 로 저장. 재파스 렌더가
`resolve_object_size` 에서 종이×(크기/10000)배로 팽창 (w×5.95, h×8.42 —
Group bbox 5449px 급 변위의 실체).

### ④ 탭 "데이터 없음" 마커의 IR 유입 (parser/body_text.rs)

직렬화기는 tab_extended 없는 탭에 `[0,0,0,0,0,0,0x0009]` 8유닛을 방출(포맷
필수). 파서가 이를 tab_extended 로 실으면 레이아웃이 `ext[0]=0` 을 탭 결과
위치로 해석해 탭이 무폭 → 탭 들여쓰기 서식 행 이동 (130~478px). 한컴 실측
탭 확장은 ext[2] 고바이트=종류 enum+1 이라 전부 0 인 실데이터는 없음 —
null 마커 판별은 안전.

## 수정 단계

1. **'$con' 상수 + 빈 묶음 판별** — `tags::SHAPE_CONTAINER_ID` 신설,
   `parse_container_children` 자식 판별과 최상위 `is_container` 판별에
   `ctrl_id == '$con'` 추가. 직렬화기 리터럴 `0x24636f6e` 도 상수로 치환.
2. **행렬 폴백 identity 화** — 폴백 translation 을 (0,0) 으로. explicit/
   raw_rendering/회전 경로는 불변.
3. **HWP3 criterion 비트 보강** — `build_common_obj_attr` 에 파서 디코드와
   동일 매핑으로 bit 15-17/18-19 기록.
4. **null 탭 마커 스킵** — `parse_para_text` 에서 null 마커면 tab_extended
   미적재 (탭 문자·char_offsets 는 그대로).
5. **픽스처 + 핀** — 2912309/2952505 를 samples/ 로, `tests/issue_1892.rs`
   에 렌더 자기정합 2건 + IR 핀 2건.

## 검증 계획

- 대법원 15건 재측정 (사전: PASS 0/15)
- `cargo test` 전 스위트
- big_hwp/big_hwpx 2,500 코퍼스 A/B (origin/devel 베이스 exe vs fix exe)

## 남기는 축 (후속)

- char_shapes `start_pos` 단위공간 전사(HWP3 char-index ↔ HWP5 code-unit)
  — 잔여 103~382px TextRun 축, #1773 "컴포저 조성 불변성" 클래스. 별도 설계.
- 2912179 PAGE_MISMATCH (TextLine −1) — 동류 여부 후속 판별.
- `hwp5-roundtrip` 단일 모드의 HWP3 입력 처리(어댑터 미경유 SectionPageDef
  소실 노출) — 도구 축, 이슈 코멘트로 기록.
