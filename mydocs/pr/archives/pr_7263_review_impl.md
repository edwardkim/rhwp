---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7263 통합·후속 처리 순서

[검토 결과](pr_7263_review.md)에 연결된 실행 기록이다. 이미 완료한 코드 보정과 검증을
향후 계획으로 바꾸지 않는다. rollback은 아래 통합 commit 범위를 기준으로 판단하며
정상 원본 파일 교체를 과거 잘못된 입력으로 자동 복구하지 않는다.

## 완료한 commit 순서

- `c00f54b60` — 수정(wmf): fuzz 가 찾은 정수 넘침·범위 밖 인덱스 패닉을 막는다
- `36235b8ad` — 수정: 문단 시작의 단 나눔도 문단을 가르지 않고 그 문단에 건다 (#5019)
- `0bcb4fa5a` — test: 단 나눔 통합 계약 정렬 및 PR 7239·7240 보류 근거 기록
- `7521c08c8` — fix: WMF TextOut 세로 정렬의 누락된 오버플로 방어 보완
- `bf8070c27` — fix(renderer): 표 뒤 저장 줄을 실제 표 흐름에 연결
- `fc668664a` — test: 셀 문단 소유권으로 분할 텍스트 런 검증
- `0e3513b66` — test: 성장하지 않은 표 뒤 저장 간격의 독립 기대값 검증
- `bb45be16b` — fix: 표 뒤 텍스트가 문단 뒤 간격을 중복 소비하지 않도록 수정
- `1e3aed178` — test: 표 뒤 글줄 회귀를 HWPX 기준 파일 하나로 통합
- `403c4ef7f` — fix(renderer): close TAC paragraph spacing after every owned item
- `baa29e96e` — test(renderer): normalize paragraph spacing fixture units
- `df1b1de7f` — fix: CLI 단 나눔 속성과 사용자 분할 명령을 분리하고 저장 축 보존
- `75a484886` — 수정: scaffold 표 셀 선언 높이를 한 줄 행 높이로 적는다 (#7234)
- `e2cb5cdc8` — docs: PR 7239·7240 보정 검증과 7242·7243 개별 시각 검토 기록
- `fc5df351b` — test: PR 7242 한컴 재저장 대조군과 PDF 기준선 검증 보완
- `086078148` — fix(renderer): 정상 규제영향분석서와 저장 표 회귀 복구
- `c877f6e48` — test: #7242 공개 검증 입력과 저장 줄 계약 정상화
- `bb401f0a7` — fix: #7242 표 줄과 문단 테두리 소유 범위 보정
- `eed2a223e` — docs: PR 7239·7240·7242·7243 최신 검토 판정 정리
- `43a0fbeec` — fix: 중첩 표 이어받기의 실제 점유 높이 예약
- `334950625` — docs: PR 7243 현재 승인과 과거 보류 기록 구분

## 진행 단계와 다음 조건

1. 분석 → 코드 수정·검증 → 결과보고 → commit을 반복해 메인터너 보정을 완료했다.
2. 사용자 PR 지시에 따라 upstream 작업 branch에 push하고 Open PR #7263을 생성했다.
3. 통합 review·이 실행 기록·2026-09-18 오늘할일을 문서 전용 trailing commit으로 추가한다.
   최신 base와의 merge simulation·문서 링크·기존 기록 보존을 확인한 뒤 push한다.
4. 최신 head required CI와 mergeability를 확인한다. 현재 생성 단계이며 CI 완료·merge를 주장하지 않는다.
5. merge 승인 후 실제 SHA 확정 → devel asset 확인 → 원 PR별 한국어 증적 comment 및 상태 처리 →
   issue 범위 재확인 순으로 처리한다. #7240은 원격 갱신 head와 실제 통합 범위를 구분해 설명한다.
   #7234는 전체 범위 확인 전 종료하지 않는다.
6. post_merge 정본을 적용해 duration refresh만 확인하고 validation CI를 별도로 재실행하지 않는다.
   devel 동기화와 소유한 전용 branch/worktree/target 정리는 활성 작업·최종 SHA·소유권을 확인한 뒤 한다.

현재 승인된 원격 작업은 PR 제출이다. merge와 이후 외부 comment/close는 해당 사용자 지시를 따른다.
