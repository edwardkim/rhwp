# PR #7283 self-review — 선택 실행 확장 E2E

## 최종 판정

**승인 — 코드 검토 판정.** 최종 code `a2df40172`의 로컬 계약과 실제 Chrome 정상/실패 대조를 통과했다.
merge 전 최신 head의 원격 checks와 메인테이너의 최종 판단을 확인해야 한다.
이 기록은 GitHub 승인·merge를 수행하지 않는다.

## 접수와 범위

- base route: collaborator_self_merge.
- modifiers: intake_and_review, local_validation, rework_and_exceptions.
- loaded documents: pr_review_workflow.md, pr_review/README.md 및 위 자식 가이드.
- 메인테이너 [범위 축소 동의](https://github.com/edwardkim/rhwp/pull/7283#issuecomment-5844951296), 사용자 갱신 지시.
- 비교 base: `443844b593c62a722cf9cc3d9d0256e94ab88cb8`; 구현 전 merge simulation 충돌 없음.
- author self-review이며 reviewer 지정·merge·comment·이슈 close는 수행하지 않는다.

## 구현과 검증 대조

| 주장 | 실제 경로/검증 | 판정 |
| --- | --- | --- |
| 자동 Chrome CI 연결 제거 | common base `eb9142dd7` 대비 `.github/`, `scripts/` diff 없음 | 충족 |
| 명시적 배포 후보만 검사 | `run.mjs`의 --dist → 모든 child env → 세 suite의 DIST 경로 | 충족 — 외부 후보 정상 통과, print.html 누락 대조 실패 |
| 실패를 통과로 표시하지 않음 | fail/timeout/spawn error, 미실행 not-run, 부분 선택 거부 계약 | 충족 |
| 후보·결과 식별 | 파일 inventory hash, 전후 변경 검사, output 충돌 및 symlink 경계 검사 | 충족 |
| 조판/렌더링 원칙 | 제품 소스·기준값 변경 및 시각 정확성 주장 없음 | 비해당 |

Node 177개·Python 130개 및 최종 영향 계약 19개 통과. 실제 Chrome 세 suite 통과, lifecycle 10개 retry 0.
검증 입력 Git blob 일치: 충족. 실제 Chrome 결과와 후보/fixture 해시는
[결과보고](../../report/task_m100_3512_report.md)에 기록한다.

## 이전 검토

기존 CI 중심 설계의 판정·P2 보완·실행 시간 기록은
[이전 검토 문서](https://github.com/edwardkim/rhwp/blob/0b3da1cbd2ca6881bd760303312966f03cfabbbe/mydocs/pr/archives/pr_7283_review.md)에 보존한다.
그 판정은 현재 선택 실행 도구에 대한 승인으로 이어지지 않는다.
