# PR #8640 검토 — semanticist21의 다음 10건 통합

## 최종 판정

**승인 — 원 기여 10건과 메인테이너 보정을 검증한 통합 head를 병합했다.**
최종 head는 `4cf202d2aba44b0cfdd33465b03e37d1826a3fdc`, 병합 SHA는 `fd598250b7d8abee39eba1b1c68e604a326faa17`다.
실제 병합 시각은 `2026-11-05T11:44:39Z (20:44:39 KST)`이며, 병합 직전 `MERGEABLE / CLEAN; success 34·정상 SKIP 6`와 exact-head 필수 검증을 확인했다.
[작성자 self-review](https://github.com/edwardkim/rhwp/pull/8640#pullrequestreview-5784229538)를 완료했고
작업지시자의 CI·self-review 후 머지·후속 처리 승인을 적용했다.

원 PR은 [#8368](https://github.com/edwardkim/rhwp/pull/8368), [#8369](https://github.com/edwardkim/rhwp/pull/8369),
[#8385](https://github.com/edwardkim/rhwp/pull/8385), [#8484](https://github.com/edwardkim/rhwp/pull/8484),
[#8488](https://github.com/edwardkim/rhwp/pull/8488), [#8490](https://github.com/edwardkim/rhwp/pull/8490),
[#8492](https://github.com/edwardkim/rhwp/pull/8492), [#8494](https://github.com/edwardkim/rhwp/pull/8494),
[#8500](https://github.com/edwardkim/rhwp/pull/8500), [#8504](https://github.com/edwardkim/rhwp/pull/8504)다.
작성자는 semanticist21이며 원 head 10개를 병합 조상으로 보존했다. 원 상태는 [완료 영수증](../assets/semanticist21-20261105-next10/validation/remote-completion.json)에 개별 기록했다.

## Source와 실제 검증

| 구분 | 실제 근거 |
| --- | --- |
| 로컬 전체 검증 base·제품 source | base `6e1de406209ba9a58032f6ca6de45b18aa687d1c`, 제품 `584c9503baa651c62d0403dad9cb77c5cb9ffde6` |
| 시각 producer | `d53115fabc92bd392044d9b906c599baddb4368d`; 최종 유한수 검사와 영향 계산식 추가 실행은 별도 구분 |
| 현재 base 호환 | `a95d1d5afad1bfd61998794fa78fb0db100d690a` + 실제 probe `06bf0855b49d6a448b70987016489a65e546aff3` |
| GitHub의 실제 검증 merge | `e01add9810f4a38d93d2eb4fb56f7ccba2c3e767`, tree `2cdd76d28053268c7affdb64007e9bcd0f29bfce` |
| 로컬 전체 회귀·Native | 12,629 PASS·45 SKIP, Native 공식 3종 4,064 PASS·12 IGNORE, 필수 Rust 게이트 19단계 성공 |
| 이번 기능의 독립 Print 전쪽 | 46상태·124쪽씩, Native 최저 96.49% / fresh WASM 97.10%; 전쪽 누락·쪽수 불일치·90% 미만·2%p 초과 하락 없음 |
| 새 devel의 39 PR 보존 | 80상태·406쪽씩, 최저 92.45% / 92.99%; 경로별 PNG 403쪽 동일, 달라진 3쪽도 점수 하락 없음 |
| 현재 base 별도 확인 | 이번 248 PNG 동일, 새 글상자 편집 3·4쪽 양쪽 100%; 집중 Rust 18건·실제 입력/Esc·API·Studio 2,721건 성공 |
| GitHub 기본 기능 | A 3,801 / B 2,152 / C 2,089 / D 2,206 / E 2,195, 총 12,443 PASS·45 SKIP |
| GitHub 나머지 | Native·Lint·Frontend·Build & Test·CodeQL·Adapter·Proptest 성공; Studio 단위 2,721 PASS·2 SKIP |
| Render Diff | 실제 Canvas 3/3, direct PDF 3/3, CanvasKit readiness 8/8 성공 |

[기능별 원 기여·보정·소비 경로](semanticist21_20261105_next10_review.md),
[원본·독립 Print·글꼴·source/build 해시와 전쪽 TSV](../assets/semanticist21-20261105-next10/validation.md),
[현재 base의 실제 병합 호환](../assets/semanticist21-20261105-next10/current-base-compatibility.md),
[최종 GitHub 실행 영수증](../assets/semanticist21-20261105-next10/validation/github-ci.json)을 연결한다.
기존 producer의 전쪽 PNG를 최종 head에서 전부 다시 출력한 것으로 표시하지 않았다.

공통 조판 원칙 검토: 각주는 실제 본문 점유 끝을 소비하고, 바탕쪽 텍스트는 저장 본문 프레임을 소비한다.
문자 획·점선은 독립 Print에서 확인한 물리 결과를 backend가 공통으로 소비한다. 저장 LineSeg·머리 bit와
API의 원자적 거부는 실제 저장 왕복·편집·독립 기준으로 검증했다. 문서 ID별 보정이나 수치를 맞추는 분기는 추가하지 않았다.
FORMULA의 명시적 재계산 API와 일반 대화창의 값 쓰기, 첫 글자 장식 메타데이터 저장 보존 범위를 구분한다.

## CI에서 발견한 누락과 운영 복구

첫 head에서 로컬 실행을 빠뜨린 합성 renderer 계약이 실패했다. 원 해결 PR #3638과 독립 Print를 추적해
흰 채움·글자 크기 비례 획의 옛 기대값을 공통 물리 획의 소비로 보완했다. 제품·golden·문턱을 바꾸지 않았고
수정 전 FAIL·후 PASS 및 실제 Canvas/PDF/readiness 검증과 새 head CI를 모두 완료했다.
[보정과 남는 보호 범위](../assets/semanticist21-20261105-next10/renderer-contract-ci-review.md)를 기록한다.

초기 정책 게시의 GitHub API 503은 승인받은 1회 재실행에서 성공했다. 그러나 이를 최종 감사 뒤에 실행해
pending을 다시 게시한 순서 오류가 있었으며 메인테이너 자동화의 실수로 기록한다. 추가 승인된 최종 감사
`[42019684654 attempt 2](https://github.com/edwardkim/rhwp/actions/runs/42019684654/attempts/2)`에서 success를 확정한 뒤에만 병합했다. 제품 workflow는 재실행하지 않았다.

## 후속 처리

이번 PR의 고유 diff는 CI 실행 정의·재사용 판정 정책을 바꾸지 않는다. 보완한 e2e는 제품 text replay 계약이다.
admin maintainer의 승인된 운영 기록 직접 반영 경로로 active review·계획을 archive로 옮기고 이 문서와
[오늘할일](../../orders/20261105_pr8640.md)을 함께 보존한다. 제품·테스트·workflow·golden을 운영 기록에 섞지 않는다.
병합 뒤에는 duration 갱신 `[42021437016](https://github.com/edwardkim/rhwp/actions/runs/42021437016) success·46개 target 갱신·게시 완료(`cf867821a67966006af25de989aa592b94bbd47c`)`를 확인했으며 제품 검증 CI를 새로 시작하지 않았다.

원 PR의 자동 MERGED 또는 통합 수용 완료 상태와 연결 이슈 없음도 개별 확인했다. 각 댓글은 해당 PR과
동일 입력·쪽의 이미지·실제 수치에 대응하며, merge SHA의 PNG 응답 형식·Git blob·SHA-256을 확인한 뒤
정본 순서로 게시한다. 이후 devel 동기화·소유 작업공간/branch·root output 전체·검토 debug 정리를 마친다.
실제 게시·정리 결과는 `mydocs/privacy/pr-review-cleanup/20261105-next10.json`의 종료 영수증에 연결한다.
