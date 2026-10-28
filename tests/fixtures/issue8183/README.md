# #8183 원본 bitmap과 upstream JBIG2 자료

`scan-source-gray-200.png`는 기존 한컴 Print PDF의 첫 쪽을 Poppler로 래스터화한 독립
원본이다. `scan-source-mono-200.png`는 같은 자료에 180의 고정 threshold를 적용한 것이다.
JPEG 2000/JBIG2의 lossless decode가 전체 원본 행·픽셀을 보존하는지 검사한다. 렌더러의
페이지 좌표나 안티앨리어싱을 고정하는 golden이 아니다.

`source/*.jbig2`는 LaurenzV/hayro의 공식 테스트 자료다. 정확한 출처·Git blob과 원본 hash는
[PDF provenance](../../../pdf/issue8183/provenance.json)에 있다. 해당 hayro-jbig2의 MIT/Apache-2.0
고지와 [제삼자 라이선스](../../../THIRD_PARTY_LICENSES.md)를 따른다.

생성·전체 페이지 검증 조건은 [PDF 입력 안내](../../../pdf/issue8183/README.md)를 따른다.
