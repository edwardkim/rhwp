# HML format inventory

## 목적과 판정 기준

독립 `.hml`을 추정 XML이 아니라 재배포 가능한 실제 파일에 맞춰 구현하기 위한 inventory다.
파일의 출처와 라이선스가 확인되어도 생성 도구가 문서화되지 않았으면 그 사실을 별도로
표시한다. 시각 정답지는 한컴이 같은 파일에서 출력했다는 provenance가 확인되어야 한다.

## 채택한 corpus

### `samples/hml/aligns.hml`

- 원본: `ohah/hwpjs/crates/hwp-core/tests/fixtures/aligns.hml`
- 고정 commit: `e2beadb2cfbbae6c814c4db6644383f054903c3c`
- 원본 URL: <https://github.com/ohah/hwpjs/blob/e2beadb2cfbbae6c814c4db6644383f054903c3c/crates/hwp-core/tests/fixtures/aligns.hml>
- 재배포 근거: upstream MIT license, notice를 `samples/hml/ohah-hwpjs-LICENSE.txt`에 보존
- SHA-256: `c0f05b2ff380ce2f64fce41c8c318d1c94b6d5caf395ac6ff2a7681adc8a1708`
- 크기: 48,883 bytes
- XML validation: `xmllint --noout` 통과
- 생성 도구 provenance: HWPML 고유 구조, `SubVersion="10.0.0.0"`, 한글 날짜 metadata,
  같은 디렉터리의 HWP/HWPX 쌍은 한컴 export를 강하게 시사한다. 그러나 정확한 한컴 버전과
  OS를 명시한 upstream 문서는 찾지 못했으므로 검증 완료로 표기하지 않는다.
- 시각 정답지: `samples/hml/aligns-hancom-viewer-macos.pdf`. 원본 바이트를 macOS 26.5.1의
  Hancom Office HWP Viewer 12.31.7 (build 6383)에서 열어 생성한 16쪽 A4 PDF다. 개인정보성
  author/title metadata를 제거했으며 SHA-256은
  `0e8065a23626668b7dcd8acc5ec37a0d388f6137762b94abb01efb5e64e880fa`이다.

### `samples/hml/formatting_table.hml`

- 원본: `osik-kwon/osk_filter/test/sample/hml/hml.hml`
- 고정 commit: `8b483dc73edfe31b34c9c2324e1096be474fa341`
- 원본 URL: <https://github.com/osik-kwon/osk_filter/blob/8b483dc73edfe31b34c9c2324e1096be474fa341/test/sample/hml/hml.hml>
- 재배포 근거: upstream Unlicense, 원문을 `samples/hml/osk_filter-UNLICENSE.txt`에 보존
- SHA-256: `177b93de7c79462bef7850e15b018dbc8138fbe713b9c7e045b6125cc7527d35`
- 크기: 29,500 bytes
- XML validation: `xmllint --noout` 통과
- 생성 도구 provenance: HWPML 고유 serializer 형태, `SubVersion="10.0.0.0"`, 2019년 author/date
  metadata, HWP/HWPX/HML parser test corpus라는 맥락은 한컴 export를 강하게 시사한다. 정확한
  한컴 제품 버전과 OS를 적은 문서는 없어 검증 완료로 표기하지 않는다.
- 시각 정답지: `samples/hml/formatting-table-hancom-viewer-macos.pdf`. 같은 Viewer/OS에서
  원본 바이트를 열어 생성한 1쪽 A4 PDF이며 개인정보성 metadata를 제거했다. SHA-256은
  `118a488520a373b0eb05b80d77920f092e0c805f3c5b780f9cf8e3b358efb89c`이다.
- coverage: section 1, paragraph 4, table/row/cell 각 1, rectangle textbox 1.
- 미포함: picture, equation, `BINDATA`, 외부 resource.

## 실물 구조 inventory

| 영역 | 관찰한 HML 구조 | 수량/값 | 공통 IR 의미 |
| --- | --- | --- | --- |
| 문서 root | `HWPML` | `Version=2.91`, `SubVersion=10.0.0.0`, `Style=embed` | format/version metadata |
| namespace | 없음 | `xmlns` 없음 | legacy HML signature 입력 |
| head | `HEAD` | `SecCnt=1` | 문서 정보와 resource table |
| resource table | `MAPPINGTABLE` 아래 `FACENAMELIST`, `CHARSHAPELIST`, `PARASHAPELIST`, `STYLELIST` 등 | font, border/fill, 글자/문단 모양, style | `Document.doc_info` 계열 |
| body | `BODY/SECTION/P/TEXT/CHAR` | section 1, paragraph 33, text 33, char 33 | section, paragraph, run/text |
| section property | `SECTION/SECDEF`, `PAGEDEF`, `COLDEF` | 1 section | page/column 설정 |
| drawing | `RECTANGLE/SHAPEOBJECT/DRAWINGOBJECT/DRAWTEXT` | rectangle 16 | 도형, anchor/wrap, 글상자 내부 문단 |
| tail | `TAIL/SCRIPTCODE` | 1 | 실행 금지, 미지원 warning 대상 |
| table | `formatting_table.hml`의 `TABLE/ROW/CELL` | 각 1 | 1x1 표 mapping 근거 |
| embedded binary | 관찰하지 못함 | `BINDATA=0` | mapping 근거 없음 |
| image/equation | 관찰하지 못함 | 모두 0 | mapping 근거 없음 |

