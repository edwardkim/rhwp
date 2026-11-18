# #7733·#7735~#7738 원본 조판 보정 증적

현재 제품·검사 생산 커밋은 `211253376d20aa40c4cf223fdaabd87467e88240`, 고정 비교 base는
`197ce19e99438c2cba452c654ae1af504dab85ec`다. **최종 검증 진행 중이며 PR 준비 완료가 아니다.**
과거 단계의 통과 값은 해당 단계의 출처로 보존하고 최신 코드의 전쪽 완료로 대신하지 않는다.

## 원본과 변경 범위

| 이슈 | 원본 문서 | 쪽수 |
| --- | --- | ---: |
| #7733 | 화학제품 표시 기준 `samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp` | 103 |
| #7735 | `samples/한글문서파일형식_5.0_revision1.3.hwp` | 71 |
| #7736 | `samples/pr4093/outline_navigation_panel_demo.hwpx` | 3 |
| #7737 | `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` | 74 |
| #7738 | `samples/pr4093/outline_navigation_table_cell_number.hwpx` | 1 |

저장된 글줄·셀의 첫 프레임과 이어받기·종료 간격, 원본 글꼴 프로그램의 실제 윤곽과 전진,
글머리표·첨자·작은 기호·수식 공급자 및 원점을 공통 측정·배치·표시 경로에서 보정한다.
[원 해결 이력·원인별 분석](analysis.md)에서 각 단계의 독립 Print 대조와 정상 대조군을 연결한다.

원본 및 독립 기준 PDF를 변경하지 않는다. 공식 사양 PDF
`pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`는 재산출하지 않고 그대로 사용한다.
다른 기준은 데스크톱 한컴의 Hancom PDF Print이며 웹한글기안기·Microsoft Print to PDF와 구분한다.
[현재 커밋과 실제 검증 입력 52개 동일성](committed-input-audit-current.json)을 확인했다.
기존 base의 대용량 PDF는 동일 Git blob을 재사용하며 이번 PR에 새로 추가하거나 수정하지 않는다.

## 현재 완료한 추가 경계

- [sample4 26쪽 회색 배경 하단](sample4-gray-tail/cause.md): 본문에 수용한 마지막 줄의 후행 채움만
  보존한다. 페이지 배분과 외곽 테두리 clip은 유지한다. 새 반례는 보정 전 1 FAIL, 모듈은 보정 후
  6 PASS다. Native/fresh WASM 25쪽은 100.00%를 유지하고 26쪽은 각각 99.37→100.00%다.
  [실제 결과](sample4-gray-tail/validation.json),
  [WASM review](sample4-gray-tail/wasm/review-026.png)·
  [overlay](sample4-gray-tail/wasm/overlay-026.png)를 함께 보존한다.
- [원본 글꼴 우선 보호 조건](priority-original-font-qualification/README.md): nested 8쪽, 네 글꼴
  스타일 대조군 1쪽, sample16 HWP2022 38·43쪽, 영월 서식 1쪽을 Native/fresh WASM과 base/후보에서
  측정한 20쪽 모두 100.00%다. 후보의 같은 페이지 review·overlay 10쌍을 직접 판독했다.
  이는 18조건 전쪽 검증의 완료를 대신하지 않는다.
- Studio 분할 프로그램의 준비와 실제 CanvasKit 소비 경로는
  [수집 원인](split-host-font-preparation/cause.md)과
  [소비 경로](split-host-font-consumer/actual-controls/validation.json)에 구분한다.
  해당 실제 소비 증거의 원 생산 커밋을 최신 paint 코드로 재명명하지 않는다.

이 수치의 100%는 기존 **2px 허용 raw 실루엣** 판정이며 픽셀 동일이 아니다. sample4 하단 약 1px,
보호 대조군의 얇은 선·잉크·회색 밴드의 미세 차이를 직접 판독 기록과 분리하지 않는다.
Visual Sweep은 원본 글꼴을 공급하는 정상 환경에서 실행하고, Rust/Studio 회귀는 별도로 검증한
macOS 최소 글꼴 sandbox에서 실행한다. 비공개 원본 글꼴 바이너리는 이 증적에 포함하지 않는다.

## 제출 전 남은 검증

다섯 문서 전 252쪽과 원본 #4138 197쪽의 Native/fresh WASM, sample4 전 36쪽 기능 보존,
필수 편람 HWPX Native 383쪽, 정본 보호 18조건 1,771쪽의 경로별 전쪽 하락 금지 검증을 완료한다.
최신 코드의 전체 nextest·lint·build·Native Skia와 실제 Studio Canvas2D·지원 CanvasKit·SVG,
초기화 JS/WASM 응답 신원과 직접 판독도 확인한다. fallback을 요청한 backend의 성공으로 합산하지 않는다.
완료 뒤에만 최종 TSV·생산 영수증·대표 raw/PDF/review/overlay 및 PR 본문의 결과를 확정한다.
