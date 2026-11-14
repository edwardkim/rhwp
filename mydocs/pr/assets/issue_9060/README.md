# PR #9075 검토 증적

이 폴더의 TSV와 PNG는 2026-11-14 메인터너 검토에서 같은 원본·기준 PDF로 다시 산출한 결과다.
기존 Native 첨부의 본문 서체가 PDF와 다르던 문제를 실제 Print 글꼴 프로그램 공급 조건으로 확인했다.
제품 조판 코드·원본·PDF·판정 임계값·비교 영역은 변경하지 않았다. 글꼴 파일은 포함하지 않는다.

## 현재 측정

코드 원 PR head는 `1309dbea65b7a95e25fa7d6abd3e10b5e65e5eb4`, 기준 devel은
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. 충돌 없는 병합 예상 트리
`4c80a94f2855eafb19bc764cdab9f84820fc51c8`에서 Native와 fresh WASM을 각각 빌드했다.
실제 producer와 입력·출력 SHA는 [측정 기록](review-measurement.json)에 있다.

- 입력: `samples/issue9060/86036-rowbreak-first-line-fragment.hwpx`
- 기준: `pdf/issue9060/86036-rowbreak-first-line-fragment-2020.pdf` — 기존 한컴 Print 14쪽을 재사용했다.
- 조건: Print profile, compat 2022, 96 DPI, 색상 경계 보정 전 2px 실루엣.
- 글꼴: PDF의 실제 face인 휴먼명조·Palatino Linotype·H2mjsM·H2gtrM·맑은 고딕을 확인하고
  로컬 프로그램을 공급했다. H2mjsM은 HY신명조의 PostScript 이름이다.
  [명시적 글꼴 환경](review-font-environment.json)은 이름·선택 규칙만 포함한다.
- 두 경로 모두 전체 14쪽이며 내용의 5·6쪽 분할과 7쪽 후속 본문을 확인했다.

| 범위 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 4쪽 | 100.00% | 100.00% |
| 5쪽 | 99.98% | 100.00% |
| 6쪽 | 100.00% | 100.00% |
| 전쪽 최저 | 99.72% (12쪽) | 99.99% (2·12쪽) |

전체 원본 TSV: [Native](native_silhouette.tsv) · [fresh WASM](wasm_silhouette.tsv).
5쪽의 지적된 첫 글줄은 새 Native에서 2px 미대응 픽셀 0개다. 5쪽 전체의 나머지 차이와
12쪽의 최저값은 숨기지 않고 전쪽 TSV에 그대로 남겼다. 표시값 100.00%는 반올림된
2px 실루엣 값이며 엄격한 RGB 픽셀 동일성을 뜻하지 않는다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 4 | [review](native_review_004.png) · [overlay](native_overlay_004.png) | [review](wasm_review_004.png) · [overlay](wasm_overlay_004.png) |
| 5 | [review](native_review_005.png) · [overlay](native_overlay_005.png) | [review](wasm_review_005.png) · [overlay](wasm_overlay_005.png) |
| 6 | [review](native_review_006.png) · [overlay](native_overlay_006.png) | [review](wasm_review_006.png) · [overlay](wasm_overlay_006.png) |
| 12 | [review](native_review_012.png) · [overlay](native_overlay_012.png) | [review](wasm_review_012.png) · [overlay](wasm_overlay_012.png) |

작성자 최초 조건의 Native 4쪽 95.47%·5쪽 99.20% 등 과거 기록은 원 커밋
`1309dbea65b7a95e25fa7d6abd3e10b5e65e5eb4`의 Git 이력에 보존된다. 회귀 검사 첫머리는
그 당시 조건의 측정을 기록한다. 현재 폴더의 갱신된 측정과 혼동하지 않는다.

정본 보호 대상 전체와 편람 gate의 최종 판정은 별도 최종 PR 검토 문서에서 확인한다.
이 폴더의 주 입력 검증만으로 통합 전체의 승인을 선언하지 않는다.
