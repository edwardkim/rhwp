# PR #8896의 실제 글꼴 공급 복원

2026-11-11~12, 작업지시자의 **기존 차이도 예외 없이 해결** 결정에 따른 검증이다.
정본 보호 TSV·판정 허용치·대상 문서·쪽수는 바꾸지 않았다. 최종 전 3,542쪽의 공식 두 자리
보호값 하락은 0이며, 상세 값은 [전쪽 비교](../protection/comparison.tsv)를 따른다.

## 원인과 공통 보정

| 범위 | 독립 근거·원인 | 처리 |
| --- | --- | --- |
| Native 다수 문서 | 첫 HFT 폴더만 387개인데 실제 snapshot 적재 상한은 256개였다. 파일 목록 1,055개가 실제 적재 1,055개를 뜻하지 않았다. | 기존 문서 선언·명시 대체 가족의 원 프로그램을 같은 순서·style로 공급했다. 한도나 메트릭을 조정하지 않았다. |
| sample16의 `○` | 세 독립 Print의 Type3 윤곽과 로컬 Myeongjo 원 프로그램의 U+25CB가 정규화된 모든 피연산자에서 같다. | 결측 글리프 대체 가족을 명시했다. [hash 증거](circle-contour-equivalence.json). |
| 중첩 표 8·9쪽의 작은 삼각형 | 원문 U+F02FB와 독립 Print의 Haansoft Batang 프로그램 윤곽·전진이 같다. 기존 지원 목록에 이 PUA가 빠졌다. | 기존 원 PUA 지원 목록 끝에 추가했다. 원 글리프가 없으면 기존 Unicode 대체를 유지한다. [hash 증거](triangle-glyph-equivalence.json). |
| 구형 분수 | 준비 도구의 실제 가족은 `RHWP Hancom Legacy Equation`인데 layout은 다른 구형 이름만 요청했다. | 공통 수식 family chain에서 준비 도구 가족을 먼저 요청하고 기존 구형·현대 fallback을 유지했다. |
| sample16 43쪽의 `×`·`÷` | HCI Poppy의 결측 문자가 Noto로 대체됐다. 독립 문자 표본의 실제 한컴 Print 프로그램과 목표 Print의 두 프로그램이 바이트까지 같다. | 원래 윤곽·전진 그대로의 독립 구형 공급을 명시했다. [증거](arithmetic-symbol-equivalence.json). |
| Native sample16 35쪽 WMF | 사용자 snapshot의 GulimChe가 본문·그림자에는 전달되지만 WMF 내부 paint에는 전달되지 않았다. | 기존 WMF 요청·등록·자원 예산에 동일 snapshot을 공급한다. 그림자와 실제 face 선택 루프를 공유하고 중복 등록을 억제한다. |

`HYHWPEQ.TTF`는 메인테이너가 제공한 실제 파일을 명시 공급했다. 가족은 HyhwpEQ이며
`×`·`÷`도 있지만, 이 TTF의 전진은 각각 831/1024이고 독립 구형은 616/1000·623/1000이다.
설치 여부만으로 서로 같은 프로그램으로 간주하지 않는다. [제공 파일 확인](provided-hyhwpeq.json).

## 선행 해결과 현재 공급 경계

- 원 PUA의 Unicode 대체는 #4224 → PR #4228(`30bad7c1d4…`)에서 시작했다.
  실제 SFNT 원 프로그램의 전진·표시 연결은 `774d414cd9…`와 PR #8188의 `17225375ff…`를
  확인했다. 이번 수정은 그 공통 목록에 빠진 작은 삼각형을 추가하며 기존 결측 대체를 유지한다.
- 구형 수식 layout은 PR #8188(`17225375ff…`), 독립 준비 도구는 PR #8319의
  `315de5dcb3…`에서 도입됐다. 후속 `00c2485117…`의 공통 줄 프레임은 보존하며,
  서로 다른 공급 가족 이름만 호환 chain으로 연결한다.
