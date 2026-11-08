# 미참조 post 이름 저장소의 FontFace 거부 보정

제품 코드 `0322cfe65b99c24685e5a30f67ae49c7e7f11272`의 WASM 전쪽 실행에서
`exam_science.hwp`와 `issue1994_behindtext_table_20200830.hwp`가 설치된 한겨레결체를
공급하다 중단됐다. Chrome OTS의 실제 오류는 `post: Bad string of length 46`이었다.
이 실패를 미달 페이지나 측정 완료로 합산하지 않았다.

[OpenType post 형식 2 명세](https://learn.microsoft.com/en-us/typography/opentype/spec/post#version-20)의
`glyphNameIndex`가 참조하는 모든 Pascal 문자열까지 원본 바이트를 보존하고,
어느 글리프도 참조하지 않는 뒤쪽 이름 저장소만 제거한다. 참조된 문자열이 잘렸거나
글리프 수가 일치하지 않으면 이름을 추측하지 않는다. 특정 글꼴명 분기는 추가하지 않았다.

[테이블 보존 근거](table-preservation.json): 원본 post116,472바이트 중 필요한58,825바이트를
그대로 보존했다. 윤곽·문자맵·폭·힌팅을 포함한 다른 테이블은 동일하며 SFNT checksum만 갱신했다.
원본 설치 파일은 수정하지 않고 파생 글꼴은 ignored output에만 생성한다.
[실제 FontFace 로드](browser-font-load.json)에서 원본은 거부되고 이 보정본은 적재됐다.
글꼴 binary는 Git에 포함하지 않는다.

공개 합성 fixture의 관계형 검사는 수정 전1 FAIL → 수정 후1 PASS다.
참조된 이름·정상 표·잘린 필수 문자열을 대조하는 검사를 더했고 관련 Python162건은 모두 통과했다.
이 검사는 실물 문서의 px 좌표나 일치율을 고정하지 않는다.

[두 문서 재실행](validation.json)은 각4쪽을 모두 완료했다. 과학 문서의 최저는99.22%,
뒤배경 표 문서는92.29%이며 90% 미만·누락은0쪽이다.
[과학 전쪽 TSV](incoming8754-science-wasm.tsv)와
[뒤배경 표 전쪽 TSV](incoming8754-clip5587-wasm.tsv)를 보존한다.

전체 공급 프로그램1,219개의 [영향 감사](catalog-impact.json)에서 미참조 저장소가 있는 프로그램은
5개였다. 기존 성공91조건의 실제 공급 메타데이터를 점검하고 영향받은 프로그램을 사용한
대조군 `nars-2021`은 전3쪽을 재출력했고 모두 원래 PNG와 byte hash가 같다.
[실제 공급 메타데이터270개·재출력 대조](whole91-reuse-proof.json)에 연결한다. 별도 보호 TSV의 WASM10쪽 하락 해결이나
PR 전체 준비 완료를 의미하지 않는다.

대표 쪽도 직접 판독했다. 뒤배경 표3쪽은98.67%, 과학1쪽은99.91%이며 본문·표·도형 소속을
확인했다. 뒤배경 표의 다른 쪽에서 남은 자형 차이를 해결했다고 확대하지 않는다.

- 뒤배경 표3쪽: [raw](incoming8754-clip5587-p003-raw.png) ·
  [PDF](incoming8754-clip5587-p003-pdf.png) · [review](incoming8754-clip5587-p003-review.png) ·
  [overlay](incoming8754-clip5587-p003-overlay.png).
- 과학1쪽: [raw](incoming8754-science-p001-raw.png) · [PDF](incoming8754-science-p001-pdf.png) ·
  [review](incoming8754-science-p001-review.png) · [overlay](incoming8754-science-p001-overlay.png).
