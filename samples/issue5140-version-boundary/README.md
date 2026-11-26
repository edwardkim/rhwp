# 구형 한컴 특수 문자 중 선 기호 15종의 재현 자료

HWP 5.0 형식의 문서 중 파일 형식 버전이 5.0.1.5 이하인 문서의 본문 기호 변환과 현대 문서의 문자 보존을 재현한다. 두 HWP는 본문과 서식이 같고 FileHeader 버전 한 바이트만 다르다. 대응 HWPX는 각 원본을 한컴오피스에서 직접 저장한 독립 기준 출력이다.

## 파일과 출처

| 파일 형식 버전 | 원본 HWP | 한컴 HWPX | 한컴 인쇄 PDF |
| --- | --- | --- | --- |
| 5.0.1.5 | [HWP](legacy-symbols-v5.0.1.5.hwp) | [HWPX](legacy-symbols-v5.0.1.5.hwpx) | [PDF](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.5.pdf) |
| 5.0.1.6 | [HWP](legacy-symbols-v5.0.1.6.hwp) | [HWPX](legacy-symbols-v5.0.1.6.hwpx) | [PDF](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.6.pdf) |

HWP는 rhwp로 만든 합성 문서다. 각 파일은 5,120바이트이며 파일 오프셋 1,568의 버전 값만 다르다. HWPX와 PDF는 Hancom Office Hangul 12.30.0.6544 macOS에서 저장·인쇄했다. 개인 문서, 원시 글꼴 파일이나 앱 바이너리는 포함하지 않는다.

## 입력 구성

각 문서는 한 구역, 설정용 빈 문단 하나와 기호 문단 15개로 이루어져 있다. 본문은 함초롬바탕 10pt이며 한 페이지다. 기호 문단은 아래 형태로, 기호 바로 뒤의 `B`부터 굵게 표시한다.

```text
U+A806: A<기호>B
NEXT
```

```text
A806 A807 A808 A80C A80E A80F A810 A811
A813 A817 A81A A81C A827 A832 A848
```

## 한컴 기준 기대값

| 항목 | 5.0.1.5 | 5.0.1.6 |
| --- | --- | --- |
| 기호 코드 | 15종 모두 Plane-15 PUA로 변환 | 원래 BMP 코드 보존 |
| 굵은 `B`의 UTF-16 위치 | 11 | 10 |
| `NEXT`의 UTF-16 시작 위치 | 13 | 12 |

5.0.1.5 재현 문서에서 관측한 변환식은 `0xF0000 | (input & 0x0FFF)`다. 기호의 UTF-16 길이가 1에서 2로 늘어나므로 뒤의 경계도 한 코드 유닛씩 이동한다. 두 출력에서 서식과 줄 시작이 가리키는 논리적 문자 위치는 각각 10과 12로 같다.

이 기대값은 나열한 본문 기호에 대한 것이다. 전체 `A000–A8FF` 구간, HWP3 또는 글자 겹침의 변환 규칙을 정의하지 않는다.

## 회귀 검사 방법

`tests/cases/issue_5140_versioned_body_symbols.rs`는 전체 본문을 한컴 HWPX와 비교하고 다음을 검사한다.

- 현대 HWP와 두 한컴 HWPX의 문자 보존
- 기호 뒤의 굵은 서식·둘째 줄 위치 보정
- 반복 저장과 실제 편집 후 HWP/HWPX 교차 저장의 문자 보존
- 의도적으로 손상한 서식·줄 위치를 일반 문서 검증이 검출하는지 여부

생성 HWP에는 저장된 줄 정보가 없다. 줄 위치 검사는 현대 한컴 HWPX의 줄 정보를 같은 BMP 본문에 적용한 뒤, 5.0.1.5 HWP의 한컴 HWPX 저장본의 위치와 비교한다. 원본 HWP를 직접 사용하는 CLI 검사와 이 회귀 입력을 구분한다.

문자·위치 대조, 실행 환경과 인쇄 결과는 [검증 보고서](../../mydocs/verification/issue5140-version-boundary/README.md)에 있다.

## 파일 무결성

| 파일 | SHA-256 |
| --- | --- |
| legacy-symbols-v5.0.1.5.hwp | bd45ac8a92eefc77695a0800130215452cd83f9a2f312e41d4c09385e31857f2 |
| legacy-symbols-v5.0.1.5.hwpx | bbe09129f125b9ffda2b583b8e81a3cec67b64da48df8c0d0fd83729d619e800 |
| legacy-symbols-v5.0.1.6.hwp | 7ccb9736edbfbd727c8fdcaa4eff44f4e196168254b3f22c539193e4d78f79d3 |
| legacy-symbols-v5.0.1.6.hwpx | b036334d8a6072a8c8e39b829b4d150617b8b2eb5592181f5fc3a10dd1c7041f |
