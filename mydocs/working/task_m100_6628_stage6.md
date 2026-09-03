---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage6.md
issue: 6628
last_verified: 2026-09-03
---

# #6628 Stage 6 — 제출 범위와 최신 devel 감사

## 1. 판정

#6628의 핵심 Gym 정상화 구현은 최신 `upstream/devel@edeaeb28910f1b84f005aabb4ec0d0f183adc2a1`
위에서 제출 준비가 가능하다. 부모 브랜치 `task_m100_6628`은 merge commit
`3950ca15738311ef23e87233d981dd2d6197b953`으로 devel을 받아들였고 충돌은 없었다.

다만 이 제출은 부모 이슈의 즉시 종료를 뜻하지 않는다. JSON 증적 시각화 sub-issue #6669를
이 구현이 devel에 반영된 뒤 독립 브랜치에서 처리하고, 그 결과까지 포함해 #6628 완료 조건을
다시 감사한다.

## 2. 제출 범위

변경은 다음 네 축이다.

1. task/reference/check의 약한 answer·artifact oracle과 stale 명령을 정상화한다.
2. 1,035 task의 authority·baseline source를 누락 없이 분류하는 결정론적 원장을 추가한다.
3. portable Python 계약과 positive/discrimination/trajectory의 fail-closed 봉투를 고정한다.
4. Gym을 제품 릴리스 게이트에서 분리하고, Gym 관련 PR의 빠른 계약과 메인테이너 수동 전수
   실행으로 운영 경계를 바꾼다.

최종 diff에는 #6641의 Rust 제품 수정을 중복 포함하지 않는다. 그 변경은 최신 devel에 이미
병합된 기반이다. 사설 코퍼스, 비밀값, 생성 제출물과 임시 전수 산출물도 포함하지 않는다.

## 3. Stage 4 전수 증적 인계

#6641은 #6628 runner와 제품 후보를 분리한 상태로 세 축을 한 번씩 실행했다.

| 축 | 결과 | 판정 |
| --- | --- | --- |
| positive | 21 pack · 1,035/1,035 · 실패/skip 0 | 통과 |
| discrimination | 1,035 task · 1,511 control · false-pass 0 | 통과 |
| trajectory | 239/239 load-bearing · theater/예외/tool 오류 0 | 통과 |

- Gym runner: `374c7416a6c2d9abe7c2701969de5f377b71183f`
- product candidate: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- binary SHA-256:
  `4334e35e3bfb7e892416e663c7a52dd055a7d82040d2a89447f32d25ccd02f34`

현재 병합 tree와의 관련 실행 의미 diff가 0임을 확인했으므로, 동일한 38분 전수 실행을 다시
수행하지 않고 이 증적을 재사용한다. 구체적인 실행 시간·오류 정산은
[`#6641 최종 보고서`](../report/task_m100_6641_report.md#5-6628-gym-인계-전수)를 따른다.

## 4. 현재 병합 tree 재검증

| 검사 | 결과 |
| --- | --- |
| `python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'` | 3,149건 통과 · skip 1 |
| `python3 gym/tools/audit.py --json` | 21 pack · 1,035 task · issue 0 · exit 0 |
| `python3 gym/tools/oracle_probe.py --json` | structural issue 0 · exit 0 |
| `python3 gym/tools/oracle_probe.py --selftest --json` | 14/14 · issue 0 · exit 0 |
| `python3 gym/tools/authority_ledger.py --json` | 1,035/1,035/1,035 · issue 0 · exit 0 |
| `python3 -m unittest scripts.tests.test_gym_benchmark_validation` | 8건 통과 |
| CI impact classifier/policy Node 계약 | 78건 통과 |
| 변경 문서 링크·redirect 검사 | 633개 문서 · 이상 없음 |
| `git diff --check upstream/devel...HEAD` | 통과 |

authority 분포는 `self-live` 987, `contract-constant` 28, `independent-fixture` 20,
`external-oracle` 0이다. baseline source는 `self-live` 1,031, `contract-constant` 4다.

문서 전수 metadata 검사는 이 변경과 무관하고 upstream/devel에도 존재하는 기존 문서 네 개의
front matter 누락 16건을 보고했다. #6628 변경 문서의 metadata 오류는 아니다. 범위를 넓혀
그 기존 문서를 함께 고치지 않는다.

## 5. 보호 불변식 대사

- 제품 릴리스 비의존: devel/main/tag trigger와 제품 게시 artifact producer/consumer가 없다.
- 정답 권위 정직성: `self-live`를 external oracle 또는 제품 정확성으로 승격하지 않는다.
- 양·음·경로 분리: 세 결과를 별도 봉투와 별도 판정으로 유지한다.
- 오류 봉투 불통과: error, missing, skip, untrusted, theater를 녹색으로 접지 않는다.
- 제품 수정 분리: Rust 변경은 #6641로 병합됐고 #6628 diff에 중복되지 않는다.
- 사설 자료 비의존: 공개 tracked Gym 자산만 사용한다.

## 6. 제출 뒤 순서

1. 이 보고서와 최종 diff를 승인받아 커밋한다.
2. 별도 승인 뒤 원격 branch push와 devel 대상 PR을 생성한다.
3. CI 성공과 self-review 뒤 정상 merge하되 #6628은 OPEN으로 유지한다.
4. #6669를 최신 devel 기준 새 작업 브랜치에서 수행한다.
5. #6669 병합·종료 뒤 #6628 전체 완료 조건과 후속 백로그를 최종 정산한다.
