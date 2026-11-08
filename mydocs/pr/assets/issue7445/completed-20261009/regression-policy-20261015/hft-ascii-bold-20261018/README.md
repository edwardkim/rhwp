# HFT ASCII 굵은 전진과 완료 페이지 위치 검증

Issue: #7445

## 독립 입력과 Print

- [독립 ASCII95 표본](independent-hft-ascii-styles.hwpx): 고딕·명조·산세리프 HFT의 일반/굵게/기울임/굵은 기울임,10/20pt를 별도 빈 한컴2020 HWPX에서 생성했습니다. 대상 문서의 본문·좌표·글리프를 복사하지 않았습니다.
- 입력 SHA256: `e5cf3c02d2a1dee00e5f4c8ac48e2ca5f20f75e5c82e4852f77062d972278b02`.
- [독립 한컴2020 Print PDF](../../../../../../../pdf/issue7445-independent-hft-ascii-styles-20261018-2020.pdf):6쪽, SHA256 `a7e38c8407056cb6741331ceea64168043fe6c4f939ffe883a02ef1aca9795e6`.
- MCP job `259d5492-6979-4d10-8aa7-9d2b168c70a2`, engine2020, Hancom11.0.0.9136, method0, `hancom2020_pdf_driver_one_up`, timeout1800초입니다.

세 face의 일반·굵게 Type3 `/Widths`는 ASCII95자 모두500/1000em입니다. 굵은 비공백의 원점만 약0.05em 더 전진합니다. 글리프 자체를 이 추가 간격으로 확대하지 않고 공통 측정 결과의 실제 tracking 기여를 paint에서 분리합니다. 실제 TTF 선택·미확인 기울임·다른 글꼴에는 새 규칙을 적용하지 않습니다. 독립 페이지를 직접 렌더하여 글자·굵기·기울임이 보이는 것을 확인했습니다. 이 PDF의 CharProcs만으로 fill-only 윤곽을 복구하는 진단은 거부됐으며, 그 결과를 글꼴 공급이나 outline 비교 통과로 사용하지 않았습니다.

## 현재 출력 검증과 회귀

C144 Native/fresh WASM의 원본 sample16 전64쪽 최저94.38750/94.46126%, sample11 전151쪽 최저90.08879/90.09923%, 각각 미달0쪽입니다. sample11은 공개 Batang을 명시한 caller font environment이며 제품의 자동 대체 선택 완료를 뜻하지 않습니다. 100쪽은88.89361/88.81174%에서97.61437/97.55750%로 개선됐고2/50쪽은 이전 점수를 유지했습니다. 코드 수정 전 metric 진단1FAIL과 수정 후 세 face의 PASS를 별도로 보존했습니다.

sample16의33쪽은 Native99.57555/WASM98.83569%이며, 독립 Print의 글머리표 원점194.76/129.48pt와 뒤 SUN 본문의209.28pt를96dpi CSSpx의±2px로 검사합니다. 원본 표537과 본문 내용으로 대상을 선택하며 화면 영역으로 고르지 않습니다. 같은 정식 새 검사를 최신 devel8766의 실제 Native library와 현재 후보 library에 연결해 실행했습니다. baseline149.61333CSSpx/독립172.64CSSpx에서 실제1FAIL, 현재 후보에서는1PASS입니다. 기존 표 소속·빈 줄·내용 검사도 보존합니다.

원시 로그·중간 JSON·전체 PNG/TSV는 ignored `output/pr-review/7445-completed-20261009/rebase-8766fb47-20261018/`에 보존합니다. 전체 회귀·48조건 base/head 비교·실제 소비 경로는 아직 별도 진행 중이며 위 수치로 대신하지 않습니다.
