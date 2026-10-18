# 개인정보 조문 대비표의 대시·낱말 공백 보정

Issue: #7445

## 독립 근거와 원인

원본 `samples/task2097/75544_pii_bunseok.hwpx`와 한컴2020 Print
`pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`의 동일66쪽을 사용했다.
정확한 입력/PDF/생산자 SHA는 [provenance.json](provenance.json)에 기록했다.
4쪽 오른쪽 셀의 `공개방법 등) ① ---------` 줄은 왼쪽 셀과 같은 글자/문단
스타일을 사용하지만, 기존 양쪽 정렬은 여유 폭 전부를 대시에 배분했다.
실제 Canvas trace의 기호 원점539.56CSSpx는 독립 Print544.96CSSpx보다5.4px
왼쪽이었다. 같은 HYSinMyeongJo-Medium 글꼴이며 대체 폰트 추정으로 처리하지 않았다.

독립 Print 대시 전진은 약0.834em으로 자연 전진 범위이고 내부 공백은 약0.65em이다.
기존 대시0.884em/공백0.5em 배분을 보정했다. 런마다 실제 스타일로 자연 대시 복원
한도를 구하고, 남은 양쪽 정렬 여유를 낱말 사이에 배분한다. 측정 계획이 정규화한
런 순서와 같은 결과를 실제 방출이 소비하며, 필드와 선행 들여쓰기 공백을 구분한다.
문서 ID나 특정 쪽 좌표 분기는 추가하지 않았다. 마지막 줄/공백 없는 줄의 기존
별도 계약은 바꾸지 않았다.

## 검증 범위와 결과

정식 `issue_7445_dash_leader_word_spacing` 회귀는 원본 셀/문단 소유와 내용 순서,
누락·중복, 항목 기호 한 번, 대시·일반 공백 전진 관계를 검사한다.
수정 전1FAIL→수정 후1PASS이며 Native Skia 경로도1PASS다. 보정 전에는 알려진 잘못된 위치를 절대 px로 고정하지 않았다.
C253 보정 후4쪽 양쪽98% 이상·직접 판독과 전쪽90%를 확인한 뒤 중요한 `①`의
독립 Print 원점(544.96,193.28)CSSpx를 각 축±2CSSpx로 보호했다. 원본 소속/내용
관계 검사도 유지한다. 이 좌표 추가 후 기본/Native Skia 집중 회귀는 각각1PASS다.
전체10,938PASS는 이 assertion 추가 전이며 runtime source는 동일하고 최종 head 검증은 별도다.

| 전쪽 조건 | 쪽수 | raw 최저 | 90% 미달 | 고정 devel 대비0.01%p 이상 하락 |
| --- | ---: | ---: | ---: | ---: |
| Native 선언 글꼴 | 66 | 94.12098% | 0 | 0 |
| Native 기본 글꼴 | 66 | 94.12098% | 0 | 0 |
| fresh WASM 선언 글꼴 | 66 | 93.77937% | 0 | 0 |
| fresh WASM 기본 글꼴 | 66 | 93.77937% | 0 | 0 |

원본 전체66쪽과 양쪽 출력/조건의 총264쪽 TSV를 첨부했다. 4쪽은 Native
99.14237→100.00000%, fresh WASM98.08467→98.89537%다. 바뀌지 않은 다른
페이지의 최저값을 개선한 것으로 기재하지 않는다. raw는 색상 경계 보정 전2px
내용 실루엣 지표이고100%라도 픽셀 동일이라는 뜻은 아니다.

- Native: [review](native-p004-review.png) · [overlay](native-p004-overlay.png)
- fresh WASM: [review](wasm-p004-review.png) · [overlay](wasm-p004-overlay.png)

양쪽 review·overlay를 직접 판독했다. 오른쪽 기호·대시 시작 위치와 원본 소속은
개선됐으나 밑줄·표 선 및 잉크/AA 잔차가 남아 있다. 관련 없는 네 페이지는 양쪽
backend에서 보정 전 C240 PNG와 RGBA가 동일했다.

Native Skia lib3862PASS/12ignore와 다른 라이브러리182PASS, fmt·Native/WASM/
workspace all-target Clippy, Native 정식 회귀1·공개 계층25·장식 trim3건이 통과했다.
C258 전체 nextest는10,938PASS/0FAIL/45skip, 검사 실행405.452초로 완료했다.
이 작업 소스의 전체 회귀 결과이며 최종 커밋 head/전쪽 시각 통과나 PR 준비 완료는 아니다.
Mac WASM은 locked wrapper의 web/no-opt 로컬 대체 빌드다. Docker 최적화 통과가 아니다.

고정 base는6ed1682c3267ea786734eb2e75252916a861d1fd, producer의 parent는
0666befe5b8fbac7c5464ac654dbaf672054d016이다. 이 증거는 C253 working source와
hash에 대한 것이며 최종 커밋 head의 전52조건/actual36 검증을 대신하지 않는다.
다른 문서의 devel 대비 시각 하락은 남아 있으므로 PR 생성·승인·통합은 계속 보류한다.
