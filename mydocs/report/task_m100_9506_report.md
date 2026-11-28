# #9506 PDF 가져오기의 시작점만 있는 경로 처리

[이슈 #9506](https://github.com/edwardkim/rhwp/issues/9506) · [구현 계획](../plans/task_m100_9506_impl.md)

기준은 `upstream/devel` `274ae9d4c83154e06da049c90eed6904a8b2ec89`, 수정 소스는 `24fc681816ee345c23c71923263df7a3fbfafaa7`이다. 원본은 167쪽이며, 17쪽의 Form XObject 안에 유한한 시작점만 있는 열린 경로를 `S`로 끝내는 경우가 세 번 있었다. 이 경로에 선분이 없어 경로 빌더가 `invalid PDF path`를 반환했고, 전체 가져오기가 중단됐다.

[공통 경로 변환](../../src/parser/pdf/path_paint.rs)에서 유한한 `MoveTo`만 있는 경로가 IR 개체나 이미지가 되기 전에 반환한다. 선분을 포함하거나 닫힌 경로는 기존 처리를 따르며, 둥근 끝이 적용되는 길이 0의 획은 표시될 수 있다. [ISO 32000-1 §8.5.3.2](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf)

## 재현과 Native 검증

- 원본과 [2쪽 합성 PDF](../../pdf/issue9506/open-move-stroke.pdf)가 기준 CLI에서 `PDF_LOWERING_FAILED`로 중단됐다. 두 번째 쪽의 Form에 있는 열린 경로 두 개를 `n`으로 끝낸 [대조 PDF](../../pdf/issue9506/open-move-control.pdf)는 기준 CLI에서도 두 쪽이 모두 변환됐다. 두 합성 PDF는 MuPDF의 96 DPI 렌더에서 전쪽 RGB 픽셀이 같았다.
- 수정 CLI로 원본 전체를 직접 가져와 HWP·HWPX로 저장했다. 두 저장본을 다시 열어 각각 167쪽을 확인했다. HWPX 파일은 기존 우회 변환본과 바이트가 같았고, 17쪽의 Native PNG도 같은 로컬 글꼴 공급 조건에서 픽셀이 같았다.
- 합성 PDF의 수정 CLI 출력은 이전 CLI가 변환한 대조본과 전 2쪽의 RGB 픽셀이 같았다. 독립 PDF 출력과 96 DPI·2px raw 실루엣으로 대조한 두 쪽 모두 100.00%였다. 일반 선, 채움과 둥근 끝의 길이 0 획이 유지됐다.
- [집중 회귀](../../tests/cases/issue_7966_pdf_close_stroke.rs) 4건이 통과했다. 동일 테스트 바이너리의 PDF 관련 15건도 모두 통과했으며, 두 결과에 공통 검사 4건이 포함된다. `cargo fmt --all -- --check`와 테스트 모음 매니페스트 검사도 통과했다.

Native 빌드는 `release-test` 프로필과 `native-skia` 기능을 사용했다. 실행 파일의 SHA-256은 `e86af19bc9a8c89e73435f4c3d53f1ff3e73475d0c5e47d8fe0ba263618ca230`이다. 원본과 원본 파생본은 로컬에서만 검증했다.

## 브라우저 검증

- `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 RHWP_WASM_CDYLIB_ONLY=1 scripts/wasm-pack-locked.sh --target web --out-dir <output/pkg> --release`로 빌드했고 `wasm-opt`도 성공했다. 새 WASM의 SHA-256은 `5f144d99922b0493e1e4a5002b6578b9a2b3eeb3af0580454d02a11d3bcf3278`이다.
- Chrome 156.0.8078.47에서 합성 입력과 대조본의 전 2쪽은 HWP·HWPX 저장 전후의 SVG와 Canvas PNG가 각각 같았다. Canvas Print와 MuPDF의 원 PDF를 96 DPI·2px raw 실루엣으로 대조한 결과는 두 쪽 모두 100.00%였다.
- Studio와 공통인 `PdfImportSession` 및 PDF Worker에서 원본 167쪽을 직접 가져왔다. Worker가 저장한 HWPX를 브라우저에서 다시 열어 167쪽을 확인했다.
