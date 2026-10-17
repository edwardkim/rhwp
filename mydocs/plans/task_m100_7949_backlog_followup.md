# #7949·#7950 백로그 후속 수행·구현계획

- 날짜: 2026-10-17 (Asia/Seoul)
- 지시: `mydocs/plans/task_m100_7949_backlog.md`의 후속 처리 착수
- 지정 백로그: `b7a5388034ee66f58a7e1d5363974e83b6374f57` — 현재 devel의 문서와 동일
- 작업 기준: fetch한 `upstream/devel` `515bd72dcde0eecdeece333666b675354b7fa91d`
- 로컬 브랜치: `task_m100_7974`
- 작업공간: `/home/edward/mygithub/rhwp-backlog7949`
- 상태: 2026-10-17 사용자 계획·이슈 등록·첫 구현 승인. 후속 이슈 [#7974](https://github.com/edwardkim/rhwp/issues/7974) 등록 후 구현 진행
- 이전 이슈 #7949·#7950은 CLOSED 상태로 유지한다. 새 후속 번호를 예측하지 않는다.

## 1. 확인한 범위와 분할 순서

원래 다섯 백로그 행을 하나의 거대한 완료 주장으로 묶지 않는다. 다음 순서로 독립적인
완료 조건을 가진 절편을 처리하며, 검증한 부분과 남은 범위를 원래 백로그에 연결한다.

| 절편 | 대상 | 현재 확인·완료 조건 |
|---|---|---|
| 입력 확보·글꼴 진단 | MOF 1쪽의 글꼴·미세 간격 | 원본 PDF 현재 경로 확인 후 최신 devel로 실제 차이를 다시 측정. #7970의 PDF 글꼴 이름·공백 보정과 중복 구현하지 않음 |
| 첫 구현 | 이미지 마스크: 색상 키 `/Mask`, 독립 stencil, 저해상도 확대 | 명시적 미지원 두 유형을 요청 예산 안에서 지원하고, Decode·원색 샘플·현재 칠 색상·Interpolate의 의미가 최종 그림까지 전달되도록 보정 |
| 다음 구현 | 부분 투명 텍스트·graphics-state soft mask·blend·텍스트 clip | 열기 성공과 실제 paint 보존을 구분. 원본 순서·alpha·clip·합성 상태를 보존하는 지원 단위별로 구현·검증 |
| 다음 구현 | 배경과 겹치는 표의 의미 승격 | 현재의 positional 보존을 유지한 채, 원본 paint 순서를 입증할 수 있는 표·배경 그룹만 편집 가능한 표로 복원 |
| 개별 수용성 감사 | 추가 codec | decoder 존재·Cargo feature·실제 bounded adapter 경로·요청 예산을 구분. 구체적인 codec·입력·예약 경계가 정해지기 전 일괄 활성화하지 않음 |

공개 가능한 동일 증상의 열린 이슈·PR을 검색했다. `PDF 가져오기`, `Mask`, `paint`,
`pdf-import` 검색에서 이번 제한된 import 마스크 후속을 직접 처리하는 열린 PR은 찾지 못했다.
멀티 렌더러 트래킹 #536이나 HWP/HWPX 배치 이슈를 이 백로그 전체의 대체 이슈로 삼지 않는다.

## 2. 수정 전 실제 실행

같은 devel source의 `native-skia` 라이브러리를 재사용해 자체 작성 PDF 7개를 공통
`DocumentCore::from_bytes` → Native Print PNG 경로로 실행했다. 기준은 입력 PDF의
Poppler 96 DPI 출력이다. 진단용 소스·입력은 ignored 경로에만 두었으며 정식 회귀나
golden으로 등록하지 않았다.

| 입력 | 실제 결과 | 독립 기준과 직접 판독 |
|---|---|---|
| unmasked-control | 1쪽 열기 성공 | 무마스크 정상 대조군. 정식 시각 gate는 아직 미실행 |
| colour-key | `PDF_IMAGE_UNSUPPORTED` | `Mask::read`가 색상 키 배열을 거절하는 경로를 실제 확인 |
| standalone-stencil | `PDF_IMAGE_UNSUPPORTED` | `PdfImageHandle::decode`의 `/ImageMask` 거절을 실제 확인 |
| low-resolution-mask | 1쪽 열기 성공 | `/Interpolate false`의 날카로운 마스크 경계가 Native에서 넓게 흐려짐 |
| partial-text-alpha | 1쪽 열기 성공 | `/ca 0.5`의 회색 글자가 Native에서 검정으로 출력됨. 글꼴 모양 차이와 별도 결함 |
| graphics-state-soft-mask | 1쪽 열기 성공 | Luminosity mask로 가려져야 할 오른쪽 빨강 영역이 Native에서 표시됨 |
| multiply-blend | 1쪽 열기 성공 | 겹침 영역의 기준 검정이 Native에서 파랑으로 출력됨 |

진단: `output/pr-review/backlog-7949/baseline/`의 `generate.py`, `probe.rs`,
`native-results.tsv`, `source.json`, 입력 PDF 및 페이지 PNG.
사용자 확인용 복사본: `E:\tmp\issue7949-backlog-review-20261017\index.html`.
이는 수정 전 Native 진단이며 fresh Docker WASM이나 전쪽 시각 gate 완료가 아니다.

MOF 원본 `/mnt/e/tmp/261016-mof-69009.pdf`는 현재 없다. 이전 작업 아카이브의
MOF 비교 PNG·TSV와 글꼴 경로는 확인했지만, 이 자료만으로 최신 source의 원본 재검증을
대체하지 않는다. 사용자가 보내 준 blob은 백로그 문서의 revision이며 PDF 원본 경로는 아니다.
원본 확보가 필요한 이 항목과 자체 입력으로 진행 가능한 마스크 절편을 구분한다.

## 3. 첫 절편의 실제 소비 경로와 구현 계약

### 3.1 이미지의 색·alpha

생산은 `vendor/rhwp-pdf-import-backend/src/extractors/images.rs`의 이미지 handle과
`image_mask_limited.rs`의 bounded mask decode다. 제품 소비는
`src/parser/pdf/extract.rs` → `lowering.rs`의 그림 생성 → 공통 Document IR →
최종 Native Skia·WASM/Studio image paint다. backend의 일반 렌더러에 유사 구현이 있어도
bounded import handle이 실제 소비하지 않으면 지원 완료로 간주하지 않는다.

- 색상 키 범위는 색 변환·Decode 전의 원래 성분 샘플과 비교한다. DeviceRGB 한 사례의
  변환 결과만 맞추지 않으며 Gray·Indexed·packed depth의 적용/비적용 경계를 확인한다.
  `/SMask` 우선순위, 잘못된 배열 길이·범위·참조, 기존 alpha와 조합도 구분한다.
- 독립 stencil은 명시적 이미지 `/Mask`와 같은 비트 극성을 사용하되, 현재 graphics state의
  nonstroking 색상·alpha를 실제 handle에서 받는다. 검정 고정이나 전체 색상 추측으로 대신하지 않는다.
- 버퍼·packed row·합성·색 변환·작업 수는 checked 산술과 실제 예약 이전의 요청 예산으로
  제한한다. 기존 sticky 실패·취소·시간 제한을 보존한다.

### 3.2 확대 sampling

현재 mask sample 합성은 `/Interpolate`를 읽지만, 최종 PNG 확대까지 같은 선택이
전달되는지는 별도 계약이다. 수정 전에 `Mask::apply`의 RGBA → PNG/BinData →
Picture/layer image → 각 backend sampling 선택을 추적한다. mask 내부 RGBA가 정상이라는
사실로 최종 paint를 통과 처리하지 않는다.

보정은 원본 크기·CTM·clip·소유 페이지를 보존하고, 명시적 sampling 의미를 전달하는
공통 결과에서 처리한다. 특정 문서·확대 배율·목표 점수에 맞춘 가짜 해상도나 좌표 보정은
사용하지 않는다. 기존 HWP/HWPX의 기본 sampling 및 저장·재열기 계약도 대조한다.

### 3.3 반례

지원 유형마다 정상·반전 Decode, 바이트 경계를 넘는 너비/행 패딩, 불일치 크기,
이미지 색과 stencil 칠 색상, `/SMask` 우선순위, alpha 0/중간/1, 중첩·순환 참조,
길이 부족·과도한 선언·예산 실패를 적용되는 범위에서 검사한다. 무마스크 및 기존 명시적
Mask/SMask/Matte를 정상 대조군으로 유지한다. 손상 입력을 정상 그림으로 조용히 복구하지 않는다.

## 4. 검증과 제출

1. 위 수정 전 증거를 보존한다. 구현마다 영향 입력의 작은 Native 출력으로 보정 방향을 먼저 확인한다.
2. Native와 표준 Docker fresh WASM에서 같은 입력·독립 기준·글꼴·Print profile로 전쪽 TSV를
   생성하고 해당 영역을 확대 판독한다. 90%·쪽수·직접 판독과 최신 devel 대비 0.01%p 하락 기준을
   함께 적용한다. 신규 지원 입력이 base에서 거절되면 기준 TSV 불가를 명시하고 별도 범위 판정을 받는다.
3. 공개 입력·검증 범위 양쪽 90% 이상을 확인한 뒤 기존 자동 수집을 따르는 `tests/cases/`의
   관계형 회귀를 추가한다. 수정 전 같은 원인 FAIL / 수정 후 PASS를 입증하며 pixel/hash로
   잠정 출력을 고정하거나 미달 baseline을 등록하지 않는다.
4. 승인된 최종 소스에서 local_validation.md 4.3의 parser/renderer 해당 lint·회귀·Skia·WASM·
   Studio/Worker 검증을 수행한다. 전체 CI 성격의 긴 검증은 해당 승인 단계에서 실행한다.
   Cargo는 `/home/edward/mygithub/rhwp/target/pr-review`만 재사용한다. 개발 서버 7700과 기존 WIP는 보존한다.
5. 단계 변경 전 변경·결과를 로컬 commit한다. PR 직전 최신 devel 확인·필요한 통합·영향 재검증을
   한 번 정리하고 원격 push·PR·CI·self-review·merge 승인을 각 실제 단계에 맞게 처리한다.

개인 입력·폰트·로그·중간 JSON/TSV를 공개하지 않는다. `parserAllocationBoundsProven=false`는
보존하고 이 절편으로 전체 PDF paint·모든 codec·보안 감사 완료를 주장하지 않는다.

## 5. 첫 후속 이슈 등록안

제목: `fix(pdf): 색상 키 Mask·독립 stencil 가져오기와 확대 sampling 보존 (#7949·#7950 후속)`

- 배경: 닫힌 #7949·#7950의 후속 백로그에서 실제 남은 마스크 경계를 분리한다.
- 재현: 위 자체 작성 1쪽 PDF. 두 유형은 `PDF_IMAGE_UNSUPPORTED`, 저해상도 명시적 Mask는
  열리지만 `/Interpolate false`의 최종 경계가 흐려진다. 수정 전 source는 위 기준 SHA다.
- 범위: 3절의 원색 샘플·현재 칠 색상·요청 예산·최종 sampling 소비 계약.
- 완료: 오류 없는 열기뿐 아니라 Decode/alpha/배치 보존, 독립 원본 PDF의 Native/fresh Docker
  WASM 전쪽 시각 검증·직접 판독, 실제 정식 회귀와 정상 대조군.
- 제외: 이 이슈가 graphics-state soft mask·blend·텍스트 clip·전체 표 의미 승격까지 완료한
  것으로 표시하지 않는다. 해당 항목은 1절의 후속 절편에 남긴다.

사용자 승인으로 #7974를 등록했다. assignee edwardkim, milestone v1.0.0, bug·pdf 라벨과 본문 UTF-8을 API 재조회했다. 원격 push·PR은 아직 수행하지 않았다.
