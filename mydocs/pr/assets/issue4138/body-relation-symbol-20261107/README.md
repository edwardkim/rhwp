# 본문 관계 기호의 원본 HFT 전진 보존

부모 `1c23f8e2b65977d376bd5cfa7cabe4f90bda0f1d`, base
`a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에 공통 글리프 폭 보정을 적용했다.
제품 소스·회귀 파일·Native 실행 파일·fresh WASM 해시는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/body-relation-symbol-20261107/validation.json`)에 있다.

입력은 `mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp`
(SHA-256 `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`)이며,
[기존 독립 한컴 2024 Print PDF](../../../../../pdf/issue4138-ascii-split-20261016-2024.pdf)를 재사용했다.
두 출력 모두 PDF와 같은 197쪽이다. 192 DPI·Print·compat 2024·PDFium·raw 2px 실루엣으로 비교했다.

## 원인과 소비 경로

21쪽 `f≤1`은 수식 개체가 아닌 본문 HFT 문자다. 표시용 환경이 `한양신명조`를
`HY Sinmyeongjo`라는 실제 family 이름으로 해소하면, 부분 HFT 폭 표에 없는 `≤`를
반각으로 추정해 1em에서 0.5em으로 줄였다. 독립 PDF의 T4·U+2264는
FontMatrix `.001`, `/Widths`와 CharProc `d1` 모두 1000이다. 확인한 원본 일반체
글리프 한 개의 자연 전진을 기존 부분 폭 표에 추가했다. 글꼴 파일이나 윤곽은 포함하지 않는다.

`declared_original_glyph_advance_em`의 원본 프로그램 hit를 공통 문자 측정이 먼저 소비하며,
같은 전진으로 줄 구성·TextRun 상자·뒤 본문의 원점·paint 위치를 계산한다. 문서나 페이지를
구분하는 제품 분기, 표시 글리프의 이동·확대, 문서의 글꼴 선언 변경은 추가하지 않았다.
측정한 원본 HFT 일반체만 대상으로 하며 명시적 대체 TrueType의 전진은 이 표로 덮지 않는다.

원 공급·측정 경계는 #8397(`99a176b7cb`)의 HFT 원본 보존 계약과 현재 #8696의
정상 단일 원본·명시 TTF 대체 계약을 대조했다. #8696의 목록 이름 처리는 유지했다.

## 실제 검사와 전쪽 비교

새 정식 관계형 2건은 부모에서 1 PASS·1 FAIL, 보정 후 2 PASS다.
원본 HFT 대조와 표시 별칭 대조에서 같은 1em 점유·원문 `f≤1`·글꼴 선언 보존을 검사한다.
절대 화면 좌표를 고정하지 않는다. 앞 단계 조판·글꼴·별칭 경계를 포함한 최소 글꼴 집중
묶음은 89 PASS이며, 두 원형 검사를 정식 모듈로 옮긴 뒤 다시 실행한 결과가 위의 2 PASS다.
이 수치를 91개의 고유 검사로 합산하지 않는다.

| 경로 | 21쪽 수정 전 | 21쪽 수정 후 | 전197쪽 최저 | 98% 미만 |
| --- | ---: | ---: | ---: | --- |
| Native | 95.85474% | 97.42188% | 94.83034% | 21·44·129·130쪽 |
| fresh WASM | 95.83624% | 97.39579% | 94.98570% | 21·44·129·130쪽 |

전197쪽을 각 경로에서 다시 출력했다. 두 경로 모두 21쪽만 PNG가 달라졌으며 나머지
196쪽은 부모와 바이트가 같다. 누락 0쪽·쪽수 불일치 0건·raw 하락 0쪽이다.
[수정 전 Native TSV](before/native-silhouette.tsv) · [수정 후 Native TSV](after/native-silhouette.tsv)
· [수정 전 WASM TSV](before/wasm-silhouette.tsv) · [수정 후 WASM TSV](after/wasm-silhouette.tsv).

Native의 자동 Noto 공급 캐시 TTF 해시 차이도 발견했다. 같은 환경 JSON과 같은 현재 글꼴
파일 바이트를 고정해 부모/후보의 21쪽을 별도로 다시 출력했고, 양쪽 모두 각각의 전쪽 PNG와
바이트가 같았다. 이 재검사의 부모/후보 공급 글꼴 목록·해시도 서로 같았다. WASM의 기존
전쪽 부모/후보 공급 기록은 처음부터 동일하다. 캐시 차이를 코드 개선으로 세지 않았다.

fmt·workspace all-target Clippy·manifest·unit tier 검사를 통과했다. WASM은 Mac의
locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화나
실제 Studio 초기화 응답 검증으로 보고하지 않는다. 전체 nextest·최종 게이트 완료도 아니다.

## 직접 판독과 잔여

`≤` 앞뒤 본문의 원점이 회복된 것을 Native/fresh WASM review와 overlay로 직접 확인했다.
그러나 같은 쪽 3.4.3 마지막 줄의 공백·기호 윤곽 차이와 44·129·130쪽 수식 간격이 남아 있다.
**21쪽도 아직 98% 미만이며 브랜치 전체 완료·PR 제출 가능으로 판정하지 않는다.**
별도의 독립 85자 수식 글리프 공급 진단은 이 75자 공급 조건의 전쪽 결과와 섞지 않았다.

| 경로 | 수정 전 review | 수정 전 overlay | 수정 후 review | 수정 후 overlay |
| --- | --- | --- | --- | --- |
| Native | [review](before/native-p021-review.png) | [overlay](before/native-p021-overlay.png) | [review](after/native-p021-review.png) | [overlay](after/native-p021-overlay.png) |
| fresh WASM | [review](before/wasm-p021-review.png) | [overlay](before/wasm-p021-overlay.png) | [review](after/wasm-p021-review.png) | [overlay](after/wasm-p021-overlay.png) |

실행 로그·컴파일용 묶음·독립 글꼴 표본·글꼴 바이너리는 ignored
`output/open-issues-oldest-20261105/post-rebase-ece9/rebase-8696-20261106-224117/`에만 보존했다.
