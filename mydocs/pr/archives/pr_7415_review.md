---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7415 검토 — 기준 PDF와 rhwp의 줄을 같은 규칙으로 구성

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head `0c2db5870827f43746445ca12b2079e52d105abf`는 이전 #7397/#7409 커밋을 다시 포함하고 현재 `devel`과 충돌한다. 고유 변경은 `f97e7e242`에 `-x` 체리픽했다. 기존의 속성 순서 무관 XML 파서와 새 baseline 줄 분리를 합친 충돌 보정, 단위 검사 갱신·보강을 거친 통합 code head `7cb91df10`에서 아래 범위를 검증했다. 원 PR 자체의 직접 병합은 승인하지 않는다. 최신 통합 PR CI와 원격 상태는 별도 조건이다.

## 접수·변경 범위

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7415](https://github.com/edwardkim/rhwp/pull/7415), `planet6897`, `devel` 대상 |
| 원 head / 고유 체리픽 | `0c2db5870827f43746445ca12b2079e52d105abf` / `f97e7e242` |
| 선행 중복 | `1d0123f39fd3`, `f1d205a481dc`는 병합된 #7397/#7409 변경과 겹쳐 적용하지 않음 |
| 메인터너 보정 | `dfc229652` (XML 파서·기준선 정밀도), `7cb91df10` (PDF/SVG 동일 기준선 반례 검사) |
| 검토 base / 통합 code head | `97073606de2545f88540429e2f7f8312eefa9641` / `7cb91df10` |
| 원격 참고 | 2026-09-25 OPEN, 원 head CI 실패·대기 없음, 최신 `devel`과 GitHub mergeability `CONFLICTING/DIRTY` |

`scripts/oracle_comparability.py`만 제품 경로로 바뀐다. 기존 PDF 파서는 `xml.etree.ElementTree`로 문자 속성 순서와 따옴표를 처리한다. 새 줄 규칙은 PDF 문자 원점과 SVG 텍스트 원점을 기준선별로 모으고 같은 가로 공백 기준에서 표의 이웃 칸을 분리한다. XML 파서에서 기준선 bucket을 만들되 보고 좌표는 원래 소수값을 보존했다. 조판·paint·Native/WASM 렌더 출력 경로는 **비해당**이다.

## 검증 입력·결과

- `samples/issue1853_caption_precedes_body_split.hwpx` SHA-256 `46bd0142f7fe2f68dffb9768e08d65488feeb19f5fd6916f9ad90ae5d72a96a9`, `pdf/issue1853_caption_precedes_body_split-hwpx-2020.pdf` SHA-256 `6577db6936337f4003553c1b8c492cfac42f06554b7dddca640b865ce5a100a8`. 두 파일 모두 검토 commit에 포함된 기존 추적 자료다.
- `RHWP_FONT_PATH=/Users/tsjang/Library/Fonts python3 scripts/oracle_comparability.py ... --rhwp-bin target/pr-review/release-test/rhwp --json`: 저장 첫 줄 재현 rhwp **308/349**, 한컴 **308/349**, `oracleKeepsButRhwpMisses=0`, 비교 불가 제외 0. 이 수치는 동일 규칙의 줄 대조가 가능한 실제 문서 한 쌍에서 얻었다.
- `python3 -m unittest scripts/tests/test_oracle_comparability.py`: 최종 head **10/10 PASS**. PDF 문자 속성 순서·글꼴 이름 인용부호, SVG run 순서, PDF/SVG 같은 기준선의 먼 칸 두 개 분리, 선택 쪽, 인쇄 배율 경계를 포함한다.
- 통합 branch의 `git diff --check upstream/devel...HEAD` 통과. 기여자 원 CI는 통합 head의 CI가 아니며, 전체 290쌍 재실행과 원격 통합 CI는 이 기록에서 완료로 쓰지 않는다.

## 조판 원칙 및 시각 증거

비교 도구의 줄 분류만 바꾸며 실제 문서의 조판·좌표·paint를 변경하지 않아 조판 원칙과 renderer Visual Sweep은 **비해당**이다. 사용자에게 보이는 판정값의 근거는 위 추적 HWPX/PDF와 직접 실행 JSON이며, 출력 PNG가 좋아졌다는 주장으로 확대하지 않는다. `#7417`의 PUA 처리와 함께 사용할 때의 수치·보정은 [별도 검토](pr_7417_review.md)에 기록했다.

## Merge 후 contributor PR comment 계획

실제 통합 PR merge SHA와 최신 CI URL을 확인한 뒤, 원 기여가 PDF/SVG 줄 구성을 대칭화한 점과 메인터너가 기존 XML 파서 보존·반례 검사를 더한 이유를 한국어 존댓말로 설명한다. `#7396`은 다른 판정 축이 남아 있으므로 이 PR만으로 닫지 않는다. comment는 `--body-file`로 게시하고 API로 본문을 재확인한다. 현재 comment·close·push·merge는 하지 않았다.
