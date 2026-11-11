# #8946 호스트 글꼴 조판 참여 — 구현과 검증 진행 기록

수정된 [#8946](https://github.com/edwardkim/rhwp/issues/8946)의 2026-11-11 범위와
수용 조건을 기준으로 구현했다. 현재 판정은 **진행 중 / re_review_required**다.
기능 검사 통과를 전쪽 시각 수용 조건 통과로 대신하지 않으며, PR을 제출하지 않았다.

## 변경과 고정 기준

- 기준 devel: `99b9d7ec5ca36884ea8fd981ac815c1edc34c653`.
- 계획: `d5965ded03`, 구현: `7676b601af`, 다중 face 보정: `dcc51ffb8e`,
  UI 저장·이력 검사: `19d9f6c3da`, 새 문서 대조: `d3f5e549c7`.
- `setProvider(provider, { layout: true })`로 연결 단위의 Canvas2D 조판 참여를
  선택한다. 옵션 생략·false·CanvasKit의 기존 공급 계약을 유지한다.
- 실제 파일 SHA-256과 TTC faceIndex를 프로그램 ID에 넣고, 기존
  `buildFontEnvironmentFromAvailablePrograms → setFontEnvironment`를 소비한다.
  자동 `preserveSourceMetrics`와 사용자 선택 `replaceSourceMetricsFor`는 분리한다.
- 최초 열기와 공급 갱신이 같은 환경 준비 함수를 호출한다. 연결·문서 세대를 검사해
  지연 응답을 폐기하고, 해제 시 소유한 자동 환경과 face·측정 캐시를 갱신한다.
- 동일 query에 여러 파일이 매칭되는 실제 521-face 목록에서 Rust snapshot 충돌을
  발견했다. 기존 선언 프로그램 순서에 따라 query별 단일 identity를 선택하도록
  보정했으며, 화면의 분할 face 공급은 기존 경로를 유지한다.
- Rust·Native 렌더러는 변경하지 않았다. 원본 저장 LineSeg를 일괄 폐기하거나
  HFT 메트릭 정책을 바꾸지 않았다.

## 실행한 기능 검증

TypeScript 검사 통과, Studio 전체 2,787 PASS / 2 skip / 0 FAIL, 호스트 집중 검사
19 PASS다. 최적화된 fresh WASM과 Native Skia 빌드를 완료했다. 기존 #7403 실제
Chrome 156 Canvas2D·CanvasKit E2E와 추가 API/환경 검사를 통과했다.

추가 검사는 실제 파일 식별, 같은 ID의 A→B 파일 교체, TTC face 교체, 실패·제거·재공급,
옵션 해제, 명시적 환경 보호, 지연 중 옵션·공급자·문서 교체, 영속 로컬 목록 독립성을
포함한다. 실제 편집 UI에서 옵션 활성화 전후 저장 바이트·dirty·undo/redo 스택과
실제 undo/redo 실행도 확인했다. 이 검사는 합성 프로그램의 API·수명 계약이며,
HWPX의 한컴 출력 일치나 조판 관계 검증을 대체하지 않는다.

추가 실행에서 활성화·A→B 파일 교체·TTC face 교체·제거·재공급·비활성화의
합성 문서를 같은 상태로 새로 만든 문서와 비교했다. 여섯 상태 모두 전체 레이어 트리와
실제 Canvas2D paint가 일치했고 Canvas2D·CanvasKit E2E를 다시 통과했다.

공개 API를 사용한 프로덕션 HWPX 최초 열기에서 딸기/nativeOtf를 선택했다.
공급 갱신→제거→재공급→해제 시 딸기→돋움→딸기→돋움으로 복귀하고,
적재 face 수는 1→0→1→0이었다. 같은 HWP3 대조도 수행했다.
최종 구현의 전체 실제 목록에서는 적재 67 / pending 0 / failed 0이며
1·608·757쪽 실제 화면을 별도로 확인했다.

## 기본 호환성

고정 base와 후보에서 같은 WASM·원본·글꼴·Chrome·Canvas2D 조건으로 공급자 없음,
옵션 생략, 명시적 false를 각각 실행했다. **세 조건 모두 763쪽의 PNG와 전체 레이어
트리 SHA-256이 동일**했다. Rust 소스가 같으므로 base 프로덕션 번들은 변경된 네
기존 TypeScript 모듈을 고정 base 내용으로 공급해 만들고 같은 fresh WASM을 사용했다.
새 helper는 base 모듈의 import 경로에 없다. 별도 base Rust 빌드라고 보고하지 않는다.
전체 해시 원본은 [증거 폴더](../pr/assets/issue8946/implementation-20261111/)에 있다.

## 전쪽 시각 검증과 미달

PDFium 96dpi, raw 2px, HWP3 호환 2022 / HWPX 호환 2024 조건이다.

| 대상 | 쪽수 | Native 최저 | fresh WASM 최저 | 판정 |
| --- | ---: | ---: | ---: | --- |
| HWP3 sample10 / 한컴 2020 Print | 763 | 98.21% | 99.55% | 전쪽 90% 이상 |
| HWPX sample10 / 새 한컴 2024 Print | 763 | 9.55% | 30.05% | Native 19쪽 / WASM 43쪽 미달 |
| 행정업무운영 편람 / 공개 최소 글꼴 | 383 | 99.96% | 해당 없음 | 보호 baseline 대비 368쪽 하락 |

[전쪽 TSV·판정](../pr/assets/issue8946/implementation-20261111/validation.json)을
보존했다. 표의 WASM은 표준 exporter 경로이며 Studio의 새 host API를 호출하지 않는다.
실제 host API의 화면·수명 증거와 구분한다. 새 host 경로 전쪽 검증의 대체 근거가 아니다.

HWPX 독립 Print는 원본 바이트 전처리 없이 한컴 2024 13.0.0.3901,
`frame_print_to_pdf_ex_one_up`으로 763쪽을 생성했다. 파일은 90,781,157바이트,
SHA-256 `d1f5b5e4047e6650277075e6cd8591e2b3f97d501028d4a15d5a18b280deea4a`다.
50 MiB 이상 PDF의 로컬 보관 규칙에 따라 `pdf/issue8946/hwp3-sample10-hwpx-2024.pdf`에
보존하며 커밋하지 않는다. [변환 근거](../pr/assets/issue8946/implementation-20261111/independent-hwpx-print.json).

독립 Print 서버는 원본의 세 외부 그림에 접근하지 못해 첫 쪽에 그림이 없다.
Native는 원본 옆 파일 부재로 placeholder, WASM은 정식 samples의 세 그림을 공급했다.
따라서 첫 쪽은 공급 조건도 다르다. 그림 제외 마스크는 적용하지 않았다.
첫 쪽 외에도 제목 폭 차이가 남아 있다. 608쪽의 [독립 Print](../pr/assets/issue8946/implementation-20261111/hwpx-print-p608.png),
[표준 WASM](../pr/assets/issue8946/implementation-20261111/hwpx-wasm-p608.png),
[실제 host 화면](../pr/assets/issue8946/implementation-20261111/hwpx-host-p608.png)을
직접 비교했다. 전체 출력·HFT 메트릭 변경은 이번 이슈의 완료 범위가 아니므로,
점수를 맞추기 위해 원본 보존 정책을 꺼서는 안 된다. 동일 글꼴 환경의 Print 여부와
원본 저장 조판/실제 파일의 대응을 추가 확인해야 한다.

608쪽 첫 글자의 코어 trace에서 전체 목록 공급은 `#세나루 → 세나루`,
`heuristicHalfwidth`를 보고하고, 원래 이슈의 딸기 한 파일 공급은
`#세나루 → 돋움`, `embeddedMetric`을 보고한다.
[trace](../pr/assets/issue8946/implementation-20261111/hwpx-p608-font-trace-summary.json)는
Studio paint snapshot을 포함하지 않으므로 최종 그리기 프로그램의 증명으로 승격하지 않는다.
서버 Print의 제목은 Type 3 글꼴이며, 서버의 실제 글꼴 파일과 공급 OTF의 동일성은
확인되지 않았다. 제목 차이의 원인과 올바른 기대값을 아직 확정하지 않았다.

편람 최소 공개 글꼴 실행은 368쪽만 보호 baseline 100.00%에서 99.97%로 하락한다.
미일치 13픽셀은 접수 일시의 U+2024 점 세 개에 집중된다.
[확대 비교](../pr/assets/issue8946/implementation-20261111/handbook-p368-detail.png).
현재 자동 환경과 과거 통과 환경은 scoped glyph fallback·프로그램 선택 구성이 다르다.
원인을 새 Studio 변경으로 확정하거나 환경을 임의로 과거 값에 고정해 통과 처리하지 않았다.
Rust 소스가 base와 같다는 사실만으로 정본 gate 실패를 면제하지 않는다.

## 남은 수용 항목

1. HWPX의 동일 공급 환경 독립 Print 근거를 확보하고 미달 페이지 원인을 판정한다.
   Downloads의 `microsoft_pdf_sample10.pdf` 원본 출처는 사용자 확인을 기다리고 있다.
2. 편람 368쪽의 선택·glyph fallback 경로와 원 해결 근거를 추적해 정본 gate를 통과한다.
3. HWPX/HWP3 저장 조판과 편집 재조판에서 소속·내용 순서·포함·겹침·후속 본문을
   직접 검증한다. 각 수명 전환을 같은 상태로 새로 연 문서와 대조한다.
4. 실제 host 경로의 최종 전쪽 증거와 확대 판독을 완료한다. 이후 해당 로컬 제출 게이트를
   완료하고 제출 여부를 판단한다. 현재 수치는 구현 완료나 PR 제출 승인을 뜻하지 않는다.

개인 글꼴 바이너리·브라우저 프로필·사설 변환 연결 정보는 증거에 포함하지 않는다.
진행 중 원본 로그와 재현 스크립트는 T7에 연결된 `output/issue8946/`에 보존한다.
