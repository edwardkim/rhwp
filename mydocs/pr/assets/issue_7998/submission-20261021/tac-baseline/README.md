# tac-img-02 기존26·45·53쪽 보정

제품 `d1d3091c5`, 최종 테스트 head `168e96b86`에서 검증했다.
메인테이너가 본 폴더의 세 쪽 비교와 실제 Studio7793에 **시각 판정 통과**를 주었다.
모든 쪽은96dpi·compat2022·PNG backend다. 독립 PDF는 한컴2020 Print다.

| 입력 | 저장소 위치 | SHA-256 |
|---|---|---|
| HWP 원본66쪽 | `samples/tac-img-02.hwp` | `f8d5b42367363de0bfde553c062e8a632c222e076a52c95b36efb490b743008e` |
| 독립 Print66쪽 | `pdf/tac-img-02-hwp-2020.pdf` | `f5b56bc65f796acafe3075aadd714ec96d2e508f862dfd899ec5e5081d03e1cc` |

| 쪽 | Native raw % | fresh WASM raw % | 확인한 경계 |
|---|---:|---:|---|
| 26 | 100 | 100 | 빈 쪽의 쪽번호 표시·기준선 |
| 45 | 99.76853 | 99.81509 | 선행 공백 줄·표·빈 줄·다음 표 |
| 53 | 99.97208 | 99.97212 | 표·제목·다음 표의 순서와 비겹침 |
| 전체66쪽 최저 | 96.39090 | 95.51053 | 누락·전체 쪽수 차이0 |

[Native66쪽 원문 TSV](native-all-66.tsv) · [fresh WASM66쪽 원문 TSV](wasm-all-66.tsv).
`*-review_026.png`, `*-review_045.png`, `*-review_053.png`는 같은 쪽의 rhwp·PDF·overlay다.
`*-overlay_*.png`도 별도로 보존한다. 표지의 번호 감추기와61쪽의1/62쪽의2 재시작도 확인했다.

## 규칙과 회귀

뒤쪽 `NewNumber` 존재로 앞쪽의 명시적 `PageNumberPos`를 숨기던 상태 처리를 바로잡았다.
굴림 쪽번호의 원본 SFNT hhea 하강145/1024em을 공통 메트릭으로 소비하며, 미수록 프로그램과
HFT의 기존 계약을 유지한다.45·53쪽은 유효한 저장 줄 전체가 닫힌 점유를 이루는 경우
이미 그린 선행 줄을 표 줄만의 상한으로 되감지 않는다. 변경/분할/무효 저장 정보는 비적용이다.

실제 CLI의66쪽 render tree를 읽는 두 정식 관계 회귀는 보정 전 각각 의도한 원인으로 실패하고
보정 후 통과했다.26쪽만 양쪽 raw100%·직접 확대 판독에 근거해 독립 PDF798pt 기준선을
96dpi±2px로 보호한다.45·53쪽은 내용 소유·순서·포함·비겹침으로 검사한다.
CI는 로컬 상용 글꼴을 새 설치 조건으로 요구하지 않는다.

## 실제 글꼴 공급 추가 대조

최초 검증에는 HCRBatang·Arial·굴림face0·맑은고딕 regular를 명시 공급했다. 추가로 실제
맑은고딕 bold를 공급하되 기존64MiB catalog 한도를 유지했다. HCRBatang은 원본 전체
문단/제어의 문자열·한글·Latin·기호에 필요한 glyph를 fontTools로 부분 집합화했다.
선택한15,679자의 원본 advance를 모두 확인했고 이름·layout·원본 hhea를 유지했다.
굴림은 TTCface0를 TTF로 추출했다. 상용 글꼴 프로그램은 저장소에 포함하지 않는다.
[프로그램 크기·SHA·추출 범위](font-programs.json)는 로컬 공급 출처다.

이 추가 공급은 글꼴 선택에 따른 RGB 차이가 있어 최초 PNG와 동일하다고 표시하지 않는다.
새 전쪽 출력의 최저는 동일하게96.39090/95.51053%이며 모든 쪽이90% 이상이다.
추가 공급 상태의26/45/53쪽은 Native100/99.76863/99.97049%,
WASM100/99.81488/99.97200%다.53쪽 제목의 실제 bold를 같은 배율로 직접 대조했다.

- [추가 공급 후보 Native66쪽](candidate-native-complete-font-all-66.tsv)
- [추가 공급 후보 fresh WASM66쪽](candidate-wasm-complete-font-all-66.tsv)
- [같은 공급 기준 Native66쪽](base-native-complete-font-all-66.tsv)
- [같은 공급 기준 fresh WASM66쪽](base-wasm-complete-font-all-66.tsv)

기준은 `5e2b9bfb`의 동결 producer이며 통합 devel `2e3b4f880`까지의 차이는
Qt/Native clipboard7파일이다. 이 문서의 렌더링 경로는 같다. 기준은 양쪽26·45·53쪽이
미달이며, 같은 공급의 후보에서0.1%p 이상 하락은 없다.
실행은 `visual_sweep.py --silhouette-only --rasterizer png --compat 2022 --dpi 96
--hwp samples/tac-img-02.hwp --pdf pdf/tac-img-02-hwp-2020.pdf --embed-fonts full`을
명시한 공급 경로/동결 producer에 적용했으며 WASM에는 대응 `--wasm-pkg`를 지정했다.
