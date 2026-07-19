---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 작업 스타일 + 협업 패턴
description: 하이퍼-워터폴, 시각 판정 게이트, 광범위 sweep, 외부 PR 흡수 패턴
type: user
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
## 강한 특징 (절대 위반 금지)

### 하이퍼-워터폴 + 외부 PR 처리
- 외부 PR도 cherry-pick 패턴: author 보존, 단계 commit 보존, 본 환경 정합 commit 별도 분리
- PR base skew 발견 시 즉시 close (단순 머지 = 사실상 revert 위험)
- fork base 동기화 권장 명시

### 시각 판정 게이트 + 정량 측정
- 시각 판정(한컴 2022 / 브라우저)은 결정적 게이트
- 정량 측정 결합: glyph 출현 횟수, byte-identical sweep 등
- 예: PR #587 — `>-</text>` 출현 횟수로 PR 본문 표 100% 검증

### 본질 추적 의지
- "본 환경 자체 증거" 검증 — 테스트 주석/코드가 본질을 가리키는지 우선 확인
- 외부 PR 검토 시 본 환경 코드의 자기-증거 점검 일관 적용

### 회귀 위험 영역 좁히기 (정량화)
- 광범위 sweep: "5 샘플 58 페이지 38 differ", "13 fixture 481 페이지 byte-identical"
- 의도된 변경과 비의도 회귀를 정량으로 분리

## 중간 특징

### 외부 기여 흡수 — 옵션 분류 체계
- 옵션 A/B/C/D + A2/A3/B1 세분화
- "rhwp 첫 PR" 의식 — 첫 기여자 환영 + 일관 처리

### 협업 PR 권유 (vs 경쟁 cherry-pick)
- 같은 이슈 두 PR → "양 PR 동시 댓글로 협업 PR 권유" 새 시도

### 처리 보고서 패턴
- `output/svg/pr{N}_before/{샘플}/` + `output/svg/pr{N}_after/{샘플}/` 패턴
- before/after SVG 자료 패턴 정합 유지

### 단기 폭주 사이클
- 4/19 v0.2.1 → 4/26 v0.7.6 → 4/27 v0.7.7 → 4/29 v0.7.8 → 5/1 v0.7.9
- 5/1~5/5 사이 commit 171건, 외부 PR 11건 처리

## 약한 특징 (주의)

### Git 정합 점검 강화
- 폴더 이동 untracked 발견 시 즉시 정정
- working tree 청결도 일관 유지

### 메모리 / 페르소나 덤프 동기화 의식
- Claude 메모리 = 외부화된 작업지시자 인지 자료
- 일자별 페르소나 덤프로 변천사 보존
- 메모리 갱신과 `mydocs/manual/memory/` 백업 사이 drift 점검 의무

### 언어 선택 + 자기검열
- 한국어 우선 + 영어 병행
- "산수" 표현 금지, "계산" 사용
- 외부 공개 시 자기검열 (비교/최상급/공공기관 오인 표현 회피)
