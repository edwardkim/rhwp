# seo-rii PR 9건 검토·통합 계획

## 대상과 승인 범위

2026-11-09 작업지시자 지시: seo-rii 기여자의 PR 처리를 시작한다. 열린 Ready PR 9건을 대상으로
maintainer_general + intake_and_review·local_validation·multi_pr_update_branch·visual_fixture_evidence·
post_merge를 적용한다. 원 PR별 reviewer edwardkim을 지정하고 API로 확인했다.
이번 지시는 접수·로컬 검토·검증 범위이며 원격 push·댓글·통합 PR·병합·close는 별도 승인 뒤 수행한다.

- 기준: `upstream/devel@06181802e574dbd91e62375536751dea8c270fcd`.
- 작업공간: `/home/edward/mygithub/rhwp-worktrees/review-integration-536-seo-rii-preview-flutter`.
- 브랜치: `review/integration-536-seo-rii-preview-flutter`.
- 원 head는 `refs/rhwp-review/seo-rii-20261109/pr-<번호>`에 보존했다.

| PR | 범위 | 고정 head |
| --- | --- | --- |
| #8749 | feat(renderer): Canvas2D strict-outline 미리보기 이관 | `0c8de92939ada977d96e9d3608f13f4fdd707cf3` |
| #8761 | feat(flutter): connect safe worker saves and recovery (P103) | `b8c948e7fb4d7bdf9c20e20e7551aa879151a810` |
| #8764 | feat(flutter): guard reference document lifecycle (P104) | `97e532e8031b1dcc777923120a618f76c13cb2c8` |
| #8765 | feat(flutter): add bounded multi-page CPU viewport (P105) | `dd62a95be83474ebbb9635f8c1bb8840c32ab3c9` |
| #8767 | test(flutter): qualify actual Windows texture consumer (P106) | `2e6268c8187ed5eda59f19ca4cb729371c1776da` |
| #8769 | test(flutter): add actual Apple SDK Metal producer probe (P107) | `40e1301ddabb6290a4ed06c5791e8fd4960403f5` |
| #8777 | test(flutter): qualify actual Linux consumer and continuous GL frames (P108) | `def466097d790671247e30c1ec412931ad606e6a` |
| #8779 | feat(flutter): add continuous Windows ownership transfer (P109) | `96a1b3b044cd3e979d24759526e5a0cc6ceaff40` |
| #8781 | feat(flutter): add continuous AppKit Metal presentation (P110) | `99732cf8ca32f2835ddd6f486e7b8a0c12346b3e` |

## 선행 관계와 검증 절차

1. #8749는 독립 Canvas2D strict-outline opt-in 변경이다. P103–P110의 8개 head는 차례로 선행
   head를 포함하며 #8781이 해당 원 이력을 모두 보존한다. 최신 devel과 각 head의 merge-tree는
   모두 종료 코드 0이다. 이를 이미 해결됐다는 증거로 사용하지 않고 실제 코드·동작을 대조한다.
2. 계획 commit 뒤 최신 base에 #8749와 #8781을 로컬 merge해 원 기여 이력을 유지한다. 누적 충돌이
   생기면 정확한 경로·기존 계약·보정안을 정리하고 충돌 해결 승인을 받는다.
3. 원 해결 PR #8739·관련 #536/#8701과 후속 보정 이력을 추적한다. 기본 API·저장/수명·자원
   한도·glyph fallback·PDF TextPaint·현재 devel 기능 보존을 확인한다.
4. #8749는 선택 계약·실제 최적화 WASM Chromium·공개 문서 API·원 입력의 Native/fresh WASM
   전쪽 TSV와 독립 출력·직접 판독을 검증한다. 기본 opt-in 비활성 출력과 명시 활성 출력을 구분한다.
5. Flutter는 Python/C++ 계약·Dart analyze/test, Linux 실제 consumer 표시·저장·종료·연속 프레임을
   검증한다. Windows 호스트 SDK·화면 실행 가능성을 확인한다. macOS/Windows의 제출된 실제
   Actions 로그·artifact·컴파일/실행 source와 Native 재사용의 입력 동일성을 대조한다.
   제출된 green CI와 스크린샷만으로 메인터너의 실제 출력 검증을 대체하지 않는다.
6. 실제 SDK·GPU consumer·Native producer·메모리/펜스·실물 성능의 입증 범위를 나눈다. 예전
   #8726의 실험 기반 예외를 새 연속 표시 PR의 검증 면제로 옮겨 쓰지 않는다. 필요한 환경·증거가
   부족하면 해당 PR의 보류 조건과 구체적인 해제 방법을 남긴다.
7. source 보정 없는 exact green CI 재사용 여부는 정본 3.2.2의 조건으로 판정한다. focused·실제
   WASM·브라우저·시각 검증은 생략하지 않는다. 새 제품 보정이 필요하면 보정 담당 범위를 먼저
   확정하고 새 head의 필수 검증을 완료한다.
8. 원 PR별 review와 공통 검증 원장을 작성한다. 결과를 공유할 수 있는 범위만 묶고 원 PR별
   수용/보류를 분리한다. 구체적인 원격 처리안·댓글을 준비한 뒤 승인 요청을 한다. #536·#8701의
   전체 로드맵 완료 여부는 이번 변경과 구분하며 자동 close하지 않는다.

최초 조회·reviewer·head·CI 원장은 [접수 원장](assets/seo-rii-20261109/intake.json)을 따른다.