두 번째 fixture인 `formatting_table.hml`에서 `TABLE/ROW/CELL`, `SHAPEOBJECT`, `CELLMARGIN`,
셀 내부 `PARALIST/P/TEXT/CHAR` 계층을 확인했다. 표의 실제 수량은 1행 1열이며, 병합과 복수
행/열, 반복 머리행, page break 동작은 아직 실물로 확인하지 못했다.

DTD와 entity 선언은 없다. `TAIL`에는 JScript source가 있으므로 파서는 이를 데이터로만
취급하거나 경고 후 무시해야 하며 실행 경로를 만들면 안 된다.

## 탐지 계약에 미치는 영향

이 실물은 HWPX의 2011 namespace를 사용하지 않는다. 따라서 다음 조건을 모두 요구하는
signature는 이 파일을 놓친다.

1. `HWPML` root
2. 한컴 HWPML namespace URI

legacy HML 탐지는 BOM과 XML declaration을 제거한 제한된 prefix 안에서 root local-name과
실물에서 확인된 version/subversion/style 구조를 함께 검증해야 한다. namespace가 있는 다른
HML 계열은 별도 실물을 확보한 뒤 추가한다. 일반 XML/HTML 오탐 방지를 위해 root 이름만으로
전체 parse를 허용하지 말고, bounded XML reader가 root 속성과 `HEAD/BODY` 구조를 최종 검증해야 한다.

HWPX parser 역시 이 XML을 그대로 받을 수 없다. HWPX의 namespace 기반 소문자 XML과 달리
legacy HML은 대문자 element/attribute 계약을 사용한다. `src/parser/hml/` adapter가 이 차이를
명시적으로 변환하고, 의미가 일치한다고 실물로 확인된 HWPX helper만 재사용해야 한다.

## coverage와 구현 gate

| 요구 coverage | 상태 | 근거/다음 조치 |
| --- | --- | --- |
| UTF-8 BOM과 XML signature | 확보 | `aligns.hml` |
| 문단, 글자/문단 shape table | 부분 확보 | `aligns.hml` |
| 도형/글상자 및 anchor 위치 | 부분 확보 | rectangle 16개 |
| 표 | 부분 확보 | `formatting_table.hml`의 1x1 표. 병합/복수 행열과 oracle은 미확보 |
| 그림 및 base64 resource | 미확보 | 한컴 저장 HML과 oracle 필요 |
| 수식 | 미확보 | 한컴 저장 HML과 oracle 필요 |
| UTF-16 LE/BE | 미확보 | 실제 파일 필요. 합성은 decoder 보안 테스트에만 사용 |
| 외부 resource | 미확보 | 실제 href 표현 확인 필요 |
| 다른 HWPML version | 미확보 | 실물 추가 필요 |
| 페이지 수/배치 oracle | 2종 확보 | Hancom Office HWP Viewer PDF: `aligns` 16쪽, `formatting_table` 1쪽. 원본 한컴 편집기 출력과 image/equation fixture는 추가 필요 |

현재 corpus로는 signature, UTF-8 decoding, bounded XML reader, HEAD/BODY/TAIL 분해와 관찰된
문단/도형/1x1 표 mapping만 근거를 세울 수 있다. 그림, 수식, binary/external resource 및
복잡한 표 mapping을 추정 구현하거나 전체 HML 호환성을 선언하면 안 된다.

## 제외한 후보와 이유

| 후보 | 관찰 | 결정 |
| --- | --- | --- |
| `recrack/ruby-hwp`의 `empty.hml`, `table.hml`, `table5x7.hml` 및 PDF | HWPML 2.8/SubVersion 8.0.0.0 실물로 보이고 PDF 쌍이 있으나 저장소에 license field/LICENSE가 없고 PDF 생성 환경도 없음 | 권리자 허가 또는 라이선스 확인 전 복사 금지 |
| `disjukr/hwpkit` HML | 공개 corpus지만 upstream이 AGPL로 고지 | 프로젝트 호환성 검토 전 복사 금지 |
| `123jimin/node-hwp` HML | MIT이나 HWP에서 `toHML()`로 생성된 fixture | 한컴 저장 실물 요구를 충족하지 않아 oracle corpus에서 제외 |
| `msjang/md2hml/readme.hml` | Python HML generator의 출력이며 저장소 license 없음 | 권리와 Hancom provenance 모두 미충족 |
| `Jelly1500/Markdown2HWP_Program` templates | MIT이고 Hancom automation으로 HWP를 HML2X로 저장하는 helper가 있으나, project generator가 template XML을 직접 수정하며 각 현재 파일의 마지막 저장 주체를 증명하지 못함 | library-generated 가능성을 배제할 수 없어 ground truth에서 제외 |
| `osk_filter/hml_한글경로.hml` | 채택한 `hml.hml`과 동일 blob SHA `0c327bccabd8e46028d912dca9357b854a3750d1` | 독립 coverage가 없어 중복 제외 |
| `osk_filter/privacy.hml` | Unlicense repo 안에 있으나 주민등록번호 형태 test 문자열과 `(주)한글과컴퓨터` author metadata 포함 | 개인정보 패턴과 제3자 provenance 때문에 재배포 제외 |
| `zreview`, `anb`, `seung`의 picture/binary HML | image 또는 binary tag 후보이나 repository license 없음 | 권리 확인 전 복사 금지 |

## 남은 corpus 확보 기록 항목

새 실물마다 한컴 제품/버전, OS, HWPML version/subversion, encoding, embedded/external resource,
재배포 허가, 원문 SHA-256, 한컴 PDF/PNG SHA-256 및 페이지 수를 기록한다. 개인정보를 제거한
뒤 한컴에서 다시 저장하고 oracle을 다시 생성해 fixture와 정답지가 동일 문서인지 보증한다.
