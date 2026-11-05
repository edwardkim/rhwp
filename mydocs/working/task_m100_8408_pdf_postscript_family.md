# PDF PostScript 이름과 HWP 글꼴 family의 구분

## 문제와 선행 확인

PR #8408의 무표기판 원본 `samples/21_언어_기출_편집가능본.pdf`를 편집 가능한 HWPX로
가져온 입력에서 새 base와 후보 모두 전15쪽 중 12쪽이 90% 미만이다. 1쪽 88.18671%,
5쪽 89.01597%, 13쪽 89.05578%다. 후보만의 회귀로 분류하지 않으며 원 PR의 공백 줄
높이 보정·독립 본문 원점 계약도 제거하지 않는다.

현재 제품으로 같은 PDF를 다시 가져온 ZIP 내부 파일은 이전 제출 HWPX와 전부 같았다.
입력 재생성만으로는 해결되지 않았다. 독립 PDF는 한컴 Viewer의 macOS Print이고,
PDF를 다시 출력하거나 바꾸지 않는다.

## 원인

원본 PDF는 `HCRBatang`, `HCRBatang-Bold`, `HCRDotum`, `Haansoft-Dotum`을 쓴다.
포함된 subset SFNT에는 Mac 플랫폼 name ID 6만 있고 family ID 1·16은 없다.
따라서 `program_family`가 family를 읽지 못하고, `imported_font_family`는 PostScript
이름을 그대로 HWP family에 넣는다. 이 이름은 정식 family와 다르다.

가져오기 글자 맞춤은 공통 정적 메트릭으로 비율·자간을 계산한다. 반면 PR #8565는 실제
PostScript 이름으로 공급된 원본 TTF도 인식하므로, 출력에서 별칭을 못 찾은 추정 폭 대신
실제 프로그램 폭을 쓴다. 예전에는 두 경로가 같은 추정값을 썼으나 이제 글자 위치가 달라진다.
실제 프로그램 인식을 되돌리는 대신, PDF BaseFont를 HWP family로 낮추는 기존 공통 경로를 고친다.

## 독립 이름 근거와 수정

한컴 설치 프로그램의 OpenType name ID 1·2·6을 읽어 다음 대응을 확인했다. 프로그램은
시스템에서 조사에만 사용하고 저장소에 추가하지 않는다. PDF subset 이름의 임의 접미사를
삭제하거나 이름이 비슷하다는 이유로 family를 추정하지 않는다.

| PostScript name ID 6 | family name ID 1 | style name ID 2 |
| --- | --- | --- |
| HCRBatang | HCR Batang | Regular |
| HCRBatang-Bold | HCR Batang | Bold |
| HCRDotum | HCR Dotum | Regular |
| HCRDotum-Bold | HCR Dotum | Bold |
| Haansoft-Dotum | Haansoft Dotum | Regular |

`lowering::imported_font_family`의 기존 ArialMT→Arial과 같은 위치에서 이 확인된 대응을
적용한다. PDF의 독립 굵기 속성은 유지한다. embedded SFNT에서 실제 family를 읽은 경로는
종전처럼 그 값을 우선하며, 미확인 이름은 그대로 둔다. 공통 family 해소 뒤 가져오기·저장·재열기·
Native/fresh WASM 측정이 같은 확인된 글꼴 메트릭을 소비하는지 검사한다.

## 검증 상태

독립 이름·설치 프로그램 SHA-256 목록은 ignored output의
`logs/pr8408-postscript-family-independent.json`에서 조사했다. 해당 해시와 수정 후 결과는
최종 검증 뒤 이 기록에 보완한다. 현재 수정 후 시각 통과나 PR 준비 완료를 선언하지 않는다.
최종 코드의 실제 API로 HWPX를 다시 생성하고, 세 PDF 입력 전체와 굵기·첨자·공백 줄 대조군을
검증한다. 기존에 공개한 저장본은 현재 API가 만든 최종 증적과 구분한다.

