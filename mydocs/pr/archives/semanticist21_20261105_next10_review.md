# semanticist21 다음 10건 통합 검토 — 2026-11-05

## 최종 판정

**승인 — 메인테이너 보정 통합 head의 CI·self-review를 확인한 뒤 병합했다.** 최종 head `4cf202d2aba44b0cfdd33465b03e37d1826a3fdc`, merge `fd598250b7d8abee39eba1b1c68e604a326faa17`. [PR #8640 최종 기록](pr_8640_review.md)에 정확한 gate와 후속 처리를 연결한다.


선정·원 head·reviewer 지정과 승인 범위는 [수행 계획](semanticist21_20261105_next10_review_plan.md)에 있다. 원 PR 10개를 merge 조상으로 보존했고 원 fork branch를 변경하지 않았다. 원격 처리 승인을 받아 통합 PR 병합과 원 PR 10건의 자동 MERGED를 확인했다.

## 실제 source와 완료 범위

- 로컬 branch: `review/integration-20261105-semanticist21-next10`.
- 전체 검증의 고정 base: `6e1de406209ba9a58032f6ca6de45b18aa687d1c`, [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622).
- 최종 제품 source: `584c9503baa651c62d0403dad9cb77c5cb9ffde6`, source tree `e66b2bf4d66655b52e6f2db925ff058dbb181941`.
- 시각 producer: `d53115fabc92bd392044d9b906c599baddb4368d`. 이후 일반 계산 API의 비유한 결과 검사만 보정했고 renderer·paint·model·parser·serializer·Studio·입력/PDF tree는 같다. 이 producer와 최종 source의 추가 실행을 구분했다.

[최종 실제 명령과 결과](../assets/semanticist21-20261105-next10/validation.md):

| 검증 | 실제 결과 |
| --- | --- |
| 정식 release build·lib | build 성공, lib 3,983 PASS·12 IGNORE |
| 최소 글꼴 전체 nextest | **12,629 PASS·0 FAIL·45 SKIP** |
| Native 공식 3종 | **4,064 PASS·12 IGNORE** |
| Rust 품질 검사 | fmt·root/WASM32/workspace all-target Clippy·workspace build·doc test 모두 성공 |
| fresh WASM·Studio | 최적화 build, 실제 API 10묶음·한컴 계산 값 37개, 실제 계산식 대화창·Canvas2D 6장면·CanvasKit 효과·bindings, TypeScript·2,720 PASS·2 기존 SKIP·build 모두 성공 |
| 이번 10건 전쪽 비교 | 46상태·124쪽씩, Native 최저 **96.49%** / WASM **97.10%** |
| 새 devel 기능 보존 | 80상태·406쪽씩, Native 최저 **92.45%** / WASM **92.99%**, 경로별 403 PNG 동일·달라진 3쪽의 점수 하락 없음 |
| 최종 계산식 추가 실행 | 7상태·7쪽씩, 양쪽 전부 **100.00%**·이전 producer와 PNG 동일 |
| canonical Native 편람 | 383쪽 gate passed, 최저 99.96%·370쪽 100% |
| 공개 fixture 보안 | 45개 × CLI 탐지 3종 전부 clean |

전쪽의 누락·독립 Print 쪽수 불일치·90% 미만·2%p 초과 하락은 없다. [전쪽 TSV·입력/PDF 해시](../assets/semanticist21-20261105-next10/validation/post8622/summary.json), [최신 devel 보존](../assets/semanticist21-20261105-next10/validation/inherited8622/summary.json), [직접 확대 판독·이미지 해시](../assets/semanticist21-20261105-next10/validation/post8622/direct-reading.md)를 완료했다. 수치와 직접 판독을 구분한다. 이전 묶음의 예외를 적용하지 않았다.

마지막 보정은 실제 대화창에서 `LN(-1)`이 잘못된 JSON과 셀의 NaN 기록으로 이어지던 공통 API다. 기존 저장 FORMULA의 유한수 검사를 일반 API와 공통화했다. 수정 전 실제 API 실패와 최종 Native/WASM·정식 회귀·대화창 성공을 연결했다. [최종 source의 실제 실행 계수](../assets/semanticist21-20261105-next10/validation/finite-final/completion.json)를 확인한다.

## 최종 확인에서의 base 전진

전체 검증 후 fetch에서 현재 upstream/devel `a95d1d5afad1bfd61998794fa78fb0db100d690a`를 확인했다. #8630의 묶음 글상자 편집 주소·클릭/Esc 복귀와 CI 디스크 관측, #8633·#8634의 후속 기록이 추가됐다. `git merge-tree --write-tree`는 충돌 없이 `a9a48f3bf44a26595d30a2e7e7e216ccd9b53af2`를 만들었다. 검증한 코드 head를 임의 갱신하지 않고 [현재-base 병합 트리의 실제 호환](../assets/semanticist21-20261105-next10/current-base-compatibility.md)을 별도로 완료했다. Native·최적화 WASM으로 이번 124쪽씩을 다시 출력했고 248 PNG가 이전 후보와 모두 같다. 새 편집 경로의 3·4쪽은 양쪽 100%이며, 집중 Rust 18건·실제 글상자 편집과 이번 API·Studio 2,721건도 성공했다. 전체 19단계와 기존 530쪽을 새 probe에서 재실행했다고 표시하지 않는다.

## 원 기여와 메인테이너 보정의 구분

| 원 PR | 수용할 기여 | 메인테이너 측 처리 |
| --- | --- | --- |
| [#8368](pr_8368_review.md) | 원형 점선·단일/이중 물결선 | 3D 비표시 계약을 보존해 충돌을 통합했다. 원형 점선은 독립 2020/2024 16굵기 Print의 실제 지름·간격을 공통 경로에 연결했다. |
| [#8369](pr_8369_review.md) | 글자 외곽선 여섯 종류 | PDF 명시 TextPaint·shaping을 보존했고 CanvasKit도 Rust 공통 pen/dash descriptor를 소비하도록 보완했다. |
| [#8385](pr_8385_review.md) | 셀 블록 높이·너비 균등화와 재격자 | 빈 격자 줄의 나눗셈을 오류 검사 뒤로 옮겼다. 실제 API 적용·HWP/HWPX 저장본·독립 Print를 추가했다. |
| [#8484](pr_8484_review.md) | LOG/LN·각도·ROUND 함수 | 과거 CI 실패는 계산식 결함이 아닌 WMF 그림/글꼴 계약이었다. 선행 해결을 추적하고 LN/ROUND를 #8504와 함께 실제 계산했다. |
| [#8488](pr_8488_review.md) | 각주 모양 조회·적용·전파 | 기존 inline 각주 배치·선두 참조/머리 겹침·위첨자 투영·LineType2 저장 매핑을 공통 결과로 보정했다. |
| [#8490](pr_8490_review.md) | 구역 개요 번호 ID 설정 | u16 overflow·타입 오류·존재하지 않는 ID를 전체 갱신 전에 거부하고 TypeScript 선언을 보완했다. |
| [#8492](pr_8492_review.md) | 첫 글자 장식 메타데이터 저장 보존 | 제품 코드의 추가 보정 없이 양쪽 저장과 검증 근거를 보강했다. 새 장식 생성 UI·자동 재배치 범위로 확대하지 않는다. |
| [#8494](pr_8494_review.md) | 구역별 미주 번호·편집 후 갱신 | 기존 바탕쪽 일반 텍스트의 원점을 저장 본문 프레임에 연결했다. 독립 Print의 의도된 겹침은 보존한다. |
| [#8500](pr_8500_review.md) | 문단 머리의 첫 줄 플래그 | 기존 공통 재조판·첫 줄 fit 흐름에 통합하고 편집·해제·양쪽 저장의 실제 플래그를 확인했다. |
| [#8504](pr_8504_review.md) | FORMULA 필드와 명시적 재계산 | 실제 셀 편집 뒤 37개 한컴 기대값·필드 33개·생성/삭제/순환 거부와 저장 왕복을 검사했다. Studio 자동 타이핑 재계산 UI를 추가했다고 설명하지 않는다. |

## 원격 처리와 후속 절차

[PR #8640 최종 기록](pr_8640_review.md)에 원격 승인·실제 CI·self-review·merge와 운영 후속 순서를 확정했다. 첫 head의 renderer 계약 누락과 초기 정책 게시/최종 감사의 순서 오류는 숨기지 않고 [보정 기록](../assets/semanticist21-20261105-next10/renderer-contract-ci-review.md) 및 최종 기록에 연결한다. 제품 CI는 병합 전에 완료했고 병합 뒤 중복 검증 CI를 실행하지 않는다.
