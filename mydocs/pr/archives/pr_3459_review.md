# PR #3459 검토 기록 — 10k 한글 오라클 서베이 r25 보고서

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3459](https://github.com/edwardkim/rhwp/pull/3459) — `docs: 10k 한글 오라클 서베이 r25` |
| 작성자·검토자 | `@planet6897` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `6a2173400e684bbfb84ff85b484de84471e13356` (`docs/survey-r25-20260728`) |
| 통합 검토 | `review/planet6897-20260727`; 적용 `6a217340…` → `c9d67544f` |
| 작성 시점 source 상태 | `MERGEABLE` / `BEHIND`, review-only fast pass 성공 |
| 라우팅 | `collaborator_external_pr` + `intake_and_review`, `multi_pr_update_branch` |

## #3458과의 결합 검토

보고서는 #3410과 **#3458 A/C/D를 합성한 10,000건 결과**로, r24와 비교해 쪽수·PI 불변 및 픽셀
개선을 기록한다. #3458 PR 본문·#3386 이슈의 사전 재게이트 기록도 “r25 10k 동승 판정 뒤 제출”을
명시하고 #3458의 검증 근거로 이 PR을 직접 연결한다. 즉 #3459는 독립 기능 문서가 아니라 #3458의
모집단 검증 보고서다. r24 보고서도 원시 10k 산출물을 저장소에 넣지 않고 `output/poc/` 경로와
`BINARY_FINGERPRINT.txt`를 참조하는 형식이므로, 원시 output 부재만으로 보류하지는 않는다.

보고서 파일·branch·실행 output·integration 식별자는 모두 **`20260728`**인데 source commit과 PR CI는
**2026-07-27**이다. 이는 실행 날짜인지 run label인지 문서만으로 판정할 수 없는 traceability 차이다.
다만 #3458의 r25 동승 관계·현재 코드의 기준 PDF/시각 증적·전체 회귀 게이트는 별도로 확인됐고, 이
label 차이는 보고된 개선 수치를 독립 재현할 수 없다는 한계로만 기록한다. 코드 수용을 막는 결함으로
분류하지 않는다.

## 최종 권고

**#3458과 함께 기술적으로 수용 가능**. 이 보고서는 #3458의 10k 모집단 근거로 같은 통합 PR에 포함한다.
향후 기준선 자료에는 실제 실행 시각 또는 run label의 의미와 `BINARY_FINGERPRINT.txt` 보관 위치를 함께
명시하도록 권고한다. 문서-only fast pass는 경로 정책의 결과이며, 수치 자체의 독립 재현 근거는 아니다.
