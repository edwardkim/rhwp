# 최종 결과 보고서 — Task #110

**이슈**: [#110](https://github.com/edwardkim/rhwp/issues/110)
**타이틀**: HWPX 양식 컨트롤 파싱 미구현 — checkBtn/btn/radioBtn/comboBox/edit IR 처리
**마일스톤**: M100
**완료일**: 2026-04-13
**브랜치**: `local/task110`

---

## 결과 요약

HWPX 파일의 양식 컨트롤(체크박스, 라디오버튼, 콤보박스, 입력상자, 명령단추)이
IR로 파싱되어 SVG/canvas에 정상 렌더링됨을 확인하였다.

---

## 원인 분석

`src/parser/hwpx/section.rs`의 `<hp:run>` 파싱 분기에 양식 컨트롤 요소 처리가 없어
`_ => {}` 분기로 무시되었다.

HWP 바이너리 파서에는 `FormObject` / `FormType` 모델과 렌더러가 완비되어 있으므로
HWPX → `Control::Form` IR 변환만 추가하여 기존 렌더러를 재사용하였다.

---

## 수정 내용

### `src/parser/hwpx/section.rs`

#### 1. `parse_form_object()` 함수 신규 추가

HWPX 양식 컨트롤 5종을 `Control::Form`으로 변환하는 함수.

- 요소 속성 파싱: `name`, `caption`, `foreColor`, `backColor`, `enabled`, `value`
- `<hp:sz>` 자식에서 `width`, `height` 읽기
- `<hp:listItem>` (comboBox 항목) 수집
- `<hp:text>` (edit 내용) 읽기
- **버그 수정**: `end_tag`를 `local_name()`으로 저장 — 네임스페이스 접두어(`hp:`) 포함 시 End 태그 인식 실패로 표 전체 파싱 붕괴 발생

#### 2. `<hp:run>` 분기 5개 태그 추가

| HWPX 태그 | FormType |
|-----------|----------|
| `<hp:btn>` | PushButton |
| `<hp:checkBtn>` | CheckBox |
| `<hp:radioBtn>` | RadioButton |
| `<hp:comboBox>` | ComboBox |
| `<hp:edit>` | Edit |

---

## 검증

| 항목 | 결과 |
|------|------|
| form-002.hwpx 파싱 (checkBtn 180개) | ✅ Control::Form으로 파싱 |
| 체크박스 □/☑ SVG 렌더링 | ✅ 시각적 확인 |
| 페이지 수 (10페이지) | ✅ 파싱 붕괴 없음 |
| 785개 회귀 테스트 | ✅ 전체 통과 |
| WASM 빌드 + 브라우저 시각 검증 | ✅ 정상 렌더링 확인 |

---

## 미구현 (별도 이슈)

- 체크박스 셀 커서 진입 불가 → #111
- 체크박스 클릭 토글 → #112

---

## 커밋 목록

| 커밋 | 내용 |
|------|------|
| `8114e51` | HWPX 양식 컨트롤 파싱 구현 + end_tag 버그 수정 |
