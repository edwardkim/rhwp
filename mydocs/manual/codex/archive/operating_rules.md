---
kind: snapshot
status: historical
canonical: mydocs/manual/codex/README.md
last_verified: 2026-07-17
---

# Operating Rules

## Hyper-Waterfall Is Mandatory

이 프로젝트는 하이퍼-워터폴 방법론을 적용한다. Codex의 기본 성향인 빠른 실행, 임의 구현, 탐색 중 패치를 억제해야 한다.

필수 순서:

1. 이슈 확인
2. 브랜치 확인 또는 생성
3. 오늘할일 문서 갱신
4. 기존 문서와 트러블슈팅 검색
5. 계획서 또는 분석 문서 작성
6. 작업지시자 승인
7. 구현
8. 테스트와 검증
9. 보고서 작성
10. 커밋

빠르게 고칠 수 있어 보여도 이 순서를 건너뛰지 않는다.

## No Implementation Before Approval

특히 HWP 직렬화, 한컴 호환, 파일 손상, Task #854 같은 고위험 작업에서는 구현부터 시작하지 않는다.

먼저 해야 할 일:

- 관련 기존 문서 검색
- 정답 파일과 POC 결과 비교
- 차이점 목록화
- 구현 후보와 위험 정리
- 작업지시자 승인 요청

## User Controls Scope

작업 범위와 중단 여부는 작업지시자가 결정한다.

금지:

- 임의로 시간 제한을 제안
- "여기서 멈추자"는 식의 종료 제안
- 미해결 이슈를 임의로 닫기
- 사용자가 지운다고 한 변경을 보존하려고 설득
- 구현 방향을 사용자 승인 없이 확정

## Search Troubleshooting First

HWP 저장, 한컴 호환, 파일 손상, 직렬화 관련 작업을 시작하면 반드시 먼저 `mydocs/troubleshootings/`를 검색한다.

우선 검색 키워드 예:

- `손상`
- `한컴`
- `저장`
- `직렬화`
- `export_hwp`
- `LIST_HEADER`
- `PARA_TEXT`
- `control_mask`
- `char_count`
- `PAGE_BORDER_FILL`

관련 문서가 하나라도 나오면 읽고 나서 다음 단계를 진행한다.

## Do Not Over-Generalize Hancom Compatibility

한컴 호환 문제는 일반화가 위험하다. 구조별 명시 가드가 일반 알고리즘보다 안전할 수 있다.

예:

- `has_nested_table` 같은 구조 기반 가드 선호
- 측정값 기반 clamp는 회귀 위험이 크므로 신중히 사용
- 하나의 샘플에서 맞는 규칙을 전체 문서에 확장하지 않는다

## Communication Style

사용자가 불만을 표현하면 방어하지 말고 문제 설정을 정정한다.

Task #854에서 사용자가 지적한 핵심:

- POC에서 되던 것이 본 구현에서 깨지는 점이 가장 큰 문제다.
- 엉뚱한 곳을 고치고 있다는 의심은 합리적이다.
- 지금은 파서나 렌더러가 아니라 저장 materialization 문제를 봐야 한다.

이 지적을 이후 작업의 기준으로 삼는다.
