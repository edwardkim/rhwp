# #8153: 목차 글머리표의 원 glyph와 전진폭 보존

## 원인과 독립 근거

두 편람 원본의 8쪽 목차는 `U+F031C`와 공백 뒤에 본문을 배치한다.
현재 의미 대체 문자 `■`의 폭과 윤곽을 사용해 기호가 커지고 뒤 본문이 밀린다.
고정 producer `0bdcdbf92d`의 미대응은 Native 724픽셀, fresh WASM 732픽셀이다.

독립 한컴 2024 Print의 기호는 PDF 내부 glyph 6으로, 실제 설치된
`HBATANG.TTF`의 `U+F031C`와 윤곽이 같고 양쪽 전진폭이 512/1024em이다.
본문 시작은 독립 PDF에서 131.279999pt, 글머리표 시작은 119.279999pt다.
원 글꼴의 cmap/hmtx와 원본 글자 속성을 소비하며 이 수치를 구현에 하드코딩하지 않는다.
분석 기록과 글꼴 SHA는 ignored `output/pr-review/issue8153-handbook-raw100-20261022/`
`remaining-raw100-diagnostic/p008-independent-glyph-proof.json`에 둔다.
글꼴 프로그램을 저장소에 복사하거나 배포하지 않는다.

## 기존 해결과 공통 소비 경로

#5599와 #5800의 의미 대체 및 원 기호 정규화는 유지한다.
`774d414cd9`와 `27f18fbac3`의 실제 공급 SFNT 기반 원 PUA 경로에 확인한 기호를 추가한다.
실제 원 glyph 공급이 없으면 기존 Unicode `■` 대체를 그대로 사용한다.
일반 Unicode `■`를 PUA로 재분류하지 않는다.

`원본 기호 → supported symbol → 실제 호스트 프로그램의 cmap/hmtx snapshot →
공통 줄 측정/구성 → render tree 표시 → Native/Canvas paint`를 공유한다.
이 경로에는 별도의 문서별 좌표 보정·backend별 폭 보정을 추가하지 않는다.

## 검증 순서

저장소가 직접 만든 합성 글꼴의 cmap만 메모리에서 바꾸어 원 기호 공급·전진폭·
해제 후 fallback·원본 IR 보존 계약의 수정 전 실패를 확인한다.
독립 hmtx 900/1000인 합성 입력으로 반각 하드코딩도 검출한다.
실물 8쪽과 기존 PUA 기호의 정상 대조군을 먼저 재출력하고 Print와 직접 대조한다.
이후 Native/fresh WASM 전쪽을 확인한다. 전체 100% 달성으로 보고하는 조건은
기존과 동일하게 1,532개 결과 모두 raw100·미대응 0픽셀이다.

## 추가로 확인한 같은 경로의 기호

43쪽의 `U+F03A8`·`U+F03A7`과 138~140쪽의 `U+F0848`도 독립 Print 내장
glyph 13개와 원 프로그램의 윤곽이 같다. 앞 두 기호는 1em, 뒤 기호는 0.5em이다.
의미 대체 문자의 윤곽·전진으로 원본을 대신하지 않고 같은 실제 공급 경로에 포함한다.
검증 전 현재 Native 미대응은 해당 쪽에서 각각 119·11·62·11픽셀이다.
`remaining-raw100-diagnostic/additional-native-pua-independent-proof.json`에
실제 PDF glyph ID·원점·원 문자 대응을 보존한다.

목차 기호만 보정한 `29b96a90f4`의 Native 22개 조건을 실행했다.
두 형식 모두 8쪽 724→1픽셀·99.99964%이며 나머지 20개 조건은 유지됐다.
이 결과는 WASM 검증 또는 100% 완료가 아니다. 같은 원점의 Print/수정 전후 crop을
직접 확인했으며 남은 1픽셀도 미해결로 유지한다.
