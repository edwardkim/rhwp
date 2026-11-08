# PDF 위치 기반 단어 간격 대조군

직접 생성한 1쪽 PDF이며 사용자 문서·원 glyph outline·폰트 프로그램은 포함하지 않는다.
PDF SHA-256: `ec5cb353f18c1225f24dc73b7ecc6ea729bc1d17c445f24b810b56ffb90d82c5`.

이름이 없는 Type3 문자 프로그램이 외부 HCR Batang CID font로 글자를 그린다.
첫 줄은 두 Tj text object 사이에 0.25em의 물리 간격을 둔다. 원 문자에는 U+0020이 없다.
둘째 줄은 연속 문자, 셋째 줄은 음수 TJ kerning이다. 기존 표 lowerer의 단어 간격 복원이
positional text에도 적용되어 첫 줄의 간격만 보존하는지 확인한다. 모든 PDF의 의미상
단어 경계나 Type3 glyph 재현이 완성됐다는 검사는 아니다.

```bash
python3 pdf/issue7958/generate.py --font <로컬의-regular-HANBatang.ttf>
```

생성과 독립 Poppler 인쇄에는 로컬로 적법하게 설치한 HCR Batang을 공급한다.
검증에 쓴 프로그램 SHA-256은
`0dbeb7b129fcec63f8d6b7f1d3b5bb991c1d599e725286518ccc598de643f014`다.
PDF는 해당 font의 이름·advance·CID/GID 매핑만 참조하며 font bytes를 내장하지 않는다.

입력 PDF의 독립 Poppler 출력과 runtime `975655b49` Native Skia 및 새 Docker WASM
(`60a17a1b6a2a98208518481ab3c5ad2561c61633c18b83b00ffb851c91745a4c`)의 전쪽
96 DPI / 400×320 비교가 각각 100%이며 같은 배율로 직접 판독했다. 이 선행 검증 뒤
정식 검사를 추가했다. 로그·TSV·PNG는 ignored
`output/pr-review/issue7957/f7/final/positioned-*/`에 보존한다.
수정 전 보존 라이브러리에 같은 Rust 검사를 연결한 실행은 첫 줄의 간격 삭제로 FAIL이다.
정식 검사는 문자 소속·누락/중복·위아래 순서·세 줄의 상대 점유 폭과 두 저장 형식을 보호하며
px 좌표나 SVG hash를 고정하지 않는다.
