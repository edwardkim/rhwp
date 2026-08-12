# #4467 Stage 3 완료 — 조판과 협업의 병행 진행 반영

- **Issue**: [#4467](https://github.com/edwardkim/rhwp/issues/4467)
- **브랜치**: `task/4467-project-roadmap`
- **피드백**: [조판과 협업의 병행 진행](../feedback/task_m100_4467_roadmap_parallel_progress.md)
- **완료일**: 2026-08-10 KST

## 1. 작업지시자 판정

초안은 v1.0 조판을 현재 단계, v2.0 협업을 다음 단계로 두어 두 작업이 순서대로 시작되는 것처럼
읽혔다. 실제로는 v1.0 이전에 이미 40명이 넘는 외부 기여자와 두 명의 콜레보레이터가 참여하며,
기여·검토·통합을 나누는 협업 체계가 운영되고 있다. 현재 상태는 조판과 협업의 병행 진행이다.

## 2. 현재 상태 근거

2026-08-10에 GitHub API로 현재 저장소를 다시 확인했다.

```text
공개 contributor 이력
- 메인테이너 edwardkim, Dependabot, 익명 항목 제외
- 이름 있는 외부 기여자 41명

직접 권한 목록
- edwardkim: admin
- postmelee: write
- jangster77: write
```

공개 문서에는 숫자의 변동 가능성을 고려해 “40명이 넘는 외부 기여자와 두 명의 콜레보레이터”로
표현했다.

## 3. 로드맵 보정

- 버전 번호를 작업 착수의 엄격한 순서가 아니라 각 목표의 성숙도와 완료 기준으로 다시 설명했다.
- 현재 상태표에서 v1.0은 `집중 진행 중`, v2.0은 `함께 진행 중`으로 표시했다.
- v2.0 제목을 `다음 단계`에서 `조판과 함께 진행 중`으로 바꿨다.
- 조판·저장 품질과 외부 기여가 서로 피드백을 주며 함께 발전한다는 관계를 명시했다.
- v2.0의 남은 조건을 지속 가능한 역할 분담, 확장 호환성, 충돌·실패 복구와 유지 책임으로 정리했다.
- README의 한국어·영문 공개 요약도 같은 현재 상태를 가리키도록 고쳤다.

## 4. 검증

현재 로컬 기준선 `upstream/devel` `629cd33db`에서 다음 검사를 통과했다.

```text
python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references
검사 문서: 534개 / 변경 파일: 14개 / redirect stub: 30개
내부 Markdown 상대 링크: 이상 없음

python3 scripts/check_document_metadata.py
메타데이터 검사 문서: 521개
문서 메타데이터: 이상 없음

git diff --check
통과

ROADMAP.md와 README.md의 순차 단계 오인 표현·공개 문체 금지어 검색
이상 없음
```

GitHub의 최신 `devel`은 `c20377b9e`로 로컬 기준선보다 201개 커밋 앞서 있다. 작업 브랜치를 최신
기준선에 다시 정렬한 뒤 같은 검사를 최종 실행해야 한다.
