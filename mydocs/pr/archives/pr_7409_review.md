---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7409 검토 — PDF 내용 배율 진단

## 최종 판정

**메인터너 보정 후 수용 가능.** #7397 위에 쌓인 원 head의 고유 변경은 내용 배율을 x 위치·글꼴 크기 비율로 진단한다. 원 head는 y 기울기가 계산되지 않으면 x 값이 있어도 `미측정`으로 잘못 반환했다. 공유 보정 `a6b27fb9dbc58c7f5de141011365933f7eb814d0`에서 x 기울기만 필수로 바꾸고 반례·대조군을 추가했다. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 도구 회귀 7건과 실제 API 문서 진단이 통과했다. 원 PR head 단독은 #7397과 보정에 의존해 승인하지 않는다.

## 접수와 체리픽 중복 제거

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7409](https://github.com/edwardkim/rhwp/pull/7409), `planet6897`, `devel` 대상 |
| 원 head / 고유 `-x` 체리픽 | `f1d205a481dc27a6b05a089844a0f81992b07366` / `35623acaf3e5c3bd8f23894521cfbf7eacf8b7a4` |
| 선행 #7397 | `e4e3392eac32f903d520805d1226e52837b4f958`에서 이미 적용; 중복 적용하지 않음 |
| 공유 메인터너 보정 | `a6b27fb9dbc58c7f5de141011365933f7eb814d0` |
| 검토 base / code head | `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` / `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88` |
| 원격 참고값 (2026-09-25) | OPEN, MERGEABLE/CLEAN, Draft 아님; 1파일, +519/−0 |

[보정·통합 순서](pr_7409_review_impl.md)와 [#7397 선행 검토](pr_7397_review.md)를 함께 본다. 원 head CI는 중복 제거된 누적 head의 CI가 아니다.

## 결함, 기대값, 적용 경계

`scripts/oracle_comparability.py::judge_content_scale`은 줄의 x 원점 기울기와 글꼴 크기 비율을 비교한다. y 기울기는 참고값이다. 원 head의 `None` 조기 반환은 같은 y에 놓인 고유 8줄의 x와 글꼴 크기가 모두 0.8배여도 `미측정`으로 만들었다. 보정의 제어 입력은 이 경우 `내용 배율`을 판정하고, x 분산 자체가 없는 대조군만 `미측정`으로 남긴다. 이 제어 입력 경계는 **충족**이다. 실제 0.8배 한컴 문서의 전 범위 분류는 **미검증**이다. renderer의 조판·paint 변경은 **비해당**이다.

## 검증과 실제 문서

- `python3 -m unittest scripts.tests.test_oracle_comparability -q`: **7/7 PASS**, 통합 Rust release-test **10,229/10,229 PASS·50 skip**와 필수 lint PASS.
- 실제 `samples/hwpctl_API_v2.4.hwp` (SHA-256 `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603`)와 `pdf/hwpctl_API_v2.4-2022.pdf` (SHA-256 `a0141ca188ca638b305d896e21749703a39529f7bd8cf2bc806dea4ff9aaac66`)의 49·60쪽에서 `contentScale=제자리`, xSlope 1.0035, ySlope 0.9995, 대응 30줄이었다. 선언 face 11개 중 PDF 대체 10개로 최종 `비교가능_주의`, `storedLines=미측정`이다. 이 실행은 정상 배율 문서의 진단으로, 실제 0.8배 입력을 검증했다는 뜻은 아니다.
- 원시 JSON·테스트 로그는 Git에서 제외한 `output/pr-review/planet6897-20260924/diagnostics/`에 있다. 도구 전용 변경에 renderer Visual Sweep은 비해당이며, 다른 PR의 통과 PNG를 #7409의 증거로 사용하지 않는다.

통합 PR의 원격 CI·mergeability는 생성 후 exact head에서 재확인한다.

## Merge 후 contributor PR comment 계획

실제 통합 merge 뒤 merge SHA·CI URL, #7397 선행 commit을 한 번만 적용한 계보, x 기울기 경계 보정과 실제 `비교가능_주의`의 한계를 한국어로 알린다. 현재 comment·approve·push·merge는 수행하지 않았다.
