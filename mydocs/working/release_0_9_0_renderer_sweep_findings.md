# 0.9.0 승격 검증의 Full Renderer Sweep 실패 조사

## 확인한 범위

2026-11-08 KST, 기준 devel `78615c52a7c7d7a6b25fad57c3bf1041fb2dfcf2`.
[수동 승격 실행](https://github.com/edwardkim/rhwp/actions/runs/42434345354)은
`BlogForm_BookReview.hwp`를 HTTP 404로 읽지 못해 실제 첫 비교 전에 실패했다.
Oracle의 assets 누락을 보정하는 #8766과 원인이 다르다.

## 1. 파일 전달 실패: 로컬 보정·검증 완료

`rhwp-studio/vite.config.ts`의 `/samples` middleware는 저장소 `samples/`에 파일이
없으면 바로 404를 반환한다. 같은 URL을 제공하는 Vite `public/samples/`에 있는 문서도
가려진다. 기존 #2185의 [실행 기록](archives/task_m100_2185_stage4.md)에도 같은 실패가
있었으며 당시에는 읽을 수 있는 여섯 표본만 검사했다. 이번 승격은 전체 표본을 요구한다.

- 원래 11개 표본 중 5개 HTTP 404를 실제 재현했다.
- 저장소 파일이 없고 public에 실제 regular file이 있을 때만 Vite의 public 처리로 넘기는
  보정으로 11개 표본의 응답과 원본 바이트가 모두 같았다.
- 저장소 샘플 우선순위, 없는 문서의 404, 상위 경로 접근 403도 실제 HTTP로 확인했다.
- 새 비시각 HTTP 회귀는 기존 `npm test` 자동 수집에 포함했다. 별도 CI 호출을 추가하지 않았다.
- Studio 2,778건 통과·기존 skip 2건, TypeScript와 production build 통과.
- 첫 전체 테스트에서는 오래된 로컬 WASM에 `insertMemo`가 없어 1건 실패했다.
  정확한 기준 devel의 [WASM Build](https://github.com/edwardkim/rhwp/actions/runs/42434338951)
  artifact `13599253771`을 적용한 뒤 전체 테스트가 통과했다. 실제 브라우저가 받은 WASM의
  SHA-256은 `385adc68f5a817ad8aa557f4572a7366f4ea04a7efde3bbe9af9fae1f6d8c378`로 일치했다.

원본 문서·기대값·workflow·비교 임계치는 변경하지 않았다. HTTP 보정은 로컬 상태다.

## 2. 입력 복구 뒤 드러난 Legacy/Layer 본문 clip 차이

같은 WASM으로 11개 표본의 첫 쪽을 실제 실행했다. 10개는 기존 기준을 통과했으나
BlogForm 1쪽은 `570 / 440,960 = 0.12926%`로 현행 `0.05%` 기준을 넘었다.
이 값은 **Legacy/Layer RGBA 차이율**이며 독립 한컴 PDF와의 실루엣 일치율이 아니다.

직접 이미지를 대조하면 차이는 오른쪽 표 테두리의 한 픽셀 열(x=506, y=196..765)에
있으며 max channel delta는 41이다. 원문 누락이나 페이지 수 차이로 판정하지 않았다.

실제 Canvas `rect` 호출을 추적한 결과:

| 경로 | 본문 clip x | 폭 |
| --- | ---: | ---: |
| Legacy | 30.000000000000004 | 480.7333333333334 |
| Layer | 30.000000000000004 | 476.7333333333334 |

- `src/renderer/web_canvas.rs`의 Legacy Body replay는 비표식 모드에도 저장 clip에
  4px를 더한다. 현재 분기의 커밋은 `448853ae787`이며 최초 코드까지 이전 이력을 추적했다.
- `src/paint/builder.rs`는 표식이 없으면 저장된 본문 clip을 유지한다. 이후 Layer replay가
  그 결과를 그대로 소비한다. 표시용 표식·텍스트 그림자 확장은 별도 계약으로 남아 있다.
- 진단 중 브라우저에서 Legacy의 해당 clip에 추가된 4px만 제거하면, 같은 문서·같은
  호출에서 차이가 **570 → 0픽셀**이 된다. 이는 원인 분리 실험이며 제품 보정 완료나
  독립 한컴 정답 검증으로 승격하지 않는다. 이 진단 시점에는 제품 Rust를 수정하지 않았다.

원 해결 #6590 → PR #6591의 본문 폭/표 우단 처리와 독립 출력 근거도 확인했다.
[기존 검토](../pr/archives/pr_6591_review.md)는 `pdf/BlogForm_BookReview-2020.pdf`를
사용했다. 이후 #7059가 초기의 근소 초과 표 축소 가정을 정정했으며,
`tests/cases/issue_6590_nearfit_tac_table_body_width.rs`와
`tests/cases/issue_7059_nearfit_shrink_stored_ladder.rs`가 현재 계약을 보호한다.
저장된 셀 줄 폭과 선언 표 폭에 따라 표는 본문 우단을 넘을 수 있다. 이 폭·위치 계약은 이번에
바꾸지 않는다. 독립 세 판본의 같은 내용과 실제 표 우단을 대조했다.

## 필요한 다음 처리

1. 저장 본문 clip·표 외곽선 점유와 비인쇄 표식 확장의 계약을 독립 출력과 대조한다.
2. 검증된 공통 경계를 Legacy와 Layer가 소비하도록 최소 보정하고, 상수 확장으로
   화면을 맞추지 않는다. 기본 화면/표식 표시의 반례를 함께 확인한다.
3. 실제 원 사례의 Native/fresh WASM 전쪽·확대 판독, 기존 전체 Canvas 비교와
   변경 범위의 필수 Rust·Studio 검증을 완료한다. 기대값·임계치 완화로 통과시키지 않는다.
4. 검증된 보정을 별도 devel PR로 통합한 뒤 최종 devel SHA를 고정해 승격 workflow를 실행한다.

#8766의 승인된 CI·self-review·병합은 계속한다. 위 추가 실패를 알고 있는 상태에서
비용이 큰 13개 승격 workflow를 먼저 반복하지 않는다. main 승격·태그·실제 게시도 미완료다.

## 추가 제품 보정 승인과 구현

작업지시자는 위 clipping 보정·필수 검증을 이번 릴리즈 전에 진행하도록 승인했다.
HTTP 보정·진단 기록은 로컬 `069ff65549`에 커밋했다. 원격 push·새 PR 승인은 아직 요청하지 않았다.

기준 devel의 같은 원문과 `pdf/BlogForm_BookReview-2020.pdf`를 96dpi, 2022 호환 세대
(2020 Print 대응)로 비교했다. 명시적 `/home/edward/mygithub/ttfs` 공급으로 Native/fresh CI
WASM 각각 1/1쪽 raw 100%, 쪽수 일치다. `venv/bin/python`, Chrome revision 1660786을
사용했다. PDF 이름만 보고 도구에 존재하지 않는 `--compat 2020`을 전달했던 첫 명령은
입력 검사에서 거부됐고 측정 결과로 쓰지 않았다.

공통 `body_clip_with_editor_marks`는 layout이 산출한 본문 잉크 경계를 유지하고 기존
비인쇄 편집 표식 gutter만 명시적으로 확장한다. LayerBuilder와 Legacy WebCanvas가 이를
소비한다. Layer의 후속 텍스트 그림자 확장·실제 ClipRect replay는 그대로 유지한다.
측정·쪽 나눔·표 기하·기본 Layer 출력은 변경하지 않는다.

원 해결 #6269의 `LineNode::ink_bbox()`·본문 clip 포함 계약도 확인했다. 선의 반 획을
본문 경계 생산자가 계상하는데 Legacy에서 다시 4px를 더하는 것을 제거한다. 48px는
기존 편집 표식 전용 계약을 공통 위치로 옮긴 것이며 문서 수치 맞춤용 확장이 아니다.
새 제품 후보의 최적화 WASM·전쪽 직접 비교·표식 대조·필수 회귀는 다음 실행에서 확정한다.


## 검증 범위와 기존 회귀

이번 제품 변경은 레이아웃의 본문 clip을 생성하는 측정·줄 구성·pagination을 바꾸지 않는다.
Legacy Canvas의 잉여 확장을 제거하고 Layer의 기존 비인쇄 표식 확장을 공통화한다.
레이아웃 원점·높이·쪽수는 별도 보정하지 않으며, 기본 Layer/Native의 동일 출력은
독립 PDF와 변경 전후 대조로 확인한다. 공통 paint 경계이므로 Native 공식 3종·전체 회귀·
Rust lint·fresh WASM·실제 브라우저 검증을 실행한다.

재발 검사는 기존 자동화의 실제 실패 경로를 사용한다. `canvas-render-diff.test.mjs`의
원래 11개 표본 중 BlogForm 1쪽이 보정 전 실패했고, 같은 문서·허용치로 보정 후 확인한다.
`tests/cases/`의 #6590→#7059 원 사례와 #6269 선 잉크 포함·raised decoration 검사를
그대로 실행한다. helper 구현을 그대로 재인용하는 별도 단위 assertion을 늘리지 않는다.
독립 PDF raw 100% 조건의 기존 BookReview 우단 ±1px 회귀는 유효한 독립 계약으로 보존한다.

Oracle #8766은 CI·실제 Oracle 검증·self-review 후 `08914c0ed00119db2f26b4fa7ea8eb0495b76cfd`로
병합됐다. 옵션 2 기록 #8768도 `80fc0c6b44bb270e5c14946cfb52e0eb0475a3a8`로 병합하고
로컬 devel을 동기화했다. 이 clipping 후보와 최종 승격 검증은 별도로 진행 중이다.


### 실제 값의 생산과 최종 소비

- `src/renderer/layout.rs`의 `expand_clip`은 본문 자식의 잉크·흐름/부동 점유를 반영한다.
  흐름·부동 clip 합집합을 `Body { clip_rect }`로 저장하며 이번에 변경하지 않았다.
- `render_tree.rs::body_clip_with_editor_marks`는 이 값과 편집 표식 여부를 받아 기존 표식
  gutter만 적용한다. 측정·pagination으로 다시 전달하거나 표 원점을 덮지 않는다.
- `paint/builder.rs`의 Body 분기는 이를 소비한 뒤 기존 텍스트 그림자 잉크 합집합을 반영해
  `ClipKind::Body`를 만든다. `web_canvas.rs`의 Layer replay는 해당 clip을 그대로 사용하며,
  Print에서는 기존 device clip 좌표 변환을 유지한다.
- Legacy Body 분기도 같은 결과로 실제 Canvas `rect`·`clip`을 호출한다. Body 종료 후의
  `render_overflow_controls`는 기존 원본 clip과 좌우 넘침 계약을 그대로 사용한다.
  후속 임의 앵커·높이 보정이나 이 4px의 재추가는 없다.

### 완료한 새 WASM 검증

Docker Rust 1.93.1의 release 빌드·wasm-opt가 17분 20초에 성공했다. source는
`8118ccb14d092f767cd1caf1aaece4753a8e61d8`, WASM SHA-256은
`23cdca6248a2532499bbf09c2a92d0268403643e48b0f28d7d8e8885b8f7f3f5`다.
pkg/public JS·WASM이 같고 실제 초기화 응답도 같은 해시였다.

- 원 11개 Canvas 표본 모두 기존 0.05% 기준을 통과했다. BlogForm은 570→0픽셀·max delta 0,
  KTX는 0.01077%, 나머지 9개는 0%다.
- 문단 부호·조판 부호의 네 조합과 다시 숨김까지 5조건을 확인했다. 두 경로의 실제 clip이
  모두 같고 기존 표식 gutter 48px를 보존한다. 표식 표시 시 glyph paint의 19/25픽셀 차이는
  별도로 기록했으며 clip 일치와 전체 픽셀 일치를 혼동하지 않는다.
- 최초 진단 assertion의 `48 == 47.99999999999994` 실패는 부동소수점 비교의 문제였다.
  진단에서 1e-9 오차만 허용했고 제품·시각 허용치는 변경하지 않았다.
- 독립 Print와 fresh WASM Canvas2D 전 1쪽은 raw 100%, 쪽수 일치다. 전쪽 직접 판독에서
  오른쪽 선·하단선·행 분할·문자와 Print의 비인쇄 안내 제외를 확인했다.
- 이전 Layer 화면 PNG와 이전 WASM Print PNG는 새 출력과 각각 바이트 단위로 같다.
- 새 WASM을 소비하는 Studio 2,778건 통과·기존 skip 2건, TypeScript·production build 통과.

Native 첫 빌드에서는 최소 글꼴 bwrap에 WSL의 DNS symlink 대상이 없어 Skia 다운로드가
실패했다. `/mnt/wsl/resolv.conf`만 읽기 전용으로 연결했고, 개인/Windows 글꼴은 계속 숨겼다.
저장소 라이선스 확인 글꼴 5개만 노출됨을 다시 확인한 뒤 빌드·회귀를 진행한다.


### Native 시각 검증 완료

동일 source의 `native-skia`·release-test 실행 파일을 별도 경로로 고정했다. Native 독립
Print 전 1쪽도 raw 100%이며, 실제 Native screen/print PNG 각각이 기준 devel의 출력과
바이트 단위로 같다. Native/fresh WASM·독립 PDF의 표 우단·행 경계·하단 모서리를 4배로
직접 대조했다. 전체 픽셀·글꼴 래스터가 PDF와 완전히 같다는 주장과는 구분한다.

[전쪽 TSV·직접 판독·빌드 및 입력 신원](../tech/investigations/release-0.9.0-renderer-sweep/README.md).
표식의 정상 대조군 `lseg-05-tab.hwp`도 Canvas2D·CanvasKit compatibility/default의
screen/print 6개 실제 capture를 완료했고 report-only parity 4개 모두 통과했다. 이는 해당
표식 경로의 보존 확인이며 전체 21개 representative 문서의 완료를 주장하지 않는다.
전체 matrix의 탐색 실행은 이 소비 경계보다 넓어 중단했으며, 최종 승격 workflow 범위에는
그대로 남겨 뒀다.

편람 383쪽 gate의 조건인 조판 변경은 비해당이다. 이 diff는 layout·줄 구성·메트릭·분할을
변경하지 않으며, 기존 Layer/Native의 화면·Print 실제 PNG도 동일하다. 실제 변경된
Legacy clip 소비와 공통 paint 경계를 위 원문·정상 대조군 및 필수 전체 회귀로 검증한다.


## 필수 로컬 검증 완료와 제출 대기

2026-11-08 KST, 제품 source `8118ccb14d092f767cd1caf1aaece4753a8e61d8`에서 완료했다.

- fmt·root/WASM32/workspace all-target Clippy·workspace build·suite manifest 검사 모두 통과.
- 집중 회귀 9건, 전체 nextest 12,997건 통과. 기존 skip 45건은 유지했다.
- Native 공식 3종은 라이브러리 4,040건·누락 그림 2건·직접 PDF 35건, 합계 4,077건 통과.
  라이브러리의 기존 ignored 12건은 유지했다.
- Studio 2,778건·TypeScript·production build, 최적화 WASM·실제 Canvas 11개 비교 통과.
- 독립 Print의 원 사례 전 1쪽은 Native/fresh WASM 각각 raw 100%, 쪽수 일치와 확대 판독 완료.
- 최소 글꼴 환경과 실제 명령·로그 해시·실행 요약을 정식 증거에 보존했다. 테스트 기준을
  바꾸거나 실패 검사를 제외하지 않았다. 대형 표 장기 실행 검사도 정상 종료했다.
- 검증에 생성된 public JS는 원래 추적본으로 복원했고, 공유 양식의 생성 JS는 `output/`으로
  옮겼다. 해당 WASM 소비 증거는 검증 당시의 실제 초기화 응답이다.

[최종 검증 원장](../tech/investigations/release-0.9.0-renderer-sweep/README.md).
최신 devel `80fc0c6b44bb270e5c14946cfb52e0eb0475a3a8`과의 실제 병합은 충돌이 없고
Oracle 후속 운영 기록 4개만 추가된다. 제품 소스는 검증 후보와 같다.
원격 push·새 PR·CI·self-review·merge는 준비된 후보에 대한 추가 원격 승인 뒤 진행한다.
그 뒤 이미 승인된 최종 devel의 13개 승격 workflow로 돌아간다.


## PR #8771 source CI·self-review 완료

사용자가 후보의 push·PR·CI·self-review·머지를 승인했고 #8771을 Open으로 제출했다.
정확한 source head `339f77f6f27fd15bae4db0b4201af3bd5e408eef`의 제품 CI와 최종 정책
감사가 성공했다. GitHub 단계 metadata 지연은 별도 승인된 최종 감사 두 번으로 해소했으며
제품 CI는 반복하지 않았다. [검토 기록](../pr/archives/pr_8771_review.md)에 실제 결과를
남겼다. 문서-only trailing head의 실제 검사와 머지 뒤 최종 승격·게시로 이어간다.
