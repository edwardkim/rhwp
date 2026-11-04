---
kind: investigation
status: historical
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-10-31
---

# Issue #8403 — PDF 패턴 이미지의 검정 채움 원인

조사일: 2026-10-31. 이슈: [#8403](https://github.com/edwardkim/rhwp/issues/8403).
기준 소스: `upstream/devel` / `3c01421e89a562de4ba8b021a4cdf943af60b58d`.
후속 범위: [수행 계획](../../../plans/task_m100_8403.md).
이 기록은 원인 조사 결과이며 제품 수정이나 최종 시각 검증 완료 보고가 아니다.

## 입력과 재현

- 원본: `/mnt/e/tmp/black-image-01.pdf`, 1,098,214 bytes, PDF 1.7, 8쪽, 960 × 540 pt.
- SHA-256: `83fc3840db07d726257520469eca50bb249644701328783df7f8fbc0cef0d801`.
- 암호화 없음. Creator/Producer는 `smallpdf.com`이다.
- 사용자는 1·2·3쪽의 검정 이미지를 보고했다. 파일의 1부터 시작하는 쪽 번호로 확인하면
  1쪽 로고와 3쪽 그래프에서 재현되며 2쪽에는 이미지·패턴이 없는 텍스트만 있다.
- Poppler로 원본 1~3쪽을 출력해 정상 그림을 확인했다. 실제 Studio의 PDF 선택 대화창을
  확인한 뒤 Worker로 열면 성공 응답을 반환하지만 그림 영역은 검게 표시된다.

| 쪽 | 이미지가 포함된 Pattern 객체 | 관측 범위 |
| --- | --- | --- |
| 1 | 25 → image 24, SMask 23 | 로고 검정 채움 재현 |
| 2 | 없음 | 텍스트 대조군 |
| 3 | 43 → image 42, 45 → image 44 | 두 그래프 검정 채움 재현 |
| 4 | 54, 56 | 같은 패턴 구조, 브라우저 출력 확보 |
| 5 | 59, 61, 63 | 같은 패턴 구조 |
| 6~7 | 없음 | 패턴 없는 대조군 |
| 8 | 68, 70 | 같은 패턴 구조 |

총 10개의 사용 위치 모두 `/PatternType 1 /PaintType 1`인 coloured tiling pattern이다.
이미지는 페이지의 직접 XObject가 아니라 패턴의 `Resources/XObject` 안에 있다.
페이지에서는 `/Pattern cs /P1 scn f` 형태로 경로 내부를 채운다.

## 사양과 원인 경로

[ISO 32000-1:2008](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
§8.7.2, §8.7.3.1, §8.7.3.2에 따르면 coloured tiling pattern은 이미지도 포함할 수 있다.
패턴은 부모 콘텐츠의 초기 좌표계에 대한 `Matrix`, 셀 `BBox`, `XStep/YStep`으로 배치되며
현재 채움 경로에 제한된다. 페이지 콘텐츠 도중의 CTM을 패턴 원점으로 잘못 사용하는 것도
피해야 한다. Form에서는 Form의 초기 좌표계와 리소스가 기준이다.

현재 가져오기 경로에는 다음 두 문제가 함께 있다.

1. `vendor/rhwp-pdf-import-backend/src/path_color.rs:228`은 `Pattern`을
   `PathColorSpace::Unsupported`로 분류한다. `colour_operator`는 이름이 있는 `scn`의
   패턴 선택을 보존하지 않는다. `process_path_color_operator`는 RGB 결과가 있을 때만
   색을 갱신하므로 이전 단색이 남는다.
2. `extractors/paths.rs:71`의 기본 채움은 검정이며 `update_from_path_state`도 색 공간과
   무관하게 저장된 RGB를 다시 채움에 넣는다. 따라서 패턴 경로가 검정 단색 도형이 된다.
3. `document.rs:23235`의 `page_image_handles`는 페이지의 직접 `Do`·InlineImage 경로를
   처리하지만 패턴 채움에 연결된 리소스의 이미지를 수집하지 않는다.
4. `src/parser/pdf/extract.rs:77,162`의 경로·이미지 추출 결과를
   `src/parser/pdf/path_paint.rs`가 소비할 때는 이미 패턴의 의미와 이미지가 빠져 있다.
   이 상태의 채움 경로가 HWP 도형으로 변환된다.

backend의 독립 렌더러에 있는 패턴 처리와 rhwp의 PDF → Document IR 추출은 별도 경로다.
렌더러에 지원 코드가 있다는 사실만으로 가져오기가 지원된다고 판단할 수 없다.

## 원인을 분리한 실험

원본을 수정하지 않고 진단 PDF를 별도로 생성했다. 위 10개 위치의 패턴 채움을 동등한
직접 이미지 `Do`로 바꾸되 이미지 객체의 압축 데이터·SMask는 그대로 사용했다.
패턴 Matrix와 채움 경로를 반영한 좌표·clip만 직접 표현했다. 샘플의 단일 이미지 패턴에
한정한 실험 도구이며 제품의 문서별 우회 처리로 사용할 구현이 아니다.

| 검사 | 결과와 한계 |
| --- | --- |
| MuPDF 원본 ↔ 진단본 | 8쪽 전체에서 크기와 렌더링 픽셀 바이트가 동일 |
| 실제 Studio 원본 ↔ 진단본 | 모두 열기 성공, 8쪽. 1~4쪽 Canvas PNG 확보 |
| 1쪽·3쪽 직접 판독 | 원본은 검정 영역, 진단본은 로고·그래프 정상. 로고 투명도도 표시 |
| Native SVG 진단 | 원본 3쪽에 그림 대신 검정 채움 존재. 1~3쪽 전쪽 Native 검증은 아님 |

동일한 JPEG와 SMask를 직접 배치하면 정상 표시되므로 이 사례의 원인은 JPEG 디코더나
알파 데이터 손상이 아니라 **패턴 채움의 이미지 수집·배치 누락과 이전 단색 잔류**다.

조사 산출물은 Git 제외 경로 `output/black-image-20261031/`에 있다.
`make-direct-image-probe.py`, `direct-image-probe.json`, `direct-image-probe.pdf`가 변환 실험이며
`browser-probe.mjs`, `browser-probe.log`, `original/`, `direct/`, `reference/`가 실행 증거다.
외부 이슈에는 문서 내용·스크린샷을 업로드하지 않았다.

사용한 기존 최적화 WASM의 SHA-256은
`dcb50238ab65d703738030ff1498da5d6db46e9c74b50f1226f49825edc0527b`이다.
`pkg/`와 Studio `public/`의 바이트가 같다. 이번 수정으로 새로 빌드한 WASM은 아니다.
Native 진단 실행 파일은 `target/pr-review/release-test/rhwp`이며 SHA-256은
`23a36f263c925b8cbc03c9f1787ef73a936e8ca1c8a8138f004b9aa51e3215f2`다.
최종 head의 Native/fresh WASM 검증 및 전쪽 TSV 판정을 이 진단으로 대신하지 않는다.

## 선행 작업 대조

동일 증상의 이슈와 PDF·Pattern·tiling 관련 열린 PR을 검색했으나 직접 대응하는 해결 작업은
찾지 못했다. 관련 계보는 다음과 같다.

- [#8120](https://github.com/edwardkim/rhwp/pull/8120), `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`:
  PDF 가져오기 IR 지원.
- [#8190](https://github.com/edwardkim/rhwp/pull/8190), `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`:
  추가 이미지 코덱과 요청 예산.
- [#8360](https://github.com/edwardkim/rhwp/pull/8360), `f125c735c7d2d44eaa44b78c70465bc89af45fb1`:
  #8174의 PDF 지원·자원 경계 보강. 해당 검증 기록과 vendor 패치 설명에도 이번 패턴 이미지
  추출 완료 근거는 없고 현재 소스에서도 위 누락을 확인했다.

위 PR이 이 결함을 새로 만들었다는 회귀 판정은 하지 않는다. 현재 가져오기 경로에 남아 있는
지원 누락으로 조사했으며, 수정 시 기존 이미지·색 공간·자원 제한 계약을 보존해야 한다.
