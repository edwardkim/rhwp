---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7393 검토 — 바탕쪽 manifest의 구역 소속

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 PR은 바탕쪽을 소속 구역 앞에 쓰는 방향이 맞지만, 균등 분포 표본만으로는 비균등 구역의 소속 보존을 입증하지 못했다. 보정 `096e42a98cee9a955f143dd68a639def9fc64a40`이 한컴 2024 저장본의 14구역 반례와 원본·왕복본 PDF를 추가했다. 보정을 포함한 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 집중 검사 3건과 전체 10,229건이 통과했다. 원 PR 단독 승인 근거로 사용하지 않으며, 통합 PR의 원격 CI와 mergeability는 생성 후 별도로 확인한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7393](https://github.com/edwardkim/rhwp/pull/7393), `planet6897`, `devel` 대상 |
| 원 head / `-x` 체리픽 | `2f35381aa6f080a24cf6398967f552e26ff1a856` / `774ab3844d1340aa24e67b1c64b684c6eaa7f60f` |
| 메인터너 보정 | `096e42a98cee9a955f143dd68a639def9fc64a40` |
| 검토 base / code head | `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` / `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88` |
| 원격 참고값 (2026-09-25) | OPEN, MERGEABLE/CLEAN, Draft 아님; 3파일, +159/−14 |

[보정·통합 순서](pr_7393_review_impl.md)를 함께 보존한다. 원 head CI는 누적 통합 head의 검증 결과가 아니다.

## 변경 계약과 독립 입력

`serialize_hwpx_with_report`가 바탕쪽의 구역 소속을 만들고 `write_content_hpf`가 각 바탕쪽을 소속 구역 manifest 항목 바로 앞에 쓴다. 파서는 manifest의 구역 전 항목을 소속으로 소비하며, 구역 XML에 `masterPage idRef`가 있으면 그 참조가 우선한다. id/href 전역 인덱스는 유지한다. 변경 대상은 HWPX 저장·재열기 계약이며 renderer의 조판·paint 좌표를 바꾸지 않는다.

독립 입력 `samples/2025 행정업무운영 편람(최종).hwpx`의 SHA-256은 `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`이다. `lastSavedWith`는 Hancom Office 2024 `13.0.0.3379`다. 원본 구역별 바탕쪽 개수는 `[0,2,1,1,1,1,1,1,1,1,1,2,2,0]`이며 0·13구역에는 `masterPage idRef`가 없다. #7393 적용 전에는 0구역에 2개가 잘못 붙어 재열기 검사가 실패했다. 적용·보정 후에는 소속 개수, 적용 범위, 앞쪽·대체 여부, 텍스트 방향, 문단 텍스트와 개체 수가 보존된다. 이 비균등 소속 계약은 **충족**이다. 임의의 모든 HWPX 생성 경로는 **미검증**이다.

## 검증과 한컴 출력

- 통합 code head의 `issue_6907` 집중 release-test **3/3 PASS**, 전체 release-test **10,229/10,229 PASS·50 skip**, fmt·Native/WASM/workspace Clippy·workspace build·manifest base 비교 PASS.
- `rhwp hwpx-roundtrip`의 원본→저장→재열기 결과는 `PASS diff=0 r2=0`. 같은 입력의 한컴 원본과 왕복본을 `hwp2024-mcp-convert --engine 2024`로 각각 PDF 변환했다. 두 PDF는 모두 383쪽이고, 96 DPI로 펼친 3–8쪽은 대응 쪽의 변경 픽셀이 0개였다. 전체 383쪽의 픽셀 동일성까지 주장하지 않는다.

| 한컴 출력 | SHA-256 |
| --- | --- |
| [원본 PDF](../../../pdf/issue6907/pr7393-source-2024.pdf) | `a1064cec905815cb0d446304cecdc6c544ef4adc46c803ca1f0f016a60d6a1fe` |
| [왕복본 PDF](../../../pdf/issue6907/pr7393-roundtrip-2024.pdf) | `6774ed2acc9ea6500a2b437c1d5bdfd7dfcebe7d54ddc9418699d96d2a189932` |

이 PDF는 저장 계약을 한컴 출력으로 대조한 증적이다. 이번 PR에 renderer 변경이 없으므로 Native/fresh WASM Visual Sweep 적용 대상은 아니다. 변환·검사 로그는 Git에서 제외한 `output/pr-review/planet6897-20260924/`에 보관한다.

## Merge 후 contributor PR comment 계획

실제 통합 merge 뒤에만 원 기여와 메인터너의 반례·PDF 보강을 구분하고, merge SHA에 고정한 두 PDF 링크, CI 결과, 검증 범위(3–8쪽)를 한국어로 알린다. #6907 전체 해결·종료 표현은 쓰지 않는다. 현재 comment·approve·push·merge는 수행하지 않았다.