- WMF Print와 실제 글꼴 등록의 공통 API는 PR #8064의 `20af4ecd67…`, Native PNG의
  snapshot은 PR #8565의 `5551091085…`, 실제 그림자 face 선택 루프는 PR #8889의
  `050e78455e…`를 대조했다. 기존 요청·등록 결과를 PNG exporter도 소비하게 연결했다.

위 도입 commit의 GitHub PR 연결·실제 코드와 독립 Print 근거를 대조했다.
기존 해결을 삭제하거나 원본 glyph·메트릭·줄 프레임을 별도 backend 규칙으로 대체하지 않는다.

## 독립 문자 표본 재현

`scripts/prepare_legacy_equation_font.py`의 기존 v1은 84개 probe·75개 Type3 문자를 유지한다.
선택적인 `--include-arithmetic-operators`는 v2의 86개 probe·77개 Type3 문자로 `times`·`div`를
추가한다. `probe → 실제 한컴 2020 Print(method 0, one-up) → recover`를 실행했다.
정확한 CLI는 `python3 scripts/prepare_legacy_equation_font.py --help`와
[글꼴 사고 대응](../../../../manual/font_incident_response.md)을 따른다.

- v1 독립 작업: `1ff6c3ab-14bc-4237-91c8-fbcd6653cdc6`.
- v2 독립 작업: `c3b442f7-0bea-4d55-a7ea-93d6eec3c2bf`.
- 엔진: 한컴 2020 `11.0.0.9136`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`.
- canonical 도구가 만든 v1/v2 입력 ZIP은 각각 앞서 실제 출력한 입력과 바이트가 같다.
  양쪽 실제 Print를 새 도구로 recover했고, v2 `×`·`÷`의 프로그램 SHA가 목표 Print와 같다.
- 검증 대상 PDF에서 글리프를 추출해 공급하지 않았다. 이동·확대·윤곽 수정 없이 독립 문자 표본을
  사용했다. 다른 기호 모두가 목표 PDF와 같다는 주장은 하지 않는다.

글꼴 프로그램·윤곽 원문·중간 probe PDF·서비스 인증 로그는 로컬 검증에만 사용한다.
저장소에는 hash·provenance·환경 설정·실제 문서의 TSV와 판독 이미지만 보존한다.

## 진단과 최종 증거의 구분

탐색용 `<160` 회색조 마스크는 정본의 최소 RGB `<232`와 다른 위치를 지목했다.
그 결과로 세운 hinting·Gulim 획 가설은 최종 원인 판정에서 제외했다. 해당 제품 변경은 없다.
최종 값은 정본 CLI의 색상 경계 보정 전 2px raw다.

2개 SEO 문서는 범용 축소 목록이 문서의 Arial 대체 정보를 누락해 원래 공급 전체를 유지했다.
최종 집계는 두 **데이터셋 전체**를 원 환경의 재검증 결과로 선택한다. 쪽별 최댓값 선택이나
공식 기준 갱신을 하지 않았다. 초기 미달 결과는 `protection/initial-comparison.*`에 구분했다.

[확대 판독](direct-review/)은 Native·fresh WASM·독립 Print의 같은 영역을 같은 배율로 대조한다.
중첩 표의 삼각형·주변 본문, WMF 글자, 곱셈·나눗셈, 분수의 괄호·분자·분모를 확인했다.
raw 100%는 RGB 완전 동일을 뜻하지 않는다. 안티앨리어싱의 색상 차이는 별도다.

## 자동 회귀

기존 `native_pua_host_font_protocol` suite에서 원 PUA의 요청·등록·해제 fallback과
WMF 전용 가족의 실제 Native PNG를 검사한다. WMF는 저장소 소유 MIT 합성 TTF를 고유 가족으로
격리해 수동 등록 대조군·미공급 음성 대조군·자동 공급·중복 등록 억제·저장 원본 보존을 확인한다.
시스템 글꼴 설치 여부에 의존하지 않는다. `issue_8896_legacy_font_family_contract`는 실제 공개
sample16 입력의 분수 SVG가 canonical 준비 도구와 같은 첫 가족을 요청하고 원 IR을 보존하는지 검사한다.
새 source-side unit-test 모듈을 늘리지 않고 기존 자동 수집 integration 경로를 사용한다.
