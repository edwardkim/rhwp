# 보고서의 도형 선과 머리말 재정의 보정 계획

2026-11-06, 제품 `9470fb026a8cd3b3d9594db55298b6f6c347bfda`에서 확인했다.
이 단계는 report-628의 미달 11쪽 가운데 도형·머리말 차이의 원인 보정이다.
원본 HWP와 독립 2020 Print PDF는 보존하며 검증 전에 완료를 선언하지 않는다.

## 도형 선의 실제 속성

136쪽의 그룹 사각형은 원본 모델과 한컴 재저장 진단 모두 검정 SOLID 33 HU다.
제목 글상자의 선은 보이지만 채우기 없는 본문 글상자의 선이 사라진다. 생산된
`DrawingObjAttr.border_line`을 `drawing_to_shape_style`이 변환한 뒤
`should_suppress_group_textbox_construction_stroke`가 `stroke_color=None`으로 덮어쓴다.
측정·좌표 문제가 아니라 모든 backend가 받는 최종 paint style의 손실이다.

선행 `232edb22b17454686092d50dbb0d59183142d984`(PR #6858, 이슈 #6852)은 일반 흰 사각형의
선 억제를 제거했으나 글상자의 경험적 A 조건은 유지했다. 그 조건은 사양상 비인쇄 계약이
아니라고 기존 설명에도 명시했다. 이번 원본의 독립 Print는 같은 조건의 선을 실제로 그린다.
문서·선 폭·채우기 조합으로 예외를 더 붙이지 않고 명시된 선을 그대로 출력한다.

이 억제의 최초 근거였던 편람의 실제 HWPX를 함께 확인했다. ‘행정업무 운영 개요’ 글상자는
원본 XML부터 `lineShape style="NONE"`이다. 전 383쪽 원본의 그룹 글상자를 순회해 기존 억제의
검정 SOLID·얇은 선·채우기 없음 조건을 가진 항목은 0개임을 확인했다. NONE의 보존은 원본
선 속성이 담당하고, 제목·본문 글상자의 가시성에서 선을 추측하지 않는다. 실제 편람 출력과
보호 PR의 TSV 보존 검증은 별도로 계속 필요하다.

## 머리말의 적용 범위 재정의

원본 Section3 문단 928은 ODD 머리말, 문단 932는 빈 BOTH 머리말을 정의한다.
155·193쪽의 독립 PDF에는 이전 ODD 머리말이 없지만 현재 후보는 계속 출력한다.
원본과 한컴 재저장 진단의 선언은 같다. 빈 문자열 여부가 아니라 적용 범위의 순서 문제다.

공통 `collect_header_footer_controls → ActiveHeaderFooter::accumulate → active →
PageContent.active_header → build_header`를 추적했다. 현재 코드는 새 BOTH를 별도 slot에
넣고 이전 ODD/EVEN slot을 남겨, `active`의 parity 우선 선택이 새 선언을 가린다.
새 BOTH 선언은 해당 머리말 또는 꼬리말의 이전 parity slot을 대체하도록 고친다.
뒤에 새 ODD/EVEN이 나오면 그 범위만 다시 정의한다. 머리말과 꼬리말은 서로 간섭하지 않는다.
복제에 실패했을 때 새 선언이 반쯤 적용되지 않도록 storage 복제를 먼저 완료한다.

## 검증과 미확정 범위

공용 target의 기존 WASM 작업이 종료된 뒤 두 보정을 함께 빌드한다. 도형 136쪽·머리말
155/193쪽을 실제 출력하고 같은 원본·PDF의 전쪽 Native/fresh WASM과 대조군을 확인한다.
기존 선 없음·채우기·일반 사각형, BOTH 뒤 parity 재정의·다른 머리말/꼬리말의 보존도 확인한다.
렌더링 회귀의 등록·변경은 정식 시각 선행 조건을 통과한 뒤 처리한다.

124·125쪽의 세로 차이는 앞 페이지 끝의 빈 문단 전진 2070/1432 HU와 일치한다.
선·머리말 보정에 섞어 좌표를 이동하지 않는다. 현재 각주 예약과 마지막 빈 줄 간격의
수용·이월·reset 소비를 별도로 추적 중이며, 이 일치만으로 원인 확정이나 수정을 하지 않는다.
수식 필드와 저장 줄 캐시를 분리·결합한 진단은 여전히 90% 미만이라 해결로 채택하지 않았다.

## R11 실제 결과 — 2026-11-07

위 두 공통 보정을 함께 빌드했다. Native는 고정 `target/pr-review`의 release-test·native-skia,
fresh WASM은 locked wrapper의 release 및 wasm-opt를 사용했고 둘 다 빌드 종료 코드 0이다.
두 소스의 SHA-256과 실제 산출물 해시는 [실행 증거](shape-header-r11/provenance.json)에 있다.
제품 기준은 `9470fb026a8cd3b3d9594db55298b6f6c347bfda`이며 그 위의 두 파일 변경을 정확히
고정한 산출물이다. source snapshot 안의 ‘미빌드’ 문구는 해시를 고정한 빌드 전 기록이며,
빌드 완료 여부는 실제 producer와 종료 코드로 확인했다.

| 대상 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 미달 |
| --- | ---: | ---: | ---: | --- |
| report-838 | 109 | 94.99% | 94.94% | 없음 |
| stored-band | 2 | 100.00% | 100.00% | 없음 |
| report-628 | 195 | 33.39% | 33.56% | 양쪽 같은 7쪽 |

[Native 전쪽 TSV](shape-header-r11/native/report-628.tsv) ·
[fresh WASM 전쪽 TSV](shape-header-r11/wasm/report-628.tsv).
Native와 fresh WASM의 이전 제품 대비 전 306쪽은 **각각 44쪽 개선·262쪽 동일·하락 0쪽**이다.
이는 [두 경로의 실제 페이지 대조](shape-header-r11/both-comparison.json)의 범위이며,
보호 PR #8389/#8397의 전체 검증 완료를 뜻하지 않는다.

| 보정 쪽 | 이전 Native | R11 Native | R11 fresh WASM | 판독 |
| --- | ---: | ---: | ---: | --- |
| 25 | 83.09% | 99.33% | 99.27% | 그룹 글상자의 명시된 선 보존 |
| 136 | 87.19% | 99.45% | 99.40% | 같은 도형의 선과 후속 본문 확인 |
| 155 | 89.13% | 99.95% | 99.95% | 빈 BOTH 이후 옛 ODD 머리말이 남지 않음 |
| 193 | 88.92% | 97.99% | 98.01% | 같은 재정의와 본문 내용 보존 |

25·136·155·193쪽의 review와 overlay를 두 경로 모두 직접 판독했다.
[Native 136쪽 review](shape-header-r11/native/review-136.png)와
[overlay](shape-header-r11/native/overlay-136.png),
[WASM 136쪽 review](shape-header-r11/wasm/review-136.png)와
[overlay](shape-header-r11/wasm/overlay-136.png),
[WASM 155쪽 review](shape-header-r11/wasm/review-155.png)와
[overlay](shape-header-r11/wasm/overlay-155.png)가 이 단계의 대표 증적이다.
선과 머리말 이외의 본문 미세 간격을 이 점수만으로 정확하다고 판정하지 않는다.
특히 193쪽 Native는 98% 미만이므로 그 쪽의 절대 px 회귀를 새로 등록하지 않는다.

최소 공개 글꼴 5개 환경의 집중 회귀는 실제 19건 통과·실패 0건이다.
자동 suite 재배치로 최초 8건 실행에서 빠진 모듈은 새 수집 경로에서 11건을 별도 실행했다.
교정한 #8679 검사도 후자에 포함된다. 세부 실행 이름과 terminal summary는
[검사 증거](stored-band-contract/r11-provenance.json)를 따른다. 전체 Rust·lint·backend 검사와
최종 통합 승인으로 확대 해석하지 않는다.

### WASM 집계 경위

최초 빌드는 성공했지만 부모 실행은 전체 쪽 집계 전에 종료 143으로 끝났다. 재개 중에는
검사·증적 커밋으로 루트 HEAD가 바뀌어 provenance 검사가 재개를 거부했다. 이 사실을
성공으로 고쳐 적지 않았다. 원 실행의 완료 묶음 5개는 모두 같은 출처이며 1~195쪽이
중복·누락 없이 존재한다. 각 묶음의 실제 WASM export metadata도 전체 195쪽이다.

모든 PNG와 쪽별 checkpoint 해시를 검사하고, 변경하지 않은 checkpoint를 새 PNG-pair
집계 경로에 모아 정본 `--silhouette-only --resume --png-pair`로 TSV를 산출했다.
새 렌더라고 주장하지 않으며 원본·PDF·실행 파일·package·글꼴·그림·계산 방법을 바꾸지 않았다.
[출처·쪽수·해시 대조](shape-header-r11/wasm-aggregation-provenance.json)에 원 실행과 재사용을
연결했다. 최종 집계의 종료 1은 실제 미달 7쪽 때문이며 PR 통과가 아니다.

## 남은 보정과 완료 경계

report-628의 16·18·120·122·123·124·125쪽은 양쪽 모두 90% 미만이다.
전 195쪽이 선행 조건을 충족하지 않아 이 보고서의 새 렌더링 회귀를 아직 등록하지 않았다.
수식·빈 꼬리 문단의 원인 보정과 직접 재검증을 계속한다.

추가 추적에서 123쪽은 Section3의 내부 page 91이며 각주 예약은 32.1px,
본문 pen은 817.8533px에서 다음 저장 reset으로 넘어간다. 마지막 빈 문단의 전체 전진
27.6px가 누적되지 않는다. `absorb_section_tail`의 조기 흡수는 실제 각주 측정 closure를
전달하는 뒤쪽 flow 단계보다 먼저 실행된다. 이 호출 순서와 fit·이월·reset 소비를 함께
검증해야 하며, 단순히 LineSeg 높이를 늘린 진단은 개선되지 않았다. 이 단계에 좌표 보정을
추가하지 않았다.

현재 전 306쪽은 영향 확인용 세 문서다. 원 PR 27건의 최종 범위는 최초 primary 51조합과
추가 원본·보고서, API 검증, 새 base 기능 보존, 보호 PR #8389/#8397, 편람 gate와 최종
회귀·lint·개별 리뷰를 포함한다. 전 306쪽만으로 그 범위를 축소하거나 완료로 판정하지 않는다.
