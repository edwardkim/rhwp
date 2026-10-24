# CanvasKit 초기화와 공통 인쇄 종이 계약 보존

Issue: #7445. 새 HWP 보정 대상을 추가하지 않았다.

부모 `c33e8435365395ac72f04400fbb878543b34ba4d`에서 upstream의 상대 경로 import와 이 브랜치의 별칭 import가 같은 `parseCssFontFamilyList`를 중복 선언했다. upstream import를 유지하고 별칭 import 한 줄만 제거했다. 실제 모듈 HTTP 500과 Canvas2D fallback 8건을 보정한 증거다.

## 실제 제품 경로

정상 문서 열기 → 문서 초기화 → CanvasView 쪽 이동 → 실제 pooled canvas와 sibling plane을 포함한 브라우저 사각형 캡처다. source bytes, 실제 backend·mode·profile, 원본 JS/WASM 초기화 응답, 준비 상태와 direct replay를 확인했다. 로컬 글꼴은 제품의 감지·저장 기능으로 준비했다. 임의의 글꼴·그림을 주입하지 않았다.

기존 지원 대조군 두 개 × compat/default × screen/print **8건 모두 요청 backend로 캡처됐고 프로세스 exit 0**이다. 숨은 Canvas2D overlay는 허용하지 않았다. 이 두 대조군의 성공을 개인정보 preflight 한도나 고용노동부의 글꼴 준비 미달에 대한 CanvasKit 통과로 일반화하지 않는다.

| 대조군 | compat Print raw | default Print raw | 판독 |
| --- | ---: | ---: | --- |
| low-slack | 99.97179% | 99.97179% | 제목·별표 행·서명·붙임·주소 보존 |
| 저장 줄 결측 | 99.83083% | 99.83083% | 표 원점·행·아래 경계와 본문 보존 |

각 대조군의 두 Print PNG는 byte 동일하다. Screen은 서로 다른 고유 PNG 세 개를 직접 판독했다. Screen과 Print PDF의 수치 비교는 진단이며 동일 profile 통과로 쓰지 않는다. 글자 농도·얇은 괘선·편집 캐럿의 작은 차이는 남는다.

- [low-slack review](low-slack-p001-actual-print-review.png) · [overlay](low-slack-p001-actual-print-overlay.png)
- [저장 줄 결측 review](missing-lineseg1-p001-actual-print-review.png) · [overlay](missing-lineseg1-p001-actual-print-overlay.png)
- [출처와 8건 runtime 신원](producer.json)
- [low-slack 원본](../../../../../../../samples/issue6535/36339092_low_slack_absorb_block.hwpx)
- [저장 줄 결측 원본](../../../../../../../samples/issue7344/36455985_no_lineseg_cell_spacing_before.hwpx)

## 검증과 한계

TypeScript 재검증 exit 0, Studio 전체 2,408건 중 **2,406 PASS / 0 FAIL / 2 skip**이다. 오래된 소스 정규식은 편집 종이의 크기만 강제해 공통 Print 종이 계약과 충돌했다. 실제 제품 메서드로 같은 크기·더 큰 종이·더 작은 종이의 소수 경계를 실행하고, 원본 편집 좌표가 renderer에 전달되는지도 검사하도록 기존 함수를 바꿨다. 비교 창의 경계 검사도 유지했다. 이전 크기 소비를 되살린 진단은 예상 크기 451×601 대신 151×301을 만들어 실제로 실패했다. 이 진단을 원본 빌드의 수정 전 실행으로 부르지 않는다.

Rust/WASM 엔진과 폰트·샘플은 이 단계에서 변경하지 않았다. 부모의 전체 nextest **11,751 PASS / 0 FAIL / 45 skip** 및 범위 게이트와 source가 같다. 이 증거는 브랜치 전체 시각·실제 소비 검증을 대신하지 않는다. 전쪽 base/head 비교와 남은 실제 출력 검토가 진행 중이므로 **PR 준비 완료가 아니다**.

실행 로그·실패·진단·전체 browser 관찰은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/canvaskit-import-repair/`에 보존했다. Chrome 종료 후 GoogleUpdater가 물려받은 stdout/stderr 때문에 Node 정리만 대기한 건은 실제 프로세스와 pipe 소유를 확인하고, 종료된 Chrome의 해당 read pipe만 닫아 exit 0을 받았다. updater·다른 프로필·공유 target을 종료하거나 삭제하지 않았다.
