# HML 샘플

이 디렉터리는 독립 HML(HWPML) 입력을 분석하고 회귀 테스트하기 위한 샘플을 보관한다.
샘플을 추가할 때는 원본 출처, 고정된 commit, 재배포 근거, 생성 도구, 인코딩,
HWPML 버전, 리소스 형태, 보조 대조 자료의 출처를 함께 기록해야 한다.

## 현재 샘플

| 파일 | 출처 | 재배포 | 실물 출처 판정 | 보조 대조 자료 |
| --- | --- | --- | --- | --- |
| `aligns.hml` | [`ohah/hwpjs`의 `aligns.hml`](https://github.com/ohah/hwpjs/blob/e2beadb2cfbbae6c814c4db6644383f054903c3c/crates/hwp-core/tests/fixtures/aligns.hml) | MIT, [`ohah-hwpjs-LICENSE.txt`](./ohah-hwpjs-LICENSE.txt) | 한컴 HML export로 강하게 추정되지만 생성 프로그램 버전과 OS는 upstream에 문서화되어 있지 않음 | `aligns-hancom-viewer-macos.pdf` |
| `formatting_table.hml` | [`osik-kwon/osk_filter`의 `hml.hml`](https://github.com/osik-kwon/osk_filter/blob/8b483dc73edfe31b34c9c2324e1096be474fa341/test/sample/hml/hml.hml) | Unlicense, [`osk_filter-UNLICENSE.txt`](./osk_filter-UNLICENSE.txt) | 한컴 HML export로 강하게 추정되지만 생성 프로그램 버전과 OS는 upstream에 문서화되어 있지 않음 | `formatting-table-hancom-viewer-macos.pdf` |
| [`exambank_math_equations_min.hml`](./exambank_math_equations_min.hml) | 기존 프로젝트 수식 최소 fixture, commit `1051e108564171a0e8ea7e557f0fac07266d4c31` | 프로젝트의 기존 배포 근거 | 합성 HML 원본과 사용자가 한컴에서 저장·Print한 대조 자료를 구분 | [2024 Print PDF](./exambank_math_equations_min-2024.pdf), [한컴 HWP 저장본](./exambank_math_equations_min.hwp), [한컴 HWPX 저장본](./exambank_math_equations_min.hwpx) |

`aligns.hml`은 upstream commit
`e2beadb2cfbbae6c814c4db6644383f054903c3c`의 바이트를 변경하지 않고 복사했다.

- SHA-256: `c0f05b2ff380ce2f64fce41c8c318d1c94b6d5caf395ac6ff2a7681adc8a1708`
- 크기: 48,883 bytes
- XML: UTF-8 BOM, XML 1.0, CRLF
- root: `HWPML`
- root namespace: 없음
- HWPML `Version`: `2.91`
- `SubVersion`: `10.0.0.0`
- `Style`: `embed`
- 구조: `HEAD`, `BODY`, `TAIL`, section 1개
- 내용: 문단 33개, 사각형 글상자 16개, 정렬과 위치 속성
- 없음: 표, 그림, 수식, `BINDATA`, 외부 리소스
- 보안 관련 관찰: DTD/entity 선언은 없지만 `TAIL/SCRIPTCODE`가 있으므로 import 시 script를 실행하면 안 됨

같은 upstream 디렉터리에 `aligns.hwp`, `aligns.hwpx`, HTML/CSS 출력물이 있다. 이 저장소에서는
원본 HML 바이트를 macOS 26.5.1의 Hancom Office HWP Viewer 12.31.7 (build 6383)로 직접 열어
`aligns-hancom-viewer-macos.pdf`를 생성했다. PDF는 16쪽 A4이며 개인정보성 author/title
metadata를 제거한 뒤 Creator/Producer만 보존했다. SHA-256은
`0e8065a23626668b7dcd8acc5ec37a0d388f6137762b94abb01efb5e64e880fa`이다.

`formatting_table.hml`은 upstream commit
`8b483dc73edfe31b34c9c2324e1096be474fa341`의 `test/sample/hml/hml.hml`을 파일명만
바꾸고 바이트는 변경하지 않았다.

- SHA-256: `177b93de7c79462bef7850e15b018dbc8138fbe713b9c7e045b6125cc7527d35`
- 크기: 29,500 bytes
- XML: UTF-8 BOM, XML 1.0, 줄바꿈 없음
- root: namespace 없는 `HWPML`, `Version=2.91`, `SubVersion=10.0.0.0`, `Style=embed`
- 구조: `HEAD`, `BODY`, `TAIL`, section 1개
- 내용: 문단 4개, 표/행/셀 각 1개, 사각형 글상자 1개
- 없음: 그림, 수식, `BINDATA`, 외부 리소스
- 보안 관련 관찰: DTD/entity 선언은 없지만 `TAIL/SCRIPTCODE` 1개가 있으므로 실행 금지

upstream은 HWP/HWPX/HML parser의 같은 test corpus에 이 파일을 보관하고 파일 metadata에는
2019년 한글 환경에서 생성된 형태가 남아 있다. 이 저장소에서는 원본 HML 바이트를 macOS
26.5.1의 Hancom Office HWP Viewer 12.31.7 (build 6383)로 직접 열어
`formatting-table-hancom-viewer-macos.pdf`를 생성했다. PDF는 1쪽 A4이며 개인정보성
author/title metadata를 제거했다. SHA-256은
`118a488520a373b0eb05b80d77920f092e0c805f3c5b780f9cf8e3b358efb89c`이다.

## 아직 확보하지 못한 필수 corpus

- 생성한 한컴 버전과 OS가 확인된 최소 텍스트 HML
- 글자/문단 서식과 표를 포함한 더 복잡한 HML 및 원본 한컴 편집기 PDF/PNG
- 내장 그림과 수식을 포함한 HML 및 한컴 PDF/PNG
- 다른 HWPML 버전과 UTF-16 LE/BE 실물
- 외부 리소스를 참조하는 실물

현재 HML 원본 3개만으로 전체 HML 지원이나 한컴 호환성을 선언하면 안 된다. 합성 XML
자체를 한컴 저장 실물로 간주하지 않으며, 수식 최소 입력의 한컴 저장본과 Print PDF처럼
독립 대조 자료가 있는 경우에도 그 출처와 검증 범위를 구분한다.

## 검토했지만 포함하지 않은 후보

- `recrack/ruby-hwp`: HML과 PDF 쌍이 있지만 저장소에 라이선스가 없어 재배포 권한을
  확인할 수 없다. PDF 생성 환경도 문서화되어 있지 않다.
- `disjukr/hwpkit`: AGPL corpus이므로 이 저장소와의 라이선스 호환성 검토 전에는 복사하지 않는다.
- `123jimin/node-hwp`: HWP에서 코드로 생성한 HML fixture라서 한컴 저장 실물 조건을 충족하지 않는다.
- `msjang/md2hml`: 저장소에 라이선스가 없고 `readme.hml`은 Python generator 출력물이다.
- `Jelly1500/Markdown2HWP_Program`: MIT이며 한컴 automation helper를 포함하지만 현재 HML
  template은 generator가 직접 읽고 수정하는 입력이다. 각 파일이 한컴에서 마지막으로 저장된
  바이트라는 provenance가 없어 한컴 ground truth로 채택하지 않는다.
- `osik-kwon/osk_filter`의 `hml_한글경로.hml`: 채택 파일과 blob SHA가 같아 독립 fixture가 아니다.
- `osik-kwon/osk_filter`의 `privacy.hml`: 주민등록번호 형태의 문자열과 제3자 author metadata를
  포함하여 corpus로 재배포하지 않는다.
- `tranquanghuy-rightsvn/zreview`, `jaehyeonjjang/anb`, `heonseung4-del/seung`: 그림 또는 binary
  후보 HML이 있지만 저장소 라이선스가 없어 복사하지 않는다.

## 수식 최소 입력과 한컴 저장·Print 대조 자료

`exambank_math_equations_min.hml`은 이 저장소의
`tests/fixtures/hml/exambank_math_equations_min.hml`을 바이트 변경 없이 복사한 공식 샘플이다.
기존 fixture의 도입 commit은 `1051e108564171a0e8ea7e557f0fac07266d4c31`이며,
프로젝트의 기존 배포 근거를 따른다. 사용자 제공 자료는 2026-10-08
`win10-ted`의 `C:\temp`에서 복사했다. 원격 HML·PDF와 기존 로컬 fixture의 SHA-256이 같다.

| 파일 | 생성·출처 | bytes | SHA-256 |
| --- | --- | ---: | --- |
| `exambank_math_equations_min.hml` | 기존 최소 fixture, UTF-8, HWPML 2.91 / SubVersion 9.0.0.0 / embed | 4088 | `b51be49cde780d39b92f42cfd1cbd58474900c46c12c4df971f79bd511c7045a` |
| `exambank_math_equations_min-2024.pdf` | 사용자가 해당 원본을 정상 수동 Print한 기준, Creator Hwp 2024 13.0.0.3901, 1쪽 | 28829 | `d5a494de0eb1eb7b14216fc7fb2197007ba7a30c61bf57f49d9b1bc101c13548` |
| `exambank_math_equations_min.hwp` | 사용자 제공 한컴 무편집 저장본. 저장 제품 metadata: 2020 / 11.0.0.9136 | 13312 | `701fedf957227d2432f859a5e6c99c170208a21ed8137c4e4e7e5704eeffe87c` |
| `exambank_math_equations_min.hwpx` | 사용자 제공 한컴 무편집 저장본. version.xml: 11.0.0.9136 / Windows 10 | 12102 | `39716b303c76ad81566217366b2aff44d614e148b3e7915f7b268b3a9cbac3d5` |

문단 19개, 수식 4개를 포함하며 외부 리소스가 없다. 원본 HML은 RATIO·RELSIZE·
PARAMARGIN·수식 SHAPEOBJECT를 생략한다. 한컴 저장본은 장평·상대 크기 0,
줄간격 0%, non-TAC 수식의 7200×7200HU 프레임을 기록한다. 원본 기본값의 실제
해석과 RHWP의 차이를 확인하기 위한 자료이며, 원본을 수정하거나 정상 글꼴 크기로
재생성한 문서가 아니다. 작은 본문도 수동 Print 기준의 실제 출력 상태로 보존한다.

HWP/HWPX 저장 제품은 제공 파일명이나 요청 제품으로 추정하지 않고 실제 metadata를
기록했다. 이 두 저장본의 정식 Visual Sweep에는 각 저장본을 engine 2020으로 다시
Print한 다음 PDF를 사용한다. 두 출력 모두 Hancom 11.0.0.9136,
`pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`을 확인했다.

| 저장본 | MCP Print PDF | SHA-256 |
| --- | --- | --- |
| HWP | [2020 Print PDF](../../pdf/issue7642-equation-print/hml-saved-hwp-exambank_math_equations_min-2020.pdf) | `45dff67c088f95e95cfed72f7a1f84e75132c77d891f8eabac65618325b94f55` |
| HWPX | [2020 Print PDF](../../pdf/issue7642-equation-print/hml-saved-hwpx-exambank_math_equations_min-2020.pdf) | `e76083fe1f2c708fb361c0fb7f5e13d9d7090b7bf24e3664f388339f09f460c3` |

원본 HML의 2024 PDF와 저장본의 2020 Print는 각각 구분하며,
현재 차이가 해결되었다거나 전체 HML 호환성을 입증했다는 뜻으로 사용하지 않는다.
