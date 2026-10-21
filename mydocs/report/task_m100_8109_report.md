# #8109 결과보고 — Base14 Helvetica의 대체 선언과 실제 소비

- Issue: [#8109](https://github.com/edwardkim/rhwp/issues/8109).
- 날짜: 2026-10-21. 브랜치 `task_m100_8109`.
- 기준: `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`; Rust 구현 `5699a3778`와
  본 보고서가 속한 커밋의 Studio 소비 보정·최종 검증 증적.
- 수행·구현계획, 2단계, 다음 절편 진행 승인을 받아 승인 범위를 완료했다.
  결과보고 승인과 원격 게시·PR·merge·이슈 close는 별도 게이트다.

## 결과

프로그램이 없다고 입증된 표준 Type1 Helvetica 네 스타일에만 Arial 대체를 문서에 명시했다.
원본 이름과 Core14 AFM 측정, 글자모양은 보존한다. 실제 scoped PDF resource를 추적하므로
같은 이름의 다른 Form resource나 embedded/unknown 상태에 같은 정책을 적용하지 않는다.
HWP의 표준 alternative face와 독립 type byte를 공통 모델로 읽어 HWP/HWPX 저장·재열기가
소비하도록 했다. HWP3의 원래 이름용 alt_name을 전역 대체로 해석하지 않았다.

Studio 준비·paint가 전체 후보를 소비하도록 보정하고 후보 전체·style·host generation으로
캐시를 구별했다. 실제 CanvasKit이 잘못된 Black Cn 프로그램을 선택하던 경계에서 공급된
Arial을 선택하며, 세 진입 경로의 편집·역삭제·undo/redo와 저장 뒤에도 보존됐다.
글꼴 크기·장평·자간·alpha나 좌표를 조정하지 않았다. 측정과 실제 face의 근거를 구별하고,
Canvas2D에서 API상 관측할 수 없는 개별 glyph face는 `notObserved`로 유지했다.

원 해결 PR과 메인터너 보정을 추적한 뒤 기존 글꼴 추적 검사의 잘못된 HWP 부재 가정,
Node WASM의 오래된 ownership 기준, 초기화 순서·공급자 해제 검사의 현재 소비 계약을
보완했다. 원시 DocInfo의 독립 관측을 근거로 했으며 렌더링 baseline·허용치를 완화하지 않았다.
늦은 local-font 갱신의 코드 검토상 우려는 실제 전체 호출을 실행해 정상으로 확인했다.

## 최종 검증

| 항목 | 결과 |
| --- | --- |
| release build / release lib | PASS / 3,984 PASS·12 ignored |
| release-test 전체 | **11,215 PASS·0 FAIL**, 기존45 skip |
| Native Skia 공식3종 | lib4,041 PASS·12 ignored / missing-picture2 PASS / 직접 PDF14 PASS |
| Rust fmt·Native/WASM/workspace/all-target Clippy·workspace build | 전부 PASS |
| doc / TypeScript / Studio 전체 / 배포 build | 8 PASS·3 ignored / PASS / **2,350 PASS·0 FAIL·2 skip** / PASS |
| 최적화 fresh WASM | Docker·wasm-opt 포함 PASS, root pkg/Studio public 동기화 |
| 실제 브라우저 | font preparation/style·resource 실패 fallback·host provider 양 backend·편집·저장·늦은 local 갱신 PASS |
| Native/fresh WASM 전쪽 | 각37개 모두 쪽수1=1·90% 이상, 최저 **99.63470% / 98.35575%** |
| SVG 소비 경계 | 각3개 모두90% 이상, 양쪽 최저99.14358% |

전쪽 TSV와 대표 review/overlay gate, 같은 영역4배 판독을 함께 확인했다.
원본/대조군의 frozen base 대비0.1%p 하락은 없다. 원문·목록·X 삽입·후속 본문·네 스타일·
교차 저장과 새 source-only font 입력을 확인했다. 새 입력도 독립 Print·양 backend100%였다.
기존 body-list 검사들은 이미 소속·순서·독립 PDF의 원점 차이 등 관계 검사여서 유지했다.

## 증적과 수용 범위

- [1단계 독립 근거·선행 PR](../working/task_m100_8109_stage1.md),
  [2단계 구현·수정 전5 FAIL/후6 PASS](../working/task_m100_8109_stage2.md),
  [3단계 실제 경로·전체 명령·실패 보정·판독](../working/task_m100_8109_stage3.md).
- [최종 조건·대표 이미지](../pr/assets/issue_8109/stage3/README.md),
  [입력/Print·전쪽 TSV80행](../pr/assets/issue_8109/stage3/inputs.tsv),
  [소스·실행 파일·font·PNG 해시](../pr/assets/issue_8109/stage3/producer.tsv).
- 원문 TSV85개·대표 PNG28개, 검증 저장본16개·독립 Print17개를 보존했다.
  최종 Studio 저장본18개는 기존 Print 입력과 바이트가 같았다. 로그·중간 JSON은 ignored output에 있다.

새 PDF import와 유효한 대체 선언의 저장·재열기·실제 소비가 수용 범위다.
대체 선언 없이 과거에 저장된 HWP의 PDF 출처를 역추정하거나 embedded 프로그램 렌더링을
새로 지원했다고 주장하지 않는다. 비적용 program 대조군의 기존 font·농도 차이는 남으며
같은 조건의 base 대비 악화는 없다. #8106–#8108·#8110과 다른 문서의 미해결 범위를 닫지 않는다.

승인된 로컬 작업은 충족했다. 사용자 작성 문서·서버와 공용 빌드 cache를 보존했고,
원격 push·PR 생성·GitHub 게시·merge·이슈 close는 수행하지 않았다.

![실제 Studio의 잘못된 face 보정과 독립 Print 비교](../pr/assets/issue_8109/stage3/studio-zoom.png)
