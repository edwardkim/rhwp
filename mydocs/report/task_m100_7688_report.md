---
kind: report
status: completed
last_verified: 2026-10-09
---

# #7688 출력 백엔드 정합성 보정과 PR 제출 준비

Issue: [#7688](https://github.com/edwardkim/rhwp/issues/7688).
최신 확인 base는 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`다.
검증한 제품·회귀 코드 기록 commit은 `33991c311a81aef43392f9f9940f5ab805972d8d`이며,
후속 시각 통과 기록은 `833a5c724`다. 제출 준비에서 제품 Rust/TS와 회귀 본문은 변경하지 않았다.
E2E 파일명은 신규 명명 규칙에 맞춰 변경하고 MANIFEST/npm 실행 명령에 등록했다.
이 보고서는 로컬 구현·검증·시각 판정과 제출 준비의 완료이며 원격 통합 완료를 뜻하지 않는다.

## 해결한 계약과 실제 소비

| 계약 | 독립 근거·실제 소비와 반례 | 수정 전후 증거 |
| --- | --- | --- |
| Native 수식 글꼴 | 같은 원문의 정상 한컴 Print, custom CJK/Latin-only face의 독립 glyph coverage. 본문 font 후보를 별도 수식 paint까지 전달하고 선택 run의 폭을 그리기·정렬이 공유한다. 저장 줄 구성은 변경하지 않는다. | [3단계](../working/task_m100_7688_stage3.md): 실제 수식의 한글 네모와 공급 무시를 검출; custom·fallback·screen/print FAIL→PASS |
| 그림 crop | pic-crop 원문의 첫 그림 70행/둘째 약 58행. 원본 전체 기준이 없는 zero-origin 끝점을 무-crop으로 해석하지 않고 등방 단위를 보존한다. Rust source rect와 Studio DOM/CanvasKit 소비를 함께 대조한다. | [4단계](../working/task_m100_7688_stage4.md): banner 70행→약 58행; 200dpi scan·부분 로고 정상 대조군. 명시 reference 경로 유지 |
| 화살촉 | 한컴 Print의 크기 9종·두께·짧은 선 관측. 공통 arrow path를 SVG/WebCanvas/Skia/paint JSON→CanvasKit이 소비하며 원래 tip 소유와 방향을 유지한다. | [5단계](../working/task_m100_7688_stage5.md): SVG intrinsic 크기 및 실제 Native PNG 누락 FAIL→PASS; [7단계](../working/task_m100_7688_stage7.md): #7333 19·31·41·43쪽 대조 |
| CanvasKit 준비·style | 실제 준비된 문서 font를 explicit CanvasKit에 공급한다. SFNT Regular/Bold 단독 공급 출력이 독립 대조군이며 pair/reversed 순서의 실제 renderPage 결과가 각각 일치한다. | [6단계](../working/task_m100_7688_stage6.md): 준비 누락·잘못된 Regular 선택 FAIL→PASS; 자원 실패를 기록하며 같은 페이지를 Canvas2D로 표시 |
| 재발 방지 절차 | 공통 producer 이후 각 backend의 별도 font/resource·메트릭·합성 소비까지 검증한다. SVG raster와 실제 Canvas/Skia 출력의 증거를 구분한다. | [검증 가이드](../manual/verification/renderer_backend_verification.md), CONTRIBUTING·거버넌스·review/local validation에 연결 |

조판 원칙 검토 결과: 위 네 소비 계약의 근거·일반성·측정/paint 연결·독립 대조군은 해당 단계의
실행 증거로 충족했다. 쪽 분할·이어받기 및 저장 LineSeg 수용 조건은 변경하지 않아 비해당이다.
파일 ID 분기·출력 은폐·새 clamp·pixel golden·허용치 완화는 추가하지 않았다.
기존 crop 2건과 #7333 3건의 잘못된 검사 조건은 독립 PDF 관측으로 교정했다.
실행한 최초 실패와 준비 오류를 최종 통과와 구별한 근거는 7단계에 있다.

## 시각 검증과 보존 입력

2026-10-09 메인테이너가 최종 비교 HTML의 crop·화살촉·수식과 #7333 영향쪽에
**시각 검증 통과**를 판정했다. [대표 review/overlay 및 실제 출력](../pr/assets/issue_7688_renderer_backends/README.md)을
제출 브랜치의 안정 경로에 보존했다. PNG는 판정 자료와 byte 단위로 동일하다.

| 범위 | Native/fresh WASM 전체 최저 | 실제 출력 확인 |
| --- | --- | --- |
| 대표 6문서 전체 8쪽 | 두 경로 95.89322%, full review gate 모두 passed | Native PNG screen/print 8쪽 각각 96.18681%; Studio 첫쪽 6개 Canvas2D 95.53281% / CanvasKit software 94.58234% |
| crop scan 대조군 전체 1쪽 | 두 경로 93.49689% | 선행 실제 Native PNG 92.22154%; 이전 대조군 보존 증거는 4단계 |
| 화살촉 크기·두께·짧은 선·복합선 5개 전체 5쪽 | 두 경로 99.84531% | 한컴 Print 관측과 Native/fresh WASM 직접 비교는 5단계 |
| #7333 전체 50쪽 | 두 경로 95.41718% | 영향 19·31·41·43쪽 실제 Native PNG 최저 99.77949%, Studio Canvas2D 99.52841% / CanvasKit software 99.07962% |

총 13개 원문·64쪽에서 Native/fresh WASM 각각 누락·측정 불가·90% 미만은 0쪽이다.
대표 6문서와 #7333은 기존 최종 검증을 재사용하고 scan·화살촉 5개 대조군은 제출용 안정 경로의
동일 입력으로 최종 Native 바이너리와 기존 Docker WASM을 다시 실행했다.
수치 게이트는 2px 이웃 관용 내용 실루엣이며 엄격 pixel parity와 다르다.
대표 full review의 pixel_match/visual_accuracy_proxy 원값은 해당 overlay_metrics에 보존했다.
낮은 proxy나 높은 실루엣을 전체 fidelity의 단독 판정으로 바꾸지 않았다.

[입력/PDF 목록과 SHA-256](../../samples/issue7688/README.md)에 원문 대응을 고정했다.
대표 6문서·scan·화살촉 대조군의 실제 사용 PDF 12개와 대조 HWPX 5개를 byte 동일하게 복사했다.
기존 원문 및 #7333 PDF는 추적 경로를 재사용하며 기존 기준 PDF를 덮어쓰지 않았다.
MCP 기준은 Hancom 2020 11.0.0.9136, print method 0, one-up이다.
원문 저장 제품과 Print 출처·job/해시는 기존 1·4·5단계 증적에 연결한다.

## 실행 신원·명령·검증

공용 target은 `/home/edward/mygithub/rhwp/target/pr-review`다.
Native 바이너리 SHA-256은 `c5ad2a3d672bcf500b86252bafda95894bfb8cd5c2539e89be61fd774def09ea`다.
Docker WASM은 `cc5e9c9d86d2d3ac117a9184510bca661fb8cfaa6aef660707de2c9063575de2`,
JS는 `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`다.
pkg/public 및 새 탭 초기화 응답·TS source map의 동일성은 7단계 provenance에 연결했다.
Chrome 154.0.8037.57, Print sweep 96dpi, Studio zoom/DPR 1 및 CanvasKit software다.
SVG sweep의 font supply는 local/default이며 실제 Studio font URL은 각 캡처 provenance에 기록했다.

| 필수 로컬 검증 | 완료 결과 |
| --- | --- |
| fmt, Native/WASM32/workspace all-target Clippy, workspace build | 모두 PASS; 추가 Native Skia Clippy PASS |
| release-test 전체 nextest | 10,553 PASS / 0 FAIL / 50 skipped |
| Native Skia/workspace lib, missing picture, 직접 PDF | lib 4,109 PASS / 13 ignored; 별도 2 PASS / 4 PASS |
| source unit-tier/suite manifest 고정 base 정책 | PASS; generated suite/manifest는 제출 제외 |
| Studio noEmit·전체 단위 검사 | PASS; 1,825 PASS / 0 FAIL / 2 skipped |
| 제출 보완 후 Studio build | `npm --prefix rhwp-studio run build` PASS |
| 제출 보완 후 E2E 등록·실제 실행 | `python3 scripts/check_e2e_manifest.py` PASS (151/151); `npm --prefix rhwp-studio run e2e:issue-7688-fonts` PASS |
| 제출 문서 검사 | 19개 문서 상대 링크 및 diff 공백 검사 PASS; 변경 canonical 메타데이터 오류 0 |

전체 Rust argv·exit code는 ignored `output/pr-review/renderer-backend-audit-20261009/final/checks-results.json`에,
각 결과 로그와 실행 시간은 [7단계 기록](../working/task_m100_7688_stage7.md)에 있다.
제출 보완 E2E는 `VITE_URL=http://localhost:7788`, 확인한 `CHROME_PATH`를 지정했다.
제품/검사 bytes와 base가 동일해 대형 회귀와 동일 WASM은 반복하지 않았다.
추가 sweep argv는 `final/submission-extra-{native,wasm}-command.json`에 보존하며
`VISUAL_SWEEP_CHROME`에 같은 Chrome을 지정했다. 처음 Chrome 환경변수 누락으로 실패한 로그는
별도 environment-failure로 보존하고 제품 결함이나 통과로 세지 않았다.
전체 장기 문서 메타데이터의 기존 오류 16건은 해당 파일이 base와 byte 동일함을 확인했으며,
이번 변경의 신규 오류는 0건이다. 다른 작업의 문서는 수정하지 않았다.

## 잔여 범위와 제출 상태

엄격 backend parity는 report-only 24건 중 16 PASS / 8 FAIL이며 탭/수식 각 4건이다.
각 차이의 크기·전후는 7단계에 보존했고 baseline 허용치를 바꾸지 않았다.
auto/GPU readiness evaluated=0, Windows/macOS·실제 GPU·모든 italic face·모든 화살촉 모양과
curve/arc·원본 크기가 완전히 불명인 모든 crop은 미검증이다. Studio 표 2·3쪽은 첫쪽 캡처에 포함되지 않는다.
이번 해결 범위를 이 전체 조합의 완료로 확대하지 않으며 PR은 `Refs #7688`로 연결한다.

PR 본문 초안·원격 명령과 전체 Native TSV ZIP은 ignored `final/`에 준비한다.
TSV·실행 로그·중간 JSON은 Git에 커밋하지 않는다. ZIP은 PR 게시 때 첨부할 자료다.
원격 push·PR 생성 후에는 채번된 archive review를 추가하고 최신 head CI와 raw 이미지의
실제 표시를 다시 확인한다. 원격 게시·merge·이슈 종료는 아직 수행하지 않았다.
