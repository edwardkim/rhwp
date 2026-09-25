---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7417 검토 — 코드 표현이 다른 글자와 저장 줄 판정

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head `4c3b16d683c7b2659a97280386de30cefa0a2ac4`는 #7415 위에 쌓였고, 현재 `devel`과 충돌한다. 고유 변경 `9271e766e`는 PUA와 옛한글 자모를 양쪽 글자열에서 무조건 지워 서로 다른 줄 컷도 같은 문자열로 판정할 수 있다. 메인터너 보정 `3d9ca69e3`은 해당 저장 첫 줄을 성공 건수로 세지 않고 `excludedUncomparable`로 드러내며, 일반 문자 문단은 그대로 비교한다. 통합 code head `7cb91df10`의 아래 검증 범위에서 수용 후보이며 원 PR head 직접 병합은 승인하지 않는다.

## 접수·출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7417](https://github.com/edwardkim/rhwp/pull/7417), `planet6897`, `devel` 대상 |
| 원 head / 고유 체리픽 | `4c3b16d683c7b2659a97280386de30cefa0a2ac4` / `9271e766e` |
| 선행 stack | #7415의 `0c2db5870827f43746445ca12b2079e52d105abf`는 `f97e7e242`에서 한 번만 적용 |
| 메인터너 보정 / code head | `3d9ca69e3`, 반례 검사 `7cb91df10` / `7cb91df10` |
| 검토 base | `97073606de2545f88540429e2f7f8312eefa9641` |
| 원격 참고 | 2026-09-25 OPEN, 원 head CI 실패·대기 없음, 최신 `devel`과 `CONFLICTING/DIRTY` |

원 HWP의 `ᄒᆞᆫ` 자모 셋과 한컴 PDF의 U+F53A처럼 글자 코드가 다른 경우에는 대응 글리프가 같다는 일반 매핑이 없다. 코드를 지우면 저장 컷이 그 글자 **뒤**인 문단과 **앞**인 출력의 key가 모두 같은 접두어가 된다. 이는 판정 도구의 거짓 양성이다. 보정은 비교 불가 문단 수를 별도로 보고하며, 해당 문단을 `rhwpReproducesStoredCut` 또는 `oracleReproducesStoredCut`에 넣지 않는다. 모든 문단이 제외되면 상태를 `미측정`으로 반환한다. 이는 문서별 예외나 글꼴 이름 추정이 아니다.

## 검증 입력·결과

- `samples/hwpspec.hwp` SHA-256 `64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d`, `pdf/hwpspec-2024.pdf` SHA-256 `9c8110347efd719c75098b97509bd4ddf4c5eb6c838d4a6d78982da5f610512b`. 두 파일은 검토 commit의 추적 자료다.
- 같은 PDF/HWP와 실제 `target/pr-review/release-test/rhwp`를 실행한 결과: 제외 **48문단**, 비교 가능한 저장 다줄 문단 **261**, rhwp 컷 재현 **219**, 한컴 **224**, 정본만 지킨 컷 **5**. 원 head의 무조건 삭제 방식은 같은 쌍에서 비교 가능한 문단을 309로 보고하면서 정본만 지킨 컷 5를 산출해, 제외 범위를 감췄다. 기존 false positive 45건을 실제 해결로 보고하지 않는다.
- `issue1853_caption_precedes_body_split.hwpx` 대조군: 제외 0, rhwp/한컴 각각 **308/349**, 정본만 지킨 컷 0. 입력·PDF 해시는 [#7415 검토](pr_7415_review.md)에 기록했다.
- `python3 -m unittest scripts/tests/test_oracle_comparability.py`: **10/10 PASS**. 글자를 지우면 다른 저장 컷이 같아지는 합성 반례는 제외 1·`미측정`, 일반 문자 대조군은 제외 0이다. 실제 두 문서와 단위 검사 이외 290쌍 전수·원격 통합 CI는 미실행이다.

## 조판 원칙 및 시각 증거

도구의 글자열 key·판정만 변경한다. renderer 조판·paint·Visual Sweep 적용은 **비해당**이며, 실제 한컴 PDF는 기준 입력으로 사용했지 rhwp 시각 일치 주장으로 사용하지 않는다. `#7396` 전체 이슈 해결은 이 PR 범위 밖이다.

## Merge 후 contributor PR comment 계획

통합 PR merge가 실제 확인되면 merge SHA·최신 CI 링크와 함께 기여자의 PUA/옛한글 비대칭 발견을 인정하고, 일반 매핑 없이 삭제하면 컷이 합쳐지는 반례 때문에 메인터너가 제외 건수를 드러낸 이유를 한국어 존댓말로 설명한다. 기여자 원 branch는 보존하며 `#7396`은 닫지 않는다. 게시 시 `--body-file`과 API 재확인을 사용한다. 현재 comment·close·push·merge는 하지 않았다.
