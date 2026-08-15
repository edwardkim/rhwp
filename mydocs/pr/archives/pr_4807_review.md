---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4807 검토 - 재현 가능한 Gym 능력 인증서

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4807](https://github.com/edwardkim/rhwp/pull/4807) · @kevin9327 |
| 원 head | `aca00299a5d1216577135e05fe8284cb024ab2ec` |
| 기준선 | `upstream/devel@bcb65ed68` |
| 누적 적용 | `aca00299a` → `849766e0c` |
| 메인터너 보정 | `f674ac7c5` |
| 원 CI | 작성 시점 참고값: CI·CodeQL 성공, mergeable `MERGEABLE` |

## 변경과 판단

`gym/certify.py`가 점수, 실행 바이너리 식별자, benchmark 지문을 인증서에 기록하고 같은 입력으로
재현 대조한다. Rust 구현과 렌더 산출물은 바꾸지 않으므로 시각 검증은 대상이 아니다.

원 지문은 `pack.json`, task, reference만 포함했다. 그러나 pack asset, checker, 기준풀이 조립기,
score/report/coverage 프로토콜도 점수를 바꿀 수 있으므로 이들을 제외하면 같은 지문으로 다른 측정을
인증할 수 있다. 메인터너 보정은 `packs/`, `core/`, `profiles/`, `tools/`, score/report/certify 파일을
결정론적으로 해시하고 asset·프로토콜 변경 회귀를 추가했다. 기존 인증서는 의도적으로 새 지문과
불일치하므로 재발급 대상이다.

## 완료 검증

- `python3 -m unittest ... test_gym_certify.py ... test_workflow_contract_wiring.py`: 총 89건 통과, 의도된 1건 skip.
- `python3 gym/certify.py --bin target/pr-review/release-test/rhwp --out /tmp/rhwp-gym-capability-certificate-20260815.json --at 2026-08-15T00:00:00Z`: 정확도 `236/236`, 100% 인증서 발급.
- 같은 인증서의 `--verify` 재현 경로를 완료해 동일 기준선 대조를 수행했다.
- `git diff --check`: 통과.

**로컬 보정 후 수용 후보.** merge 직전에는 원 PR 최신 head와 required check를 다시 확인한다.
