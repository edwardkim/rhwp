# jeong-sik 열린 Draft PR의 최신 devel 해결 여부 조사

## 결론

**전부 해결되지 않았다.** #7668의 soft-edge 꼬리 크기 파싱은 #8656 및 #8889 보정 뒤 재현물에서 통과하지만, 혼합 그림 셀은 미해결이다. #7675의 줄바꿈·최소 높이와 #7682의 여러 문단 배치도 재현물에서 차이가 남는다. 세 PR은 #7668 → #7675 → #7682의 누적 관계이며 서로 독립된 수정 3개로 합산하지 않는다.

기준은 `upstream/devel` **d7b927d073489b8c5ea95655224b04096c939534**다. 제품 코드는 변경하거나 체리픽하지 않았다. 기존 reviewer `edwardkim`을 유지했으며 PR 상태·리뷰어·댓글·issue·fork를 변경하지 않았다. 정식 수용 review가 아닌 현재 devel의 해결 여부 조사다.

## 같은 입력의 Native·독립 Print 비교

공개 재현물 31종 중 각 주장과 직접 관련된 7종·7쪽을 최신 devel에서 실제 출력했다. 동일 파일은 MCP 한컴 2020 **11.0.0.9136 / Print method 0 / hancom2020_pdf_driver_one_up**로 새 출력했다. 원 PR의 WebHwp 화면은 Print 기준으로 사용하지 않았다. 정상 Mac 로컬 글꼴 환경·96DPI·compat2022·정본 Visual Sweep 2px raw 실루엣을 사용했다. 각 원본의 저장 제품은 한컴2018이며 2020 engine 선택과 일치한다.

| PR | 입력 | Native/PDF 쪽수 | raw 일치율 | 결과·TSV |
| --- | --- | ---: | ---: | --- |
| [#7668](https://github.com/edwardkim/rhwp/pull/7668) | [grid-soft-hancom.hwp](cases/pr7668-grid-soft-hancom/grid-soft-hancom.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7668-grid-soft-hancom-2020.pdf) | 1/1 | 99.97213% | passed · [TSV](cases/pr7668-grid-soft-hancom/native.tsv) |
| [#7668](https://github.com/edwardkim/rhwp/pull/7668) | [mixed.hwp](cases/pr7668-mixed/mixed.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7668-mixed-2020.pdf) | 1/1 | 0.00000% | re_review_required · [TSV](cases/pr7668-mixed/native.tsv) |
| [#7675](https://github.com/edwardkim/rhwp/pull/7675) | [all-inline.hwp](cases/pr7675-all-inline/all-inline.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7675-all-inline-2020.pdf) | 1/1 | 1.94372% | re_review_required · [TSV](cases/pr7675-all-inline/native.tsv) |
| [#7675](https://github.com/edwardkim/rhwp/pull/7675) | [bottom.hwp](cases/pr7675-bottom/bottom.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7675-bottom-2020.pdf) | 1/1 | 6.06745% | re_review_required · [TSV](cases/pr7675-bottom/native.tsv) |
| [#7682](https://github.com/edwardkim/rhwp/pull/7682) | [bottom.hwp](cases/pr7682-bottom/bottom.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7682-bottom-2020.pdf) | 1/1 | 44.51006% | re_review_required · [TSV](cases/pr7682-bottom/native.tsv) |
| [#7682](https://github.com/edwardkim/rhwp/pull/7682) | [two-paras.hwp](cases/pr7682-two-paras/two-paras.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7682-two-paras-2020.pdf) | 1/1 | 9.53881% | re_review_required · [TSV](cases/pr7682-two-paras/native.tsv) |
| [#7682](https://github.com/edwardkim/rhwp/pull/7682) | [wrapped.hwp](cases/pr7682-wrapped/wrapped.hwp) · [Print](../../../../pdf/jeong-sik-develop-audit-20261114/pr7682-wrapped-2020.pdf) | 1/1 | 18.89514% | re_review_required · [TSV](cases/pr7682-wrapped/native.tsv) |

0%는 파일·그림이 전혀 없다는 뜻이 아니라, 현재 배치와 기준 그림의 내용 실루엣이 2px 범위에서 일치하지 않는다는 의미다. 실제 review에서 겹침·행 소속·위치·그림 내부 자르기 차이를 확인했다. 숫자는 그림 안에 들어 있는 비트맵이므로 번호 소실을 본문 글꼴 미설치로 분류하지 않았다.

## 직접 판독한 대표 증적

| 입력 | Native raw | PDF PNG | review | overlay |
| --- | --- | --- | --- | --- |
| pr7668-grid-soft-hancom | [raw](cases/pr7668-grid-soft-hancom/raw-001.png) | [PDF](cases/pr7668-grid-soft-hancom/pdf-001.png) | [review](cases/pr7668-grid-soft-hancom/review-001.png) | [overlay](cases/pr7668-grid-soft-hancom/overlay-001.png) |
| pr7668-mixed | [raw](cases/pr7668-mixed/raw-001.png) | [PDF](cases/pr7668-mixed/pdf-001.png) | [review](cases/pr7668-mixed/review-001.png) | [overlay](cases/pr7668-mixed/overlay-001.png) |
| pr7675-all-inline | [raw](cases/pr7675-all-inline/raw-001.png) | [PDF](cases/pr7675-all-inline/pdf-001.png) | [review](cases/pr7675-all-inline/review-001.png) | [overlay](cases/pr7675-all-inline/overlay-001.png) |
| pr7682-two-paras | [raw](cases/pr7682-two-paras/raw-001.png) | [PDF](cases/pr7682-two-paras/pdf-001.png) | [review](cases/pr7682-two-paras/review-001.png) | [overlay](cases/pr7682-two-paras/overlay-001.png) |

## 다른 PR을 통한 선행 해결

- [#8656](https://github.com/edwardkim/rhwp/pull/8656)의 `d8e6c8a445`는 알려진 단독 soft-edge 뒤 원본 크기·alpha의 위치를 보정했다. [기존 검토](../../archives/pr_8656_review.md)와 같은 원인 계층이다. 단독 soft-edge 외 모든 복합 효과 지원을 의미하지 않는다.
- [#8889](https://github.com/edwardkim/rhwp/pull/8889)의 `050e78455e`는 반경의 원본/저장·공통 paint·인쇄 평면을 보완했다. [기존 보정 기록](../../archives/pr_8889_review_impl.md)을 대조했다.
- #8658/#9025의 그림 칸 보정도 확인했지만, 그 해결 범위를 이 Draft의 TopAndBottom+TAC 혼합·내어쓰기·여러 문단 경로 전체와 동일시하지 않았다. 현재 재현 결과가 전부 해결됐다는 가정을 반증한다.

## 범위와 재현성

[최종 조사 원장](findings.json)에 source head·입력 Git blob/SHA-256·PDF SHA·작업 ID·실제 출력 제품·Native producer·TSV/PNG 해시·직접 판독을 연결했다. 정상 종료한 Native 빌드는 167.125초이고 별도 전체 회귀·fresh WASM은 실행하지 않았다. 이 자료로 비공개 회사 원본이나 공개 31종 전체의 상태를 추정하지 않는다. 이번 결과만으로 source PR의 코드가 수용 가능하다는 판정이나 close 승인을 하지 않는다.

임시 드라이버·전체 API 응답·원 로그·중간 JSON은 ignored output에서만 만들었다. 이 경로에는 검증에 실제 사용한 7개 원본과 최종 TSV·대표 PNG·확정 조사 원장만 보존한다. 제품 source·테스트·기존 PDF·판정 문턱은 수정하지 않았다.
