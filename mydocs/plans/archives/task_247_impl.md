# Task 247 구현 계획서 (수정): 도형 인라인 컨트롤 구조 전환

## 완료된 작업
- ✅ `create_shape_control_native` 인라인 삽입으로 변경
- ✅ 한컴 호환 직렬화 (5개 시나리오 통과)
- ✅ PARA_HEADER 예약 바이트 수정, flip/attr/description 한컴 기본값

## 근본 원인 분석

현재 "후처리 마커 삽입(MarkerInsert)" 방식은 좌표 동기화 불가능:
1. MarkerInsert가 shift한 좌표와 getCursorRect의 원본 좌표 불일치
2. 같은 char_start에 마커와 텍스트 런 2개 존재 → 충돌
3. navigable_text_len이 겹치는 위치를 이중 계산

## 근본 해결 방향

**인라인 도형을 텍스트 조합(composition) 단계에서 실제 텍스트로 취급**

### 단계별 계획

#### 1단계: navigable_text_len 정확한 계산
- 인라인 컨트롤의 text_position을 고려하여 `max(text_len, max_ctrl_pos + 1)` 반환
- 겹치는 위치를 이중 계산하지 않음

#### 2단계: 조판부호 모드에서 도형을 텍스트로 조합
- MarkerInsert 후처리 대신, **compose 단계에서 도형 마커 텍스트를 문단 텍스트에 통합**
- 조판부호 모드: "[선]", "[사각형]", "[타원]"을 텍스트로 삽입
- 일반 모드: FFFC(U+FFFC, Object Replacement Character)로 0폭 치환
- 이렇게 하면 마커가 일반 TextRun으로 렌더링되어 커서 네비게이션이 자연스럽게 동작

#### 3단계: getCursorRect 인라인 컨트롤 위치 지원
- 2단계에서 마커가 TextRun으로 통합되면 getCursorRect가 자연 동작
- 일반 모드에서도 0폭 앵커로 커서 위치 제공

#### 4단계: F11 도형 선택 + 스페이스 삽입 테스트
- 커서가 도형 사이를 이동하면 F11이 자연 동작
- 도형 사이에 스페이스/텍스트 삽입 확인
