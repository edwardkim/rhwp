# #8413 PDF clip·mask·rotation의 저장 결과

[공개 합성 PDF](../../../pdf/issue8413/README.md)를 가져와 저장·재열기한 13쪽 HWP/HWPX다.
실제로 시각 검증한 바이트이며 [입력 manifest](../../../samples/issue8413/manifest.json)에 SHA-256을 기록했다.
이 자료에는 non-painting Type3의 검색 문자열과 투명 이미지처럼 의도적으로 보이지 않는 내용이
있다. 정상 업무 문서의 숨김 내용 오탐 코퍼스인 `samples/`와 구분한 계약 fixture다.
한컴 원문 또는 평범한 보안 clean 문서라고 주장하지 않는다.

Native/fresh WASM 전쪽 TSV와 직접 판독은 [검증 자료](../../../mydocs/pr/assets/issue8413/curve-mask-validation-20261101/README.md),
동일 PDF에서 glyph의 비표시 증명·검색 문자열·저장 보존을 검사하는 자동 회귀는
[정식 계약 검사](../../cases/issue_8413_pdf_curve_mask_rotation.rs)에 있다.
