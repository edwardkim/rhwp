# 참조 번호의 문자군·원본 스타일과 공통 전진 칸

stage13 `01d2197973`의 본문 참조 `56)`는 한글 run의 휴먼명조를 사용했다. 원본 char shape 75는
한글 휴먼명조와 영문 HCI Poppy를 구별한다. 생성된 번호의 문자군으로 슬롯을 선택하고 상대 크기·
장평·원본 메트릭 문맥을 보존했다. 본문의 자간·양쪽 정렬·탭·장식은 번호 칸에 복사하지 않는다.
번호의 공통 전진 배열을 Native·SVG·Canvas와 paint JSON에 전달하며 논리 control 소유는 보존한다.
CanvasKit의 기존 미지원 guard는 그대로다.

## 독립 근거와 검사

같은 report-628 원본과 한컴2020 Print PDF의 SHA·실행 producer는 [검증 기록](validation.json)에 있다.
PDF의 `/HCI Poppy` 이름에는 escape되지 않은 공백이 있어 일반 텍스트 추출 결과를 믿을 수 없다.
font descriptor와 실제 내장 프로그램의 `5`, `6`, `)` 윤곽·hmtx를 메모리에서 비교하여 Gulim임을
확인했다. 이 글꼴 바이너리·재구성 글꼴은 커밋하지 않았다. PDF 숫자의 Widths 574와 TJ 74 보정은
표시 face의 자연 전진을 그대로 배치하지 않는다는 근거다. 이 진단값을 제품 상수나 px 핀으로 넣지 않았다.

- 새 문자군·언어별 상대 크기·장평 검사: 수정 전 **1 PASS / 2 FAIL**, 수정 후 **3 PASS**.
- 기존 본문 자간 비간섭 검사는 유지했다. 다른 기존 검사는 표시 face의 비양자화 자연 폭을 요구하여
  **1 PASS / 1 FAIL**이었다. Native/fresh WASM의 대상 95·96쪽 90% 이상과 확대 판독 뒤,
  실제 공통 paint 전진 배열·원본 소유·칸과 뒤 본문의 비겹침으로 교체하여 **2 PASS**했다.
- 두 파일 모두 `tests/cases/`의 관계 검사다. 최소 글꼴 95쪽이98% 미만이므로 절대 px를 고정하지 않는다.
- Studio TypeScript 검사는 통과했고 기존 Studio 검사 **2,784 PASS / 2 skip**를 확인했다.

## 같은 페이지·조건별 결과

| 조건 | 쪽 | Native | fresh WASM |
| --- | ---: | ---: | ---: |
| 최소 공개 글꼴 | 95 | 97.64481% | 97.64831% |
| 최소 공개 글꼴 | 96 | 96.08366% | 96.09842% |
| 기존 원본 글꼴 대조 | 95 | 99.99111% | 99.99111% |
| 기존 원본 글꼴 대조 | 96 | 99.98261% | 99.98261% |
| 독립 Print Gulim 공급 대조 | 95 | 99.99487% | 99.99487% |
| 독립 Print Gulim 공급 대조 | 96 | 99.98950% | 99.98950% |

최소 조건의 stage13 95쪽은 97.60552%/97.63529%였다. 같은 조건에서 개선됐지만 실제 대체 글리프의
윤곽 차이는 남으며 이를 픽셀 일치로 표현하지 않는다. Gulim은 PDF의 실제 프로그램을 확인한 별도
읽기 전용 [글꼴 공급 대조](print-gulim-reference-environment.json)다. 문서 원본·PDF·기본 대체 정책과
최소 조건을 변경하지 않았다. SVG도 같은 참조 칸·글리프를 직접 확인했으나 원본 조건의 전쪽 픽셀
결과를 Native와 같다고 주장하지 않는다.

- 최소 95쪽: [Native review](minimal-native-p095-review.png) · [overlay](minimal-native-p095-overlay.png),
  [WASM review](minimal-wasm-p095-review.png) · [overlay](minimal-wasm-p095-overlay.png)
- 같은 Print 공급 95쪽: [Native review](print-gulim-native-p095-review.png) · [overlay](print-gulim-native-p095-overlay.png),
  [WASM review](print-gulim-wasm-p095-review.png) · [overlay](print-gulim-wasm-p095-overlay.png)
- 같은 Print 공급 96쪽: [WASM review](print-gulim-wasm-p096-review.png) · [overlay](print-gulim-wasm-p096-overlay.png)

이 단계 증거는 위 집중 범위다. 새 source의 전체 회귀·9문서군 전쪽·편람 gate·정본 보호 TSV·
나머지 실제 소비 검증이 모두 완료됐다는 기록이 아니며 최종 검증에서 별도로 확정한다.
