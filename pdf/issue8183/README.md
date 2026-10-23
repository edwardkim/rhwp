# #8183 PDF 이미지 코덱 검증 입력

[3단계 기록](../../mydocs/working/task_m100_8183_stage3.md)과
[선택 빌드 안내](../../mydocs/manual/pdf_image_codecs.md)를 참고한다.
출처·파일 SHA-256과 생성 조건은 [provenance.json](provenance.json)에 있다.

17개 정상 PDF는 모두 1쪽이다. 정상 그림, 공유 JBIG2 사전, JPX tile·palette·PDF Indexed,
embedded alpha·premultiplied alpha와 별도 Mask/SMask를 다룬다. `corrupt-*.pdf` 두 개는
손상 거부 계약이다. `work-limit-jpx.pdf`는 정상 200dpi JPX 그림을 같은 쪽에 세 번 그리는
유효한 입력이다. 고정 요청 작업 예산의 누적 초과와 실패 후 복구를 검사하며 시각 golden이 아니다.

## 생성과 독립 기준

```bash
venv/bin/python tools/fixture_generators/pdf_codecs_issue8183.py
```

Pillow의 libopenjp2·libtiff encoder로 압축하고 PDF object/xref를 직접 작성한다.
스캔 입력은 기존 `pdf/hwp_table_test-2022.pdf`의 첫 쪽을 Poppler에서 래스터화한 자료다.
한글 문자·표를 이미지로 보존하며 OCR 결과로 사용하지 않는다. 200dpi 입력은 1,655×2,339픽셀이다.
JBIG2의 Group4 TIFF는 **단일 strip**으로 만든다. 각 strip의 독립 참조 줄·EOFB 때문에 첫 strip만
옮기면 본문이 빠진다. MuPDF가 추출한 전체 원본 해상도 bitmap과 압축 전 monochrome bitmap을
바이트 단위로 대조했다. 최초 잘못 생성한 자료와 관측은 ignored 작업 증적에 보존했다.

공유 사전 두 입력은 LaurenzV/hayro의 공식 시험 stream에서 standalone header·EndOfPage·EOF를
제외하고 page/global segment를 나눈 것이다. 원본은 `tests/fixtures/issue8183/source/`에 있고
정확한 Git blob·원본 hash를 provenance에 기록했다. 독립 decoder가 서로 불일치한 Huffman
실험 입력은 정상 입력·회귀 대상으로 채택하지 않았다.

독립 기준은 원본 PDF다. 다만 PDFium 153에서 JPX alpha·preblend 처리와 PDF Indexed의 JP2
palette 우선순위가 다른 세 입력은 `reference/*-gs.pdf`를 사용한다. 이 파일은 **원본 PDF만**
Ghostscript 10.02.1 pdfwrite에서 PDF 1.4의 Flate·외부 SMask로 변환한 결과다. rhwp의 출력을
기준 생성에 쓰지 않는다. 사양의 sample 합성, 원본 채널 값과 Ghostscript 출력이 일치하는지
확인하고 같은 PDFium·DPI·2px 실루엣 알고리즘으로 측정했다.

원본 JPX에 대한 최초 실패 TSV와 검정 Indexed 비교도 보존했다. 실루엣 100%만으로 색상을
판정하지 않으며 blue/cyan lookup, alpha 64/128, premultiplied 복원을 직접 판독한다.
[PDF 32000-1 §7.4.9·Table 89·11.6.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf),
[PDFium의 SMaskInData=2 검증 기록](https://pdfium.googlesource.com/pdfium.git/+/b18d018b2f9aa2913cc435b5ccdd59d0a5856ad8)을
함께 대조했다. source bitmap의 픽셀 계약과 backend별 페이지 raster의 안티앨리어싱 차이를 구분한다.

## 회귀 편입 근거

제품 source `cfc5be8fe`의 Native Skia/fresh WASM Canvas 전체 17쪽 TSV는 양쪽 최저 100%다.
전체 입력과 쪽수의 대응을 확인한 뒤 대표 9종의 compare·overlay·review를 직접 판독했다.
검사에서는 source sample·원본 bitmap, 그림 리소스·쪽 소유·저장 후 보존 관계를 확인한다.
렌더 페이지의 SVG hash나 조판 절대 좌표를 고정하지 않는다. 재생성하면 동일 절차를 다시 수행한다.