## 조사한 원 프로그램의 신원

| 프로그램 | SHA-256 |
| --- | --- |
| `HCRBatang` | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` |
| `HCRBatang-Bold` | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` |
| `HCRDotum` | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` |
| `HCRDotum-Bold` | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` |
| `Haansoft-Dotum` | `2989f37963a66f865ff9875bb5da668593dd297da03a0d6a727d8da3c093eb8b` |

## 첫 Native 집중 검증

제품 `874fac36f2`에서 같은 원본 PDF를 실제 importer로 재생성한 뒤 Native로
1·5·13쪽을 측정했다. 세 쪽 모두 98% 이상이며 최저는 98.54%다. PDF는 그대로
사용했고 일반 시스템 글꼴을 제공했다. 전15쪽과 fresh WASM은 아직 이 결과로
대신하지 않는다. 수정 전 같은 세 쪽은 88.18671%·89.01597%·89.05578%였다.

## 새 base에서 Native 전46쪽 완료

제품 `2071603998`의 실제 API로 세 PDF를 다시 가져온 뒤 Native 전쪽을 측정했다.
2010판 16쪽 최저 97.80%, 2020판 15쪽 최저 99.19%, 무표기판 15쪽 최저 98.54%로,
전46쪽에 90% 미만·누락이 없다. 2010·2020 HWPX는 이전 제출본과 전체 바이트가 같다.
무표기판은 header·section의 family와 관련 맞춤만 바뀌었으며 새 저장본 SHA-256은
`5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6`이다.

이 실제 저장본을 정식 `editable-plain.hwpx`에 반영했다. 원본 PDF는 모두 그대로다.
전쪽 TSV와 원 주장 5·13쪽의 실제 Native review·overlay 및 신원은
`mydocs/pr/assets/issue_8381/integration_20261103/native-postscript-family.md`에 연결했다.
fresh WASM·최종 통합 제품 검증과 개별 리뷰 전체 갱신은 아직 완료하지 않았다.

## 최적화 fresh WASM의 실제 가져오기·저장 확인

제품 `dd0ac546e3389f1703e67cb86f1c5bdf615ff0c9`의 최적화 package에서
`HwpDocument.importPdf(..., {layout: 'editable'})`와 `exportHwpx()`를 실제 실행했다.
세 판본의 저장 HWPX SHA-256이 위 Native 제출본과 각각 같았다. 무표기판도
`5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6`으로 같다.
이는 새 family 보정이 실제 WASM API에서도 같은 저장본을 만드는 증거다.
같은 저장본의 전쪽 fresh WASM 렌더링·독립 PDF 확대 판독은 아직 별도로 완료해야 한다.
원시 결과는 ignored `logs/rebase-782ab3e2d0-wasm-producer-parity.log`에 있다.

## 새 저장본의 fresh WASM 전쪽과 직접 판독

같은 최적화 제품 `dd0ac546e3`으로 세 판본의 전46쪽을 측정했다. 2010판 16쪽 최저
98.08%, 2020판 15쪽 99.18%, 무표기판 15쪽 98.44%로, 90% 미만·누락이 없다.
각 독립 PDF와 전체 쪽수도 같다. 실제 원 주장인 무표기판 5·13쪽의 review·overlay를
현재 PNG에서 생성해 직접 대조했다. raw는 각각 99.68359%·99.69047%다.

공백 줄 뒤 본문과 문항·보기의 위치, 줄바꿈·상자 경계·내용 보존을 확인했다.
13쪽 쪽번호 외형의 기존 한계는 Native 보고서와 동일하게 남기며 본문 배치와 구분한다.
전쪽 TSV·같은 쪽의 대표 PNG와 제품 해시는
`mydocs/pr/assets/issue_8381/integration_20261103/native-postscript-family.md`에 보완했다.
새 base Native 제품의 재확인과 최종 공통 검증은 아직 진행 중이다.
