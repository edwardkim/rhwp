---
kind: guide
status: active
canonical: mydocs/manual/pr_review/intake_and_review.md
last_verified: 2026-10-09
---

# PR #N 리뷰 — 변경 요약

## 최종 판정

**머지 보류 — 검증 진행 중.** 완료되지 않은 검증과 해제 조건을 한 문장으로 쓴다.
검증이 끝나면 [공통 판정 용어](../pr_review_workflow.md#11-최종-판정-용어와-원격-조치의-분리)의
`승인`, `머지 보류`, `메인터너 보정 후 수용 가능` 중 정확히 하나로 고친다. 원 head와 보정
head를 혼동하지 않는다. 최신 head CI와 mergeability 같은 merge 전 조건은 판정 아래에 구분한다.

collaborator self PR은 [8.2.1절](collaborator_self_merge.md#821-pr-채번과-오늘할일-생성갱신-시점)에 따라
PR 채번과 code head CI 성공 뒤 이 review와 오늘할일을 문서-only trailing commit으로 기록한다.
검증한 code SHA·CI run URL·결과와, 아직 확인 전인 trailing head의 CI·merge 조건을 구분한다.

메인터너 보정의 모든 필수 로컬 검증이 완료되면 첫 문장은
`메인터너 보정 후 수용 가능 — #이슈번호 해결 범위의 로컬 검증 완료.`로 통일한다.
이슈가 없으면 `#이슈번호 해결 범위`를 실제 검증 범위로 바꾼다.
미실행·실패·필수 증거 부족이 남아 있으면 이 완료 문구를 쓰지 않는다.
이어지는 문장에는 해결 범위와 남은 차이를 적고, 원격 CI·push·merge 완료 여부는
별도로 기록한다. 형식의 예시는 [PR #7491 최종 판정](../../pr/archives/pr_7491_review.md#최종-판정)이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | #N / 작성자 / devel |
| 원 head·검토 후보 | 두 SHA와 역할 |
| 관련 이슈 | 링크와 닫기 또는 참조 여부 |
| reviewer·작성 시점 상태 | reviewer, draft·mergeability·CI 참고값 |

## 변경과 검토 범위

핵심 동작, 영향받는 입력·호출 경로, 범위 밖 변경과 의존 PR을 적는다.

## 검증 입력과 결과

실제 사용한 HWP/HWPX·한컴 기준 PDF의 저장소 경로, 출처·SHA-256·검토 commit 포함
여부를 적는다. 실행한 명령과 대상 head·통과/실패/미실행 범위를 구분한다. 조판 원칙,
focused·전체 회귀·lint·Native/WASM·시각 검증 중 적용되는 것을 기록한다.

렌더링 회귀를 추가·변경했다면 관련 모든 페이지의 Native/fresh WASM 최저 일치율 90% 이상을
먼저 확인한 source SHA·TSV·직접 판독 증거를 연결한다. 실패한 기존 회귀는 원본 출력 보정 후
90% 이상 확인을 거쳐 필요한 검사만 변경했는지, 절대 px 좌표·고정 px 영역 대신 문단·줄·개체의
소유·순서·보존·포함·겹침 관계를 검사하는지 기록한다. 미달·미검증이면 완료 문구를 쓰지 않는다.

## 시각 증적과 남은 차이

입력·기준 PDF·쪽 대응, Native/fresh WASM review와 overlay의 안정 경로, 실제 지표와
사람이 직접 판독한 결론을 쓴다. 예외는 정본의 글꼴 증거 요건에 맞춰 별도로 증명한다.

90% 이상이어도 변경 영역·알려진 차이·사용자 지적 영역을 같은 배율로 확대해 수식의 모양·장평·
기준선·첨자·장식과 관계 기호 양쪽·본문·보기 번호·탭·줄 사이 간격을 독립 Print PDF와 대조한다.
Native/fresh WASM의 내용 식별자·확대 증거·원인·처리 결과를 기존 증적에 연결하고 점수 통과와
직접 판독을 구분한다. 미해결 결함이나 미검증이 있으면 해당 범위를 완료로 쓰지 않는다.

## Merge 후 contributor PR comment 계획

시각 검증을 판정에 사용했다면 최종 merge SHA와 CI URL, Visual Sweep 정본 링크,
merge SHA 고정 raw image URL, 남은 차이, `--body-file` 게시와 API 재조회를 계획한다.
기여자에게 게시할 문안은 한국어 존댓말로 준비한다. 메인터너 보정이 있으면 원 기여가 해결한
문제와 추가 보정이 필요했던 이유·범위를 구분해 설명하고 반말이나 책임 전가 표현을 쓰지 않는다.

이 파일은 작성 예시다. 각 review에 해당하지 않는 절은 이유를 적고, 실제 검증 전에는
예시 문장을 완료 사실로 옮기지 않는다. 필수 기록과 실행 순서는
[PR 접수와 리뷰 기록](intake_and_review.md)이 정한다.
