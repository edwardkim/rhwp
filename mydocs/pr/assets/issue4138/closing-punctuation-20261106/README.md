# #4138 한점 리더·한자 금칙의 공통 줄 소유

2026-11-06 부모708256f1ba에 공통 토큰화 보정을 적용한 source다. 실제 생산 파일·diff·실행 파일·
WASM·입력·독립 PDF 해시는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/closing-punctuation-20261106/validation.json`)으로 고정했다. 입력과 기준 PDF는
앞 단계와 동일하며 새로 생성한 분할 저장본이나 새 정답 PDF로 바꾸지 않았다.
현행 Visual Sweep의 PDFium·192DPI·2024 compat·색상 경계 보정 전2px raw다.

| 경로 | 30쪽 | 189쪽 | 전197쪽 최저 | 98% 미달 |
| --- | --- | --- | --- | --- |
| Native | 100.00000% | 99.98891% | 94.83034% | 21·44·120·129·130쪽 |
| fresh WASM | 100.00000% | 99.99010% | 94.98570% | 21·44·120·129·130쪽 |

30쪽은 U+2024가 줄머리 금칙에서 빠져 `흡기 | ․냉각수`로 나뉘었다. 독립 Print의 `흡 | 기․냉각수`와
[Unicode UAX #14의 IN/LB22](https://www.unicode.org/reports/tr14/#LB22)를 근거로 공통 금칙 목록을 보완했다.
선행 PR#6725→9ae601354d3→f7426ad95는 U+2027의 같은 처리를 도입했지만 U+2024는 빠져 있었다.
189쪽은 CJK 토큰만 후행 금칙 흡수를 건너뛰어 `物 | )`로 나뉘었다. 한글 글자 토큰과 공통 처리를
공유하도록 하여 독립 Print의 `下 | 物)`을 복원했다. CJK는 낱말 모드에서도 기존 글자 단위를 유지한다.
문서·쪽·너비 상수의 예외를 추가하거나 저장 프레임 검사를 제거하지 않았다.

[Native30 review](native-p030-review.png)·[overlay](native-p030-overlay.png),
[189 review](native-p189-review.png)·[overlay](native-p189-overlay.png),
[WASM30 review](wasm-p030-review.png)·[overlay](wasm-p030-overlay.png),
[189 review](wasm-p189-review.png)·[overlay](wasm-p189-overlay.png)를 직접 열었다.
앞뒤 문단, 물리 줄 소속, 닫는 괄호, 행 시작과 마지막 본문을 비교했다. 일부 섭씨 기호의 미세 차이는 남는다.
전쪽 TSV의 197쪽 및 두 경계 PNG 해시는 집중 검사와 전체 실행이 같다.
새 고정형2건은 실제 수정 전2 FAIL→수정 후2 PASS다. 관계형은 절대 좌표와 분리해 원문 보존도 검사한다.

## 수식 글꼴 공급과 남은 범위

일부 수식은 `RHWP Hancom Legacy Equation`을 요구했다. Mac의 기존 `RHWP Legacy Equation`은
다른 family이므로 자동 동일시하지 않았다. 대상 문서와 독립된84문자 표본을 기존
`scripts/prepare_legacy_equation_font.py`로 만들고 한컴2020 Print(11.0.0.9136, 방식0,
job9a0ef340-fe76-4ebc-b090-a6d6544c008a)로 출력한 뒤 Type3 75자를 로컬 검증용으로 복원했다.
대상 PDF는 글꼴 원본으로 사용하지 않았다. 대상 T6의39개 프로그램 중37개가 독립 표본과 같고
공백·U+22C5 두 개는 다르거나 표본 범위 밖이다. 모든 글리프가 같다고 주장하지 않는다.
같은 공급 조건의 base/후보 44·129·130쪽 결과도 같으며 글꼴 효과를 구현 개선으로 합산하지 않는다.
글꼴 프로그램·바이너리·윤곽 배열과 표본 중간 PDF는 로컬 output에만 두며 배포하지 않는다.

Native/fresh WASM은 전197쪽을 각각 완료했으나 위5쪽은 아직98% 미만이다. 일부 수식의 자연 폭·간격,
21쪽과120쪽의 줄 배치를 계속 보정해야 한다. 전체 브랜치98% 완료나 PR 준비 완료가 아니다.
Mac WASM은 locked wrapper web/pkg/--no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

정식 고정형2건·관계형2건은 모두 수정 전4 FAIL→수정 후4 PASS였다. 기존 저장 프레임·90쪽·중첩표 잘림을 포함한 Mac 최소 글꼴 집중 검사는 총11 PASS다.
