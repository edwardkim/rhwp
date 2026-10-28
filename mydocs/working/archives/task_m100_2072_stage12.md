# Task M100 #2072 Stage 12 - tech 루트 잔여 조사 문서 분류

## 목표

`mydocs/tech` 루트에 남은 문서를 내용 기준으로 다시 감사해 특정 이슈의 가설·실험·관찰은
`investigations/issue-####/`, 확정 원인과 대응은 `troubleshootings/`, 대체된 계획은 `archive/`로
분리한다. 장기 스펙·아키텍처·결정 문서는 루트에 유지한다.

## 분류 기준

- 특정 이슈·샘플의 재현, 원인 가설, 실측 차이, 미확정 후보를 중심으로 하면 `investigation`이다.
- 재현 가능한 증상과 확정 원인·적용 가능한 대응을 함께 제공하면 `troubleshooting`이다.
- 이슈 채번 전 전략 또는 완료된 구현 계획으로 현재 기술 사실의 근거가 아니면 `archive`다.
- 승인된 장기 계약, 포맷 reference, 아키텍처 결정은 이슈 번호가 있어도 루트에 유지한다.

## 이동 범위

- 이슈 조사: #257, #310, #511, #516, #1151, #1224, #1236, #1238, #1239, #1246,
  #1248, #1251, #1414, #1589, #1600, #2004 계열 20개 문서
- 트러블슈팅: #2269 PDF 폰트 리소스 채번 비결정 1개 문서
- archive: all-in-one-parser 사전 전략, 문단 부호 구현 계획 2개 문서

`hwp_ole_chart_renderer_architecture_decision_1251.md`, `ci_cache_policy_1664.md`처럼 장기 결정의
근거를 보존하는 문서는 조사 문서와 함께 일괄 이동하지 않는다.

## 외부 이력 호환

각 기존 파일명을 GitHub issue/PR 본문·코멘트에서 검색했다. 검색 결과가 1건 이상인 경로는 기존
위치에 새 문서로만 연결하는 redirect를 남기고, 0건인 경로는 내부 링크만 갱신한다. 새 내부 참조는
Stage 11의 전체 `mydocs` 금지 경로 검사로 거부한다.

## 검증 계획

- 기본 상대 링크 검사
- Stage 4~12의 이전 경로 금지 검사
- `mydocs/tech` 루트 잔여 문서의 역할 재감사
- `actionlint .github/workflows/docs-link-check.yml`
- `git diff --check`

## 결과

- 23개 문서를 내용 기준으로 분류했다. 이슈별 가설·실측 20개는 `investigations`, 확정 원인과 대응
  1개는 `troubleshootings`, 대체된 전략·계획 2개는 `archive`로 이동했다.
- GitHub issue/PR 본문·코멘트 검색 결과가 1건 이상인 이전 경로 17개에만 redirect를 남겼다.
- 이동 뒤 `tech` 루트는 실문서 58개와 redirect 19개로 구성된다. 특정 이슈의 조사 보고서·가설 검증·
  Stage 인벤토리로 판정된 실문서는 루트에 남지 않았다.
- 기본 상대 링크 검사는 308개 문서에서 통과했다.
- Stage 4~12의 이전 경로 63개를 전체 `mydocs` 7,102개에서 검사해 신규 참조가 없음을 확인했다.
- `python3 -m py_compile`, `actionlint`, `git diff --check`가 통과했다.
