# HFT 본문 곱셈 전진폭 — C52

**부분 보정·PR 보류 상태**다. 기준 devel은 `f5251e7d`, 수정 전 head는 `8899309b`다.

## 독립 근거와 구현

원본115쪽의83쪽, 과거·현재 분할본197쪽의140쪽에서 실제 `×` 글리프를 확인했다.
세 독립 한컴2024 Print의 글리프 프로그램 SHA와 `/Widths=616/1000em`이 같고,
원본 해당 슬롯은 HFT 한양신명조다(측정·PDF SHA（로컬 중간 자료 `independent-multiplication-width.json`）).

기존 overlay는 이 기호에서 대체 H2MJSM의 전각 폭을 빌려16px로 전진했다.
확인한 기호만 기존1024em 표의631로 보충했다. 공통 메트릭 → 장치 전진 → 줄 폭·
양쪽 정렬 → TextRun 상자·문자 위치 → Native/Canvas가 같은 값을 소비한다.
문서·페이지·일치율 분기는 추가하지 않았다. 별도 HY신명조·휴먼명조·굴림체·바탕체·
Palatino는 그대로다(반올림된 측정 API 대조（로컬 중간 자료 `font-controls.json`）).

47조건의 원문 슬롯 조사（로컬 중간 자료 `scope-faces.tsv`）에서 이 기호에 한양신명조가 선택되는
입력은 이번 세 입력이다. 이 조사를47조건의 현재 전쪽·소비 경로 통과로 보고하지 않는다.

## 검증

| 같은 원본·독립 PDF | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 쪽수 |
|---|---:|---:|---:|
| 원본 | 90.74868% | 90.63175% | 115/115 |
| 과거 분할본 | 90.24017% | 90.24017% | 197/197 |
| 현재 생성본 | 90.24017% | 90.24017% | 197/197 |

각509쪽·합계1018쌍을192DPI·2px 관용·현행 정본 지표로 다시 산출했다. 여섯 TSV를
이 폴더에 보존한다. C51 대비 각 입력의 곱셈3쪽씩만 개선, 총18개 페이지/backend쌍
+0.37239~+1.30633%p이며 나머지는 같다(전후（로컬 중간 자료 `before-after.json`）). 원본83쪽은
양쪽96.43509→97.34529%다.

새 원본 비율·본문 소속 검사는 frozen C51 실제FAIL→C52 정식 Cargo2PASS(기존
수식 검사 포함)다. TextRun 상자와 paint 전진, 원문·치수 순서도 검사한다. 해당 쪽은
98% 미만이어서 새 px 핀을 추가하지 않았고 기존197쪽 보호는 유지한다.
Native build·Mac locked web/no-opt 대체 빌드·root/Studio JS/WASM hash 일치,
manifest·unit-tier·fmt·all-target Clippy를 확인했다. Docker 최적화 통과는 아니다.
전체10440건 회귀는 미실행이며 앞선10438PASS는 C51/C52 이전 결과다.

## 직접 판독·잔여

- [Native 원본83쪽 review.png](native-giant4138-original-review_083.png)
- [Native 과거 분할140쪽 review.png](native-giant4138-tab-period-review_140.png)
- [fresh WASM 원본111쪽 review.png](wasm-giant4138-original-review_111.png)
- [fresh WASM 현재 분할191쪽 review.png](wasm-giant4138-current-ascii-review_191.png)
- [같은 영역의 수정 전·후·독립 출력 확대](multiplication-before-after-detail.png)

곱셈 뒤 치수·본문 간격은 개선됐고 확인 영역의 내용·줄 소속을 유지했다. 대체 곱셈의
세로 모양·HFT 윤곽과 기타 수식 미세 차이는 남는다. 정식 글꼴 예외나 수식 fidelity
완료로 판정하지 않는다. 처음 고른 분할142쪽은 변경 없는 대조군이며 실제140쪽으로
근거·그림을 정정했다. 빌드 뒤 두 주석의 쪽수만 바뀐 기록（로컬 중간 자료 `comment-equivalence.json`）을
실제 빌드 source와 구별한다.

최신 고정 devel 대비 Native 원본·과거 분할의0.01%p 이상 하락은0쪽이지만 WASM
원본60쪽·과거 분할7쪽은 하락이 남는다. 현재 생성본의 base는 쪽수 불일치로 보류다
(페이지 목록（로컬 중간 자료 `latest-base-comparison.json`）). 법원 서식 미달·수식 확대 차이·나머지
최종 제출 검증과 실제 소비 경로36건도 잔여이며90% 통과로 면제하지 않는다.

추가 출처 감사28건은 현재18PASS/10FAIL, 고정 base17PASS/11FAIL이고 현재 실패
이름은 base의 부분집합이다. 과거 overlay 모집단·문서 링크에 관한 실패를 통과로
보고하지 않는다. 명령·해시·실패·잔여는 validation.json（로컬 중간 자료 `validation.json`）에 있다.
이전 글자 위치 재생·PDF 인쇄 격자 실험은 진단만 보존했으며 일반600DPI 상수나
kerning 우회를 production에 넣지 않았다.
