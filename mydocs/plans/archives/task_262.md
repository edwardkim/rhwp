# Task 262 수행 계획서: 문단번호모양 고도화

## 현재 구현 상태

| 항목 | 상태 |
|------|------|
| Numbering/Bullet 모델 (7수준) | ✓ 구현 |
| HWP 바이너리 파서 | ✓ 구현 |
| 번호 텍스트 생성 (expand_numbering_format) | ✓ 구현 |
| NumberingState 카운터 | ✓ 기본 구현 |
| 12종 프리셋 (numbering-defaults.ts) | ✓ 구현 |
| 번호/글머리표 대화상자 | ✓ 기본 구현 |
| **시작 번호 방식 제어** | ✗ 미구현 |
| **컨텍스트 메뉴 "문단 번호 모양"** | ✗ 미구현 |
| **Ctrl+K,N 단축키** | ✗ 미구현 |

## 미구현 기능 (한컴 기준)

### 시작 번호 방식
1. **앞 번호 목록에 이어(C)**: 같은 numbering_id의 이전 번호에서 이어서 증가 (현재 기본 동작)
2. **이전 번호 목록에 이어(P)**: 다른 numbering_id였다가 돌아올 때 이전 카운터 복원
3. **새 번호 목록 시작(G)**: 해당 수준의 카운터를 지정 값으로 리셋
4. **수준 시작 번호(S)**: 특정 수준의 시작 번호 지정 (기본값: Numbering.level_start_numbers)

### 구현 계획

#### 1단계: NumberingState 확장 (Rust)
- "이전 번호 목록에 이어" 지원: 이전 numbering_id의 카운터를 히스토리로 보존
- "새 번호 목록 시작" 지원: 문단별 시작 번호 오버라이드 속성 추가
- Paragraph 모델에 `numbering_start_override: Option<u32>` 필드 추가

#### 2단계: WASM API
- `getNumberingInfo(sec, para)`: 현재 문단의 번호 정보 조회
- `setNumberingStart(sec, para, mode, startNum)`: 시작 번호 방식 설정

#### 3단계: 대화상자 UI 강화
- 시작 번호 방식 라디오 버튼 (앞 번호 이어 / 이전 번호 이어 / 새 번호 시작)
- 수준 시작 번호 스피너
- 문단 번호 모양 12종 프리셋 선택

#### 4단계: 컨텍스트 메뉴 + 단축키
- 우클릭 컨텍스트 메뉴에 "문단 번호 모양(N)... Ctrl+K,N" 추가
- Ctrl+K,N 코드 단축키 등록 (chordMapK)

## 참조 파일

| 파일 | 역할 |
|------|------|
| `src/model/style.rs` | Numbering, Bullet, HeadType, ParaShape |
| `src/renderer/layout.rs` | NumberingState |
| `src/renderer/layout/utils.rs` | expand_numbering_format |
| `src/renderer/layout/paragraph_layout.rs` | apply_paragraph_numbering |
| `rhwp-studio/src/ui/numbering-dialog.ts` | 번호 대화상자 |
| `rhwp-studio/src/core/numbering-defaults.ts` | 프리셋 |
