# #8153: KoPub 결측 한점 리더의 실제 공급자 전진

## 원본과 독립 근거

두 원본 368쪽 접수인의 날짜 줄은 구역 12의 같은 셀 문단이다. 원문은
`(     ․     ․      ․      ：   )`이며 실제 한점 리더는 세 개다. 바깥 run의
중첩 표 텍스트를 중복 집계하지 않는다. 글자모양 53은 10pt, 장평98%, 기호 자간−6%,
가운데 정렬이며 공백 최소값은 0이다. 원본·기존 독립 한컴 2024 Print PDF를 유지한다.

실제 KoPub/KoPubWorld 바탕·돋움 세 굵기, 총12개 TTF의 cmap에는 U+2024가 없다.
이 프로그램의 SHA-256은 기존 `font_metric_cjk_punctuation.rs`에 이미 기록돼 있다.
독립 Print에서 이 점만 Haansoft Batang으로 공급하며 CIDFont `/W`는173/1000,
설치된 원본 HBATANG.TTF의 cmap/hmtx는178/1024em이다. 파일 SHA-256은
`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`다.
이는 반올림한 PDF 폭 표와 원본 프로그램의 차이이며 새 숫자를 화면에 맞추지 않는다.

기존 공통 장치 계산에 원본178/1024em을 넣으면 ppem250, 장평 적용42단위,
자간−3단위로 `(42−3)×4/75 = 2.08px`다. 독립 Print의 점 다음 공백 원점 차이는
2.07935px이며 같은 반각 공백은6.56px다. 현재 점은6.13333px로 측정돼 줄이12.16px
넓어지고 가운데 정렬의 시작도 달라진다. 공백 폭·정렬 원점·페이지별 상수를 고칠 문제가 아니다.

## 실제 경로와 공통 보정

같은 글자코드의 실제 paint는 이미 선언한 `KoPub바탕체 Light,한컴바탕`에서
HBATANG을 선택한다. 기존 고정 Native producer와 같은 PNG임을 확인한 공개 core 진단에서
세 점 모두 `font_metric_trusted=false`, `glyph_fallback_advance_explicit=false`,
`source_font_is_hft=false`다. KoPub은 별도 부분 폭 표를 사용하므로 표 전체를 신뢰하는
플래그를 바꾸어서는 안 된다.

`verified_named_program_has_glyph`가 호출하는 기존 프로그램 coverage 조회는
CJK 구두점 블록만 수용해 U+2024의 확인된 결측을 전달하지 못한다. 같은 원본 프로그램과
이름 해석을 재사용해 확인한 U+2024 결측만 공개한다. 메트릭 표 미수집을 결측으로
간주하거나 새 family matcher를 중복 구현하지 않는다.

측정에서는 이 독립적인 named-program 결측 근거와 HFT 보호 조건을 구별해 유지한다.
U+2024의 일반 미확인 슬롯을 위한 기존 gate는 보존하되, 원 프로그램의 cmap 결측이
확인된 이 경로는 같은 선언 paint 체인의 검증된 공급자 메트릭을 소비한다.
HBATANG178/1024em과 기존 장치 계산을 재사용하며 KoPub 전용 폭 표에 공급자 폭을
복사하지 않는다. 유효한 원 glyph hit, HFT 전각 점·원본 HFT 계약은 그대로 지킨다.

생산·소비 연결은 `원본 프로그램의 cmap 결측 → 선언 paint 공급 체인 → 기존 공급자 hmtx →
공통 장치 전진 → 줄 폭/가운데 정렬 → 실제 positions → Native/Canvas paint`다.
글꼴 프로그램·윤곽·SO 분석 자료는 output에만 보존하고 Git에 추가하지 않는다.

## 검증

두 원본의 실제 368쪽에서 세 점의 독립 전진 구간·같은 원문 셀 소유·내용 순서를 검사한다.
공백의 독립6.56px 전진과 KoPub 자체에 있는 U+2027의 원래900/1000em 계약도 보호한다.
고정 producer에서 실제 FAIL을 확보하고 새 구현에서 PASS를 확인한다. 측정용 합성 스타일의
대조와 실제 문서의 시각 검증을 구분한다.

기존 74건과 glyph/fallback 정상 대조, 35·64·83·368쪽 Native/fresh WASM 직접 판독,
두 원본 전쪽 TSV를 확인한다. 모든1,532조건의 raw100·미대응0과 최종 필수 테스트를 완료한
뒤 PR 준비까지만 하고 대기한다. upstream 동기화·원격 게시·PR 생성은 하지 않는다.
