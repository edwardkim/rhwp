# Task 321 설계서: 바탕쪽 아키텍처 재설계

## 1. 현재 구현 분석

### 바탕쪽 저장 구조 (HWP)
- SectionDef.master_pages: Vec<MasterPage>
- 인덱스 순서로 apply_to 결정: 0=Both, 1=Odd, 2=Even (3+= 확장Both)
- LIST_HEADER byte 18-19: 확장 속성 (마지막 쪽 페이지번호 등)

### 현재 선택 로직 (rendering.rs:831~864)
```
mp_both = 첫 번째 Both
mp_odd = 첫 번째 Odd
mp_even = 첫 번째 Even
ext = 두 번째 이후 Both
```
- 홀수 페이지: mp_odd ?? mp_both
- 짝수 페이지: mp_even ?? mp_both
- 확장: page_number==1에만 적용 (하드코딩)

### 문제점
1. 첫 번째 Both만 기본으로 사용 → science(5개)에서 잘못된 선택
2. 확장 Both = 마지막 쪽인데 1페이지에만 적용
3. Both가 여러 개일 때 역할 구분 없음
4. 마지막 쪽 판별 불가
5. 바탕쪽 표의 위치 계산이 body_area 대신 paper_area 사용 필요

## 2. 대조군 분석

### exam_social (1개)
- [0] Both: 세로선 + 헤더 표
- 구역 1에 Both + Odd 추가
- 단순: 모든 페이지에 동일 바탕쪽

### exam_math (2개)
- [0] Both: 헤더 표
- [1] Both(확장): 페이지번호 표 — **마지막 쪽**
- 핵심: 두 번째 Both = 마지막 쪽(페이지번호 포함)

### exam_kor (3개)
- [0] Both: 짝수 쪽용 (세로선 + 헤더 표)
- [1] Odd: 홀수 쪽용 (세로선 + 헤더 표, 좌우 반전)
- [2] Both(확장): 마지막 쪽 (헤더 + 세로선 + 페이지번호 표)

### exam_eng (3개)
- exam_kor와 동일 패턴

### exam_science (5개)
- [0] Both: 세로선만 (모든 페이지 공통 배경)
- [1] Both(확장): 세로선 + 헤더 표 — **마지막 쪽 또는 기본?**
- [2] Odd: 홀수 쪽 (세로선 + 헤더 표)
- [3] Even: 짝수 쪽 (세로선 + 헤더 표)
- [4] Both(확장): 세로선 + 헤더 + 페이지번호 표 — **마지막 쪽**

## 3. 바탕쪽 적용 규칙 (한컴 도움말 기반)

### 우선순위 (높은 것이 낮은 것을 대체)
```
임의 쪽 / 마지막 쪽 > 홀수 쪽 / 짝수 쪽 > 양 쪽(Both)
```

### 적용 로직
1. 기본 바탕쪽: 첫 번째 Both (모든 페이지에 적용)
2. Odd가 있으면: 홀수 페이지에서 Both **대체**
3. Even이 있으면: 짝수 페이지에서 Both **대체**
4. 확장 Both (마지막 쪽): 구역의 마지막 페이지에서 Odd/Even/Both **대체**
   - "겹치게 하기" 옵션이면 대체 대신 **추가**
5. 확장 Both (임의 쪽): 특정 페이지에서 대체 또는 추가

### 확장 Both 구분
- LIST_HEADER byte 18-19의 값으로 구분
  - 0x00: 일반 Both
  - 0x03: 마지막 쪽 (겹치기?)
  - 0x07: 마지막 쪽 + 겹치기?

## 4. 재설계 방안

### 4.1 MasterPage 모델 확장
```rust
pub struct MasterPage {
    pub apply_to: HeaderFooterApply,
    pub is_extension: bool,     // 확장 바탕쪽 여부 (마지막 쪽/임의 쪽)
    pub overlap: bool,          // 겹치게 하기
    pub ext_flags: u16,         // raw 확장 플래그 (byte 18-19)
    pub paragraphs: Vec<Paragraph>,
    // ... 기존 필드
}
```

### 4.2 바탕쪽 선택 로직 개선
```
fn select_master_page(mps, page_number, is_last_page, section_page_count) -> (Option<&MasterPage>, Vec<&MasterPage>)
1. base = 첫 번째 Both (is_extension=false)
2. if is_odd && Odd 있으면 → active = Odd
3. if is_even && Even 있으면 → active = Even
4. else → active = base
5. if is_last_page → 마지막 쪽 확장 Both가 있으면:
   - overlap=true: active + 확장 Both (extra_master_pages)
   - overlap=false: active = 확장 Both (대체)
6. return (active, extras)
```

### 4.3 바탕쪽 렌더링 개선
- 표/도형/이미지: compute_object_position으로 종이 기준 배치
- 모든 바탕쪽 컨트롤에 동일 로직 적용 (현재 표만 수정됨)
- 페이지 번호 치환: TextBox 내 AutoNumber(Page) 처리 (기존 구현 활용)
- **클리핑 예외**: 바탕쪽은 Body 클리핑(편집용지 영역) 밖에서 렌더링
  - 현재: MasterPage 노드가 Body 노드와 독립 → ✅ 이미 올바름
  - 주의: 바탕쪽 내부 표/도형이 종이 전체 영역에 배치 가능

## 5. 구현 단계
1. MasterPage 모델에 is_extension/overlap/ext_flags 추가
2. 파서에서 byte 18-19 파싱하여 is_extension/overlap 설정
3. 바탕쪽 선택 로직 개선 (rendering.rs)
4. 대조군 검증 (exam_social → exam_science 순서)
