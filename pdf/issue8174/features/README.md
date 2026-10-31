# #8174 기본 PDF 기능의 독립 검증 입력

24개 입력은 각각 1쪽이다. ICC RGB·Gray·CMYK의 raw/JPEG/Indexed/path·간접 Filter, JPX의
Indexed·CMYK와 straight/premultiplied alpha의 raw J2K·boxed JP2, xref 누락·가짜 객체
header·incremental update를 다룬다. 입력·기준 hash와 ICC profile의 출처는
[provenance.json](provenance.json)에 보존한다. 기존 #8183의 정상 17개도 함께 검증했다.

## 생성과 독립 기준

```bash
venv/bin/python tools/fixture_generators/pdf_features_issue8174.py
```

PDF 객체는 직접 작성하고 Pillow·OpenJPEG encoder로 원본 샘플을 압축했다.
ICC profile은 기존 vendored ProPhoto/CGATS/sGrey 자료이며 새 라이선스를 추정하지 않는다.
RGB/Indexed의 `reference/icc-lcms-expected.png`는 Pillow/LittleCMS의 relative-colorimetric
변환으로 만들었다. 제품 qcms의 출력·RHWP가 생성한 PDF·PNG는 기준 생성에 쓰지 않았다.

ICC 기준은 원본 PDF만 Ghostscript pdfwrite에서 PDF 1.4·Flate로 변환한 `*-gs.pdf`다.
CMYK JPEG는 encoder의 역상 샘플과 PDF 기본 Decode를 그대로 유지한다. 이 입력이 어둡게
보이는 것은 독립 Ghostscript도 재현하며, 제품에서 임의로 반전하지 않는다.

JPX의 일부 2/5성분 raw stream과 Indexed+alpha는 독립 decoder 사이에도 차이가 있다.
Ghostscript의 원 JPX 오류·palette 차이를 RHWP와의 일치 증거로 쓰지 않았다. 해당 기준은
**독립 encoder에 넣은 샘플**의 원 인덱스로 palette를 lookup한 뒤, `/SMaskInData=2`의
base 색 성분을 alpha로 복원한다. ICC 변환보다 먼저 복원하며 외부 SMask를 둔
`*-source.pdf`를 독립 입력으로 쓴다. 이를 Ghostscript에서 정규화한
`*-independent.pdf`를 같은 PDFium으로 비교한다. raw/boxed는 같은 샘플 계약을 공유한다.
근거는 [PDF 32000-1 §7.4.9·Table 89·11.6.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf)다.
이 네 합성 계약이 모든 JP2 cdef·subsampling 지원을 의미하지 않는다.

복구 기준은 xref를 없애기 전의 같은 객체·내용과, 최신 incremental 객체를 정상 xref로
작성한 PDF다. 가짜 header는 주석·stream·문자열의 유효한 payload이며 실제 객체가 아니다.
trailer가 없는 입력에서 참조되지 않은 Info를 되찾았다고 주장하지 않는다. 남아 있는 trailer의
Info 보존, 모호한 stream 길이와 잘린 trailer의 거부는 별도 자원 계약으로 검사한다.

## 회귀 추가의 선행 증거

제품 source `5a77054519216f9f2dae6a779e7e334f3894466d`의 Native Skia와 최적화한
fresh WASM Canvas에서 **각 41조건·41쪽, raw 최저 100%**, 쪽수 누락 0을 확인했다.
새 24건과 기존 17건을 합친 값이다. 대표 ICC·JPX alpha·복구·200 DPI scan의
compare·overlay·review를 직접 확대 판독했고 색·연속 alpha·배치가 보존됐다.
미세 경계의 래스터 AA 차이는 실루엣과 구분한다. 전쪽 TSV 원문·대표 PNG·producer hash는
[영구 증적](../../../mydocs/pr/assets/issue8174/v2-s02/manifest.json), 명령과 소비 경계는
[S02 기록](../../../mydocs/working/task_m100_8174_v2_stage2.md)에 연결한다.

`tests/cases/issue_8174_pdf_default_features.rs`는 source 샘플·ICC 독립 픽셀·alpha·그림/쪽
소유와 HWP/HWPX 저장 보존, 실제 예산 실패·후속 정상 요청·cache 수명 관계를 검사한다.
렌더 해시·전체 페이지의 절대 좌표를 새 기대값으로 고정하지 않는다. 합성 PDF 계약이며
HWP/HWPX 원본에 대한 한컴 Print 일치나 F05/F06의 브라우저 제품 검증으로 확대하지 않는다.

## Indexed preblend 기준 해석의 보정

초기 `98e8431695`의 구현·기준은 인덱스를 alpha로 나누어 같은 잘못된 색을 공유했다.
§11.6.5.3의 명시적 palette 규칙을 대조해 공통 expander와 독립 기준을 바로잡았다.
**기존 원본 20개는 바이트 그대로 보존**했다. 기존 `jpx-indexed-premult`의 첫 색은
인덱스 64의 green을 복원·범위 제한한 값이며 red가 아니다. 과거 기준은 해당 Git 이력과
ignored `reference-before-palette/`에 보존한다. 정상 preblended RGB palette와 ICC palette의
raw/boxed 네 대조군을 추가했다. 새 전체 gate·확대 판독 뒤 엄격한 색 assertion을 교정했고,
테스트를 통과시키려고 허용치를 완화하거나 원본을 다른 입력으로 바꾸지 않았다.

이 확장은 모든 JPX 조합의 지원이 아니다. 이미 Lab→RGB로 변환해 원 Lab 성분 범위가 없는
palette의 preblend는 명시적으로 거부한다. straight alpha의 이전 출력 보존과 구분하며,
현재 정상 Gray/RGB/CMYK/ICC 확대를 해당 Lab 조합의 완료로 표시하지 않는다.
