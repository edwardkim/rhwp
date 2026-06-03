# Task #495 단계 3 보고서 — 회귀 발견 후 진행 중단

**이슈**: #495
**브랜치**: `local/task495`
**상태**: ⚠️ **회귀 식별 — 진행 중단, 별도 승인 요청**
**전제**: 단계 2 (`task_m100_495_stage2.md`) 에서 후보 (B) text_before TextRun 발행 제거 결정

---

## 1. 진행 사항

### 1.1 코드 수정 적용

`src/renderer/layout/table_layout.rs:1612~1635` 의 TextRun 생성/push 블록 + 라인 1599~1611 의 baseline/adjacent_shape_h 계산 블록을 제거하고 inline_x 누적만 유지하도록 수정했다.

### 1.2 결함 검증 (정상)

- `samples/exam_science.hwp` 페이지 2 SVG 재생성
- cell-clip-21 (7번 박스) 영역의 baseline 분포에서 **y=224.26 중복 라인 사라짐**
- baseline 분포 (수정 전→후): 19개 → 9개. 사라진 baseline 모두 paragraph 다른 줄과 동일 텍스트의 중복 발행이었음.
- 시각 확인 (Chrome 헤드리스 PNG): 박스 안 텍스트가 정상적으로 한 번씩만 그려짐.

→ 결함 자체는 해결.

## 2. 회귀 발견 (Critical)

### 2.1 회귀 위치

`/tmp/baseline_495` (수정 전) vs `/tmp/after_495` (수정 후) 의 64 SVG 체크섬 비교 결과:

| 파일 | 차이 |
|---|---|
| `exam_science_p1/exam_science_002.svg` | 의도된 변경 (결함 수정) |
| **`synam-001/synam-001_031.svg`** | ⚠️ **회귀** |

다른 62 SVG 동일.

### 2.2 회귀 내용

`samples/synam-001.hwp` 페이지 31 의 "2 0 2 0 [년] [공백] [월] [공백] [일]" 패턴 (날짜 입력 박스):

- baseline y=1009.74 라인의 글자 분포:
  - **수정 전**: x=397.49 "년", x=460.40 "월", x=523.31 "일" — 정상
  - **수정 후**: x=523.31 "일" 만 남음. **"년", "월" 사라짐**

### 2.3 PNG 시각 비교

`/tmp/exam_p2/synam_p31_before.png` vs `/tmp/exam_p2/synam_p31_after.png`:
- 수정 전: "2 0 2 0 년 [공백 박스] 월 [공백 박스] 일" 형태로 정상 표시
- 수정 후: "2 0 2 0 [공백 박스] [공백 박스] 일" — "년", "월" 누락

## 3. 단계 2 가정의 결함

단계 2 결정의 근거였던 가정 — **"paragraph_layout 이 paragraph 의 모든 줄 텍스트를 발행한다"** 가 모든 케이스에서 성립하지 않음.

synam-001 p31 의 해당 paragraph 는 **사각형(treat_as_char) 들 사이에 짧은 텍스트("년", "월", "일") 가 끼어 있는 패턴**:

```
[사각형] [사각형] [사각형] [사각형] 년 [사각형] 월 [사각형] 일
(2)     (0)     (2)     (0)         (공백)         (공백)
```

이 케이스에서:
- `layout_composed_paragraph` 는 paragraph 의 마지막 부분("일") 만 발행하거나 paragraph 끝쪽 텍스트만 발행한 것으로 보임
- `Shape` 분기의 `text_before` (사각형 사이 텍스트 추출) 가 "년", "월" 발행의 **유일한 경로** 였음
- → 단순히 TextRun 발행을 제거하면 "년", "월" 가 그려지지 않음 (회귀)

이는 단계 2 의 분석 부족. exam_science.hwp 단일 케이스만 보고 paragraph_layout 동작을 일반화한 것이 원인.

## 4. 즉시 조치

- **코드 변경 되돌림**: `git checkout src/renderer/layout/table_layout.rs` 로 베이스라인 복원.
- **재검증**: synam-001 p31 SVG 가 베이스라인과 정확히 일치 (`diff … | wc -l == 0`) — 회귀 영향 0.
- 현재 워킹트리 src/ 변경 없음 (`git diff --stat src/` 빈 상태).

## 5. 단계 2 분석 보강 필요사항

다음 사항을 단계 2 로 되돌아가 추가 분석해야 한다:

1. **paragraph_layout 의 텍스트 발행 정확한 동작**: 어떤 조건에서 paragraph 텍스트의 일부만 발행하는가?
   - 인라인 컨트롤(treat_as_char) 가 있을 때 컨트롤 양쪽 텍스트를 모두 발행하는지, 컨트롤 마지막 이후 텍스트만 발행하는지
   - line_seg / char_offsets / utf16 인덱스 처리에서 텍스트 누락 케이스
2. **Shape 분기 text_before TextRun 발행의 정확한 역할**: 결과적으로 paragraph_layout 이 발행 못하는 텍스트를 보강하는 코드인가?
3. **수정 방향 재선택**:
   - 후보 (A) Picture 정합 — target_line 산출 + 줄별 처리. paragraph_layout 보강 역할 보존하면서 multi-line 결함 해결.
   - 후보 (C) 가드 추가 — multi-line paragraph 에서 사각형이 ls[1]+ 에 있을 때만 차단. exam_science 결함 해결, synam-001 케이스 영향 없음 (단일 줄 paragraph 라면).

## 6. 회귀 위험 보고

구현 계획서 6항 정책:

> 다만 단계마다 결과를 명확히 사용자에게 보고하고, 회귀 위험이 식별되면 그 단계에서 진행 중단 후 별도 승인 요청.

회귀가 발견되었으므로 **자동 승인 정책에서 벗어나 작업지시자 승인 대기**.

## 7. 대기 중인 결정사항

작업지시자에게 다음 중 하나를 결정 요청:

- (옵션 A) 단계 2 로 되돌아가 paragraph_layout 동작 정밀 분석 후 후보 (A) Picture 정합 방식으로 재시도 — 변경 범위 큼, 회귀 위험 재발 가능
- (옵션 B) 단계 2 로 되돌아가 후보 (C) 가드 추가 방식으로 재시도 — 변경 최소, exam_science 만 차단하고 synam-001 영향 없음 (단, 단일 줄 사각형 케이스의 시각적 중복은 잠복)
- (옵션 C) Task #495 보류 — 본 결함이 다른 layout 결함(사각형 자체 위치, paragraph_layout 텍스트 발행 누락) 과 얽혀 있어 본 task 단독 수정으로는 안전 해결 어려움. 더 큰 layout 리팩터링 필요성이 보임.

## 8. 산출물

- 보고서: `mydocs/working/task_m100_495_stage3.md` (본 문서)
- 코드 변경: 없음 (되돌림 완료)
- 베이스라인 보존: `/tmp/baseline_495/` (재검증 필요 시 사용)
