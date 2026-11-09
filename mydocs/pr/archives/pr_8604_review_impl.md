# PR #8604 통합 적용·메인터너 보정 기록

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8604_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8604](https://github.com/edwardkim/rhwp/pull/8604)의 원 head는 `013459bff4ff5313af53ece5646e79c48647a3aa`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `032dcb80e3de44b3ed4a7ca09549a197b6a18067` | `5f1b279d8b13c39e493cd98009f0783cf4f26aa0` | fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다 |
| `e74368c345437ca63591ea064c4b972ade999ee7` | `06c1a44d1c4e7f6f289282432baaa4fec182c828` | docs(#8596): 필수 편람 Native 383쪽 TSV — devel 7fdd5441ff 와 바이트 동일 |
| `013459bff4ff5313af53ece5646e79c48647a3aa` | `1be7113c514713d9d749b51ade4e9756670364ae` | docs(#8596): 최신 devel rebase head 재측정 — base·head 전쪽 TSV, 대표 PNG, 편람 gate |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.

## Native 표시 family 변경 시 체크 기호의 원본 폭

R36의 원본 전 14쪽은 Native 최저 96.52%, fresh WASM 최저 98.44%다. 그러나 실제
11쪽 연구 방식·예산 항목의 U+221A `√`를 확대하면 Native의 기호와 뒤 항목이 좁아진다.
같은 쪽 raw는 Native 97.59%, WASM 99.96%이며, 점수로 이 차이를 면제하지 않는다.

선행 Git의 `6c039b2006`·`b277a89737`은 HFT의 고유 전진과 꾸밈을 분리했고,
`6097253468`(PR #8191)은 이 계약을 유지했다. 현재 sparse HFT 표에는 ASCII만 있어
U+221A의 폭이 빠져 있다. Native의 실제 공급 환경이 표시 family를
`한양신명조`에서 `HY Sinmyeongjo`로 바꾸면 원본 표의 미수집 글자를 표시 family에서
다시 찾다가 반각 추정 550 HU로 내려간다. 표시 이름이 유지된 WASM과 환경 없는 진단은
기존 전각 1100 HU를 사용한다. 두 경로의 실제 글꼴 환경과 공개 font decision trace로
원인을 구분했다. 저장 줄·표 원점·문서 자체가 달라진 문제는 아니다.

독립 `pdf/issue8596_policy_committee_agenda-2020.pdf` 11쪽의 T6는 ToUnicode 04를
U+221A, CharProc HFT285에 연결한다. 해당 프로그램의 `d1` 전진은 1000/1000 em이다.
원본의 1100 HU 글자 크기와 합치며 현재 실제 공급 CFF의 hmtx도 같은 값을 준다.
글꼴 윤곽이나 프로그램 파일을 코드·저장소에 복사하지 않는다.

보정은 공통 HFT 메트릭 표에 이 독립적으로 확인한 전진을 추가하는 것이다. 문서별 좌표나
표 위치 보정은 추가하지 않는다. 원본 source 메트릭 → 글줄 측정 → TextRun 전진 → Native
glyph fitting과 뒤 항목의 원점이 같은 값을 소비하게 한다. 기존 ASCII·다른 family·수식
레이아웃의 별도 radical 프로그램은 적용 대상이 아니다. 실제 14쪽 재대조와 최소 글꼴
회귀를 끝내기 전에는 완료·승인을 선언하지 않는다.
