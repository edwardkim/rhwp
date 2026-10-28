---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7397 검토 — 한컴 PDF 비교 가능성 진단

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head의 SVG 한 text run 글자 순서 손상을 보정 `a6b27fb9dbc58c7f5de141011365933f7eb814d0`에서 해결했다. 제어 입력 도구 회귀 7건과 실제 API 문서의 진단 실행이 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 통과했다. 원 PR head 단독은 승인하지 않는다. 실제 진단은 `비교가능_주의`이며 모든 글꼴·줄 정보를 일치로 판정한 것이 아니다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7397](https://github.com/edwardkim/rhwp/pull/7397), `planet6897`, `devel` 대상 |
| 원 head / `-x` 체리픽 | `1d0123f39fd398e6702225a693fcc9dfd529b914` / `e4e3392eac32f903d520805d1226e52837b4f958` |
| 공유 메인터너 보정 | `a6b27fb9dbc58c7f5de141011365933f7eb814d0`; #7409에도 적용 |
| 검토 base / code head | `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` / `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88` |
| 원격 참고값 (2026-09-25) | OPEN, MERGEABLE/CLEAN, Draft 아님; 1파일, +426/−0 |

[보정·통합 순서](pr_7397_review_impl.md)와 #7409의 별도 검토를 함께 본다. 원 head CI는 이 누적 head의 CI를 대체하지 않는다.

## 결함과 적용 경계

`scripts/oracle_comparability.py`는 rhwp SVG text를 (x, glyph, size) 줄로 읽어 PDF 줄과 대응시키고 `paperBox`, `fontScale`, `contentScale`, `storedLines`, 선언 face 대체 여부를 판정한다. 원 head는 동일 x의 한 run 글자까지 tuple 전체로 정렬해 `cab`을 `abc`로 바꿨다. 보정은 run 내부 문자 순서를 보존하고 run 사이만 x로 정렬한다. PDF XML 속성 순서와 선택 쪽 처리도 같은 commit에서 보정했다. 제어 SVG에서 줄 키가 원문 순서를 유지하는 계약은 **충족**, 모든 실제 문서의 자동 진단 정확도는 **미검증**이다. renderer 조판 변경은 **비해당**이다.

실제 입력 `samples/hwpctl_API_v2.4.hwp` SHA-256 `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603`, 기준 `pdf/hwpctl_API_v2.4-2022.pdf` SHA-256 `a0141ca188ca638b305d896e21749703a39529f7bd8cf2bc806dea4ff9aaac66`을 썼다.

## 검증과 진단 결과

- `python3 -m unittest scripts.tests.test_oracle_comparability -q`: **7/7 PASS**. 통합 전체 Rust release-test **10,229/10,229 PASS·50 skip** 및 필수 lint PASS.
- 같은 code head의 API 49·60쪽 실제 실행: `paperBox=일치`(PDF 595.0×841.0pt, rhwp 595.3×841.9pt), `fontScale=일치`(47줄, 중앙값 0.996), `contentScale=제자리`(30줄, xSlope 1.0035, ySlope 0.9995). 선언 face 11개 중 PDF가 10개를 대체했고 `storedLines=미측정`이므로 최종 `비교가능_주의`다. 대체 슬롯의 폭 차이를 곧바로 rhwp 결함으로 귀속하지 않는다.
- 원시 JSON과 로그는 Git에서 제외한 `output/pr-review/planet6897-20260924/diagnostics/approved-v2-api-oracle.json` 및 `approved-v2-oracle-tests.txt`에 있다. 다른 PR의 Visual Sweep PNG를 이 도구 결함 수정의 시각 증거로 승격하지 않는다.

도구 전용 변경에는 renderer Native/fresh WASM Visual Sweep이 비해당이다. 통합 PR의 원격 CI·mergeability는 생성 후 exact head에서 확인한다.

## Merge 후 contributor PR comment 계획

실제 통합 merge 뒤 merge SHA·CI URL, 원 기여의 비교 가능성 분류와 메인터너의 문자 순서·XML·선택 쪽 보정, 실제 `비교가능_주의` 결과 및 미측정 범위를 한국어로 알린다. 현재 comment·approve·push·merge는 수행하지 않았다.
