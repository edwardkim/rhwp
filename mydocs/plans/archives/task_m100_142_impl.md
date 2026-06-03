# 구현 계획서: 수식 레이아웃 보정

- **타스크**: [#142](https://github.com/edwardkim/rhwp/issues/142)
- **마일스톤**: M100
- **브랜치**: `local/task142`
- **작성일**: 2026-04-14
- **수행계획서**: `mydocs/plans/task_m100_142.md`

## 단계 구성 (3단계)

---

### 1단계: 인라인 수식 조판 영역 너비 보정

**핵심 문제**: `composer.rs:130-132`에서 수식의 TAC 너비를 `eq.common.width`(HWP 저장값)로 사용하나, rhwp 레이아웃 엔진의 실제 렌더링 너비와 불일치하여 텍스트 겹침 발생.

**수정 파일**: `src/renderer/composer.rs`

**변경 내용**:
- 수식 컨트롤의 TAC 너비를 HWP 저장값 대신 레이아웃 엔진 산출값으로 대체
- 수식 스크립트 → tokenize → parse → EqLayout → LayoutBox.width 계산
- LayoutBox 너비를 px → HWPUNIT 역변환하여 tac_controls에 설정
- 높이도 동일하게 보정

---

### 2단계: 레이아웃 상수 보정

**수정 파일**: `src/renderer/equation/layout.rs`

**변경 내용**:
- exam_math.hwp 렌더링 결과를 기반으로 상수 조정
- BIG_OP_SCALE, MATRIX_COL_GAP, MATRIX_ROW_GAP 등 (필요 시)

---

### 3단계: 테스트 및 검증

- `cargo test` 전체 통과
- exam_math.hwp SVG 출력 → 수식-텍스트 겹침 해소 확인
- output/svg/ 폴더에 검증용 SVG 생성

---

## 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | 수식-텍스트 겹침 해소, `cargo test` 통과 |
| 2단계 | 레이아웃 상수 조정 후 시각 비교 |
| 3단계 | exam_math.hwp 전체 렌더링 검증 |
