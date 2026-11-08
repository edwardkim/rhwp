# 단계 39: 한점 리더의 확인된 glyph 공급 프로그램 복구

## 독립 원인 근거

- parent `e7b50eb8b0c731f676f26b7c4fdb42096a62a268`, base `2e087a5b4bc1515ad877e8020b30114639a59392`.
- 이미지 스택3쪽 제목의 원문 U+2024는 HYHeadLine-Medium으로 선언돼 있다. 설치된 H2HDRM.TTF(SHA e50c7200521c3ea59840e5dd42e79708905a74a607f2300ed862fe2491caf831, UPEM1024) cmap에 그 glyph가 없음을 실제 파일로 확인했다. 단순 DB 미수집을 glyph 결측이라고 추정한 것이 아니다.
- 독립 한컴2020 Print의 양각 세 text replay 모두 이 문자만 Haansoft Batang을 사용한다. 다음 문자 원점 차이는3.119995pt, 약4.159993CSSpx다. 현재 HBATANG.TTF의 같은 glyph 전진178/1024는 기존 등록 데이터와 같다. 새 글꼴·폭 배열·윤곽을 저장소에 추가하지 않는다.
- 원 프로그램 결측을 기존 `verified_program_has_glyph`, 공급자를 `verified_missing_glyph_supplier`와 공통 paint fallback 조회에 연결했다. 확인한 정체 regular U+2024만 결측 데이터에 추가한다. 다른 문자·weight·italic의 cmap 부재를 추정하지 않는다. HFT·실제 metric hit·명시 공급자의 기존 경계는 유지한다.
- 생산의 동일 char-width 질의 → 줄 구성/정렬 → emitted run → Native paint가 기존 공급 프로그램의 자연 전진과 원본 자간을 소비한다. 특정 페이지의 x 이동, 자간 덮어쓰기, 출력 clamp는 없다.

## 실제 검증

- 같은 원문/style(font26.666667px, 장평1, 자간-10%)의 한점 리더 run width는23.936790px(추가 압축 전24px)에서4.16px로 복구됐다. 독립 Print와 일치하며 뒤 원문을 바꾸지 않았다. [전후 기하](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/glyph-before-after.json), [설치 파일 coverage](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/installed-glyph-coverage.json), [PDF 관측](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/pdf-dot-advance.json).
- 기존 #4962·#7092·#6802, 원본 프로그램/슬롯 소유, 대시/한점 리더 및 #4068 기존43건 PASS/0FAIL, 테스트3.725초. 필터 제외1241건이다. 기대값을 바꾸거나 새 렌더 좌표 검사를 추가하지 않았다.
- Cargo jobs8/test threads8, 공유 target 순차 실행. [명령/결과](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/focused-result.json).
- 동일 글꼴713개·PDFium smooth_rgb_v1·96DPI Print·원본 세대 compat2022/2024·2px raw로 base와 후보를 각각 전쪽 재산출했다. 이미지 스택8/8쪽은 최저96.94292%, 규제분석17/17쪽은 최저99.26826%, 둘 다90% 미만0쪽이다. source/독립PDF/frozen producer hash는 [producer](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/producer.json)에 보존한다.
- 이미지 스택3쪽은 단계38의96.42147%에서98.12236%로 개선됐다. 직접 review/overlay에서 제목의 뒤 글자 밀림 해소를 확인했다. 그러나 base98.50524% 대비0.38288%p 낮고, 1/2쪽도 각각0.47930/0.31094%p 낮아 전체 수용은 보류다. 본문·괘선/래스터 잔여를 별도로 보정한다. 규제분석은 base 대비0.1%p 이상 하락0쪽이다.
- [3쪽 review](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/image-stack-003-review.png) / [overlay](../pr/assets/issue7445/completed-20261009/stage39-verified-dot-supplier-20261022/image-stack-003-overlay.png).

## 완료 범위와 잔여

이것은 확인된 glyph 공급자 보정과 Native 검증이다. 새 후보의 fresh WASM, 전체 회귀와 실제 소비 경로·PR 제출 게이트를 대체하지 않는다. 단계38의 실패147건 재검사 중56FAIL은 계속 보정 대상이며 이번43건 통과로 소거하지 않는다. 지침·원본·기준 PDF·글꼴 설치 파일은 바꾸지 않았고 외부 push/PR/댓글 게시도 하지 않았다.

원본 로그/PNG/TSV/source hash/producer는 ignored `output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v5`, `one-dot-native-v5`, `image-stack-page3-text-programs`에 보존한다. `.log`와 글꼴 바이너리는 커밋하지 않는다.
