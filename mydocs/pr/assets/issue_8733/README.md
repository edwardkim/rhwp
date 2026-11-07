# #8733 단색 배경 저장 검증 입력

이 디렉터리는 합성 문서, Studio 대화상자, 한컴 Print 관찰 증적이다. 신고자의 원 문서를 재배포하지 않는다. Print PDF는 [정식 PDF 경로](hancom-print.md)에 보존한다.

- 최초 기준 devel: `b8dd67c5001dfb0410e9e192528676396cdd6a4e`
- 검증한 소스: `e551fcc76e0794790e25a565f2509a4891eac6cb`
- 브라우저: Chrome 155, Rust 1.93.1 / wasm-pack 0.15.0의 optimized release WASM(`wasm-opt -O`). 실제 응답 WASM 해시를 산출물과 대조했다.
- 전체 결과와 파일별 SHA-256: [validation-summary.json](validation-summary.json)

## 입력과 재현

1. `cell-new-solid.*`, `table-new-solid.*`: 새 셀/표 단색 배경의 무늬 없음은 `-1`이다. HWP와 HWPX 저장 후 다시 열어 같은 값과 색상을 확인했다.
2. `imported-raw-zero.hwp`: 기존 가로 무늬 `0`을 합성한 대조군이다. 대화상자를 열어 테두리만 바꾸면 원본 무늬와 색상이 유지된다.
3. `cell-cleared-zero.hwp`, `table-cleared-zero.hwp`: 위 대조군에서 무늬 없음을 명시적으로 선택한 결과이며 `-1`로 저장된다. 실제 undo/redo도 확인했다.
4. `*-dialog.png`는 Studio 대화상자의 미리보기다. `hancom-*.png`는 독립 Print PDF에서 추출한 관찰 자료이며, Native/Canvas 정량 비교와 구분한다.

## 범위와 제한

수정 대상은 편집 속성의 기본값·부분 변경 보존과 직렬화 계약이다. compositor, 크기 계산, 페이지 배치, paint backend는 변경하지 않았다. HWP 원시 BorderFill 바이트와 HWP/HWPX roundtrip을 별도 Rust 회귀로 확인했다.

후속 사용자 승인으로 로컬 한컴 2020(11.0.0.9166)을 실행했고, 요청한 7개 입력의 채움·무늬 동작을 실제 화면과 Print PDF에서 확인했다. 새 단색 HWP/HWPX와 명시적 무늬 제거는 무늬가 없고, 기존 무늬 0은 가로 무늬를 유지한다. [원본 대응·해시](hancom-print-manifest.json)와 [Print 관찰 범위·제한](hancom-print.md)을 확인한다. 최초 표 HWP PDF는 수동 출력 중 덮어쓰기되었으나 후속 HWP 재인쇄를 고유 파일로 보존해 현재 7개 입력 모두 PDF가 있다. 출력자가 이후 7개 모두의 가로·세로 100%, 원래 용지, 전체 페이지, 한 쪽씩 설정을 확인했다. 이는 출력자 확인이며 UI 직접 관찰과 구분한다. 보안 우회나 원격 업로드는 하지 않았다.

양수 무늬 번호의 전면 매핑 변경과 이미 잘못 저장된 문서의 자동 복구는 포함하지 않는다. 너비·줄 간격 문제의 기존 #7608 수정은 유지하고 #7606 회귀 3건을 재검증했다.

## 페이지 시각 비교

[WASM 판독 집계](wasm/inspection.json), [Native 판독 집계](native/inspection.json)와 각 하위 폴더의 문서별 원본 TSV, compare·overlay·review PNG를 함께 확인한다. 양쪽 빌드·측정·리뷰 실행은 모두 exit 0이다.

- Native-skia optimized release 및 fresh optimized WASM, print profile, compat 2022, 96 DPI, 반경 2px, 색상 경계 보정 전 raw silhouette를 사용했다. 입력 확대·축소나 마스크를 추가하지 않았다.
- Native와 WASM 각각 7개 입력의 1쪽은 모두 raw silhouette 100.00%이다. 새 단색 HWP/HWPX 4개 및 명시적 무늬 제거 2개는 직접 판독에서 채움·배치가 일치하고 경계 래스터 차이는 남는다.
- **imported-raw-zero.hwp는 전체 페이지 시각 일치가 아니다.** 한컴에는 내부 가로 무늬 2줄이 있지만 Native와 WASM에서는 단색이다. [WASM 직접 비교](wasm/imported-raw-zero-hwp-review.png)와 [Native 직접 비교](native/imported-raw-zero-hwp-review.png)를 보존한다. 저장값과 한컴 인쇄의 기존 무늬 보존은 통과하지만 페이지 렌더러의 무늬 0 해석은 별도 잔여 문제다.
- 실루엣은 채워진 면적 내부의 무늬 차이를 놓칠 수 있어 100% 점수만으로 통과를 선언하지 않았다. 96 DPI 셀 내부에서 한컴 y138·146의 검은 픽셀은 각 554개이고 WASM은 0개다. 이 관찰은 진단 전용이며 공식 gate를 변경하지 않았다.
- 기존 Xpdf pdftotext 4.00은 Poppler의 `-bbox-layout`을 지원하지 않아 marker 추출이 생략됐다. 문항·본문 글자가 없는 단일 셀 합성 문서이며, PNG/PDFium 측정과 직접 판독은 수행됐다.

Native 명령은 "cargo build --locked --release --features native-skia --bin rhwp"이며 Rust 1.93.1, target/pr-review, 작업수 4를 사용했다. 실행 파일·WASM 해시, 고정 소스 SHA, 측정 집계는 [validation-summary.json](validation-summary.json)에 기록했다. TSV는 원본 바이트를 보존하며 개행 자동 변환을 비활성화했다.

정확한 측정 조건과 공개 입력 재현 명령은 [visual-commands.md](visual-commands.md)에 있다.


## 최신 devel과 무회귀 비교

최신 고정 devel e4ffabea38062e716f6c333cc950fa31b42b4dd7과 후보를 같은 7개 입력/PDF·Print profile·Windows 글꼴·96 DPI·compat 2022로 비교했다. Native/fresh WASM 전체 14개 page/backend 쌍의 raw는 모두 100.00%, Decimal 최대 하락은 0.00%p다. 실제 렌더 PNG 14개가 바이트·RGB 픽셀 모두 같고 대표 compare/overlay/review 18개도 같다.

[최신 비교 JSON·원본 TSV·이미지 목록](latest-base/comparison.json) · [빌드와 재현 명령](latest-base/REPRODUCE.md). 최초 b8dd67c5의 [역사적 비교](base/comparison.json)도 별도로 보존한다. Native profile과 WASM codegen partition의 기록 차이는 숨기지 않으며 실제 producer 해시와 출력 동일성을 확인했다. 기존 raw0 해치 누락은 최신 기준에도 존재하고 저장·편집 수정의 수용 범위와 구분한다.
