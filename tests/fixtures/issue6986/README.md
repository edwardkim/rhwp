# PAGE / TOTAL_PAGE 다중 쪽 검증 입력

PR #7053 및 #7048 누적 후보의 native/WASM SVG 검증에 사용한 실제 HWPX 두 파일이다.
임시 작업 경로의 파일과 byte 단위로 동일한 산출물을 커밋에 보존한다.

원본은 `samples/issue6986/cell-page-and-total-page-in-one-run.hwpx`다.
`Contents/section0.xml`의 첫 문단을 11개 복제하여 `id=101..111`, `pageBreak=1`을 지정하고,
복제 문단의 `secPr`를 제거했다. 역순 파일은 모든 `autoNum`의 PAGE/TOTAL_PAGE를 맞바꿨다.
원본의 나머지 ZIP entry 내용은 보존했다. XML 직렬화와 ZIP 압축 때문에 패키지 바이트는 원본과 다르다.

- `page-total-page-12-pages.hwpx`: 각 쪽에 현재 쪽 번호 / 전체 쪽 수(12).
- `total-page-page-12-pages.hwpx`: 각 쪽에 전체 쪽 수(12) / 현재 쪽 번호.
- SHA-256과 크기는 `MANIFEST.json`을 따른다.

합성 field 계약 검사다. 한컴 PDF를 생성하거나 이 변형의 한컴 시각 일치를 확인한 것으로 기록하지 않는다.
