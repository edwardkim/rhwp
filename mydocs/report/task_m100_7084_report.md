# #7084 최종 보고서 — 이모티콘 진행폭과 Canvas 그리기 일치

- Issue: [#7084](https://github.com/edwardkim/rhwp/issues/7084)
- 작성: 2026-09-14, 같은 날 메인테이너 최종 결과 승인.
- 계획: [수행계획](../plans/task_m100_7084.md), [구현계획](../plans/task_m100_7084_impl.md).
- 통합 전 제품/테스트: `230f801136a4fc73725b847fe56c49b2c11c88bd`.
- 최신 devel 통합 검증 후보: **`d3a189d5d964e4e6eb3b376062df846744c4c58d`** (base `11860a9f4`).
- 선행 검증 기록: `925e2cf9d`. 이번 보고서 통합은 제품 변경이 아니다.
- 상태: 최종 보고서 승인, 최신 devel 통합·통합본 로컬 검증 완료. 원격 제출·CI·self-review·병합·이슈 종료는 남았다.

## 1. 결과 요약

**문서가 요구하지 않은 이모티콘의 가로 압축을 제거했다.** 기존 구현은 지정 글꼴에
진행폭 정보가 없으면 좁은 추정 폭으로 자리를 배정했지만, 브라우저는 더 넓은 대체 글꼴로
그렸다. 두 폭을 맞추는 마지막 단계에서 높이는 유지하고 가로만 약 36.4%로 압축했다.

수정 구현은 기존 정보로 폭을 결정할 수 없는 지원 대상에 한해, 실제 Canvas2D 설정으로
측정한 자료를 조판·문자 위치·그리기가 함께 사용한다. 이모티콘의 크기를 정사각형으로
강제하거나 특정 샘플·문자에 폭 상수를 넣지 않았다. 문서의 장평·자간은 유지한다.

메인테이너가 원본 HWP/HWPX의 rhwp-studio 이모티콘 렌더링을 모두 시각 통과로 판정했다.
이후 결합열 경계와 오류 복구를 보완하고 원본 동작을 재확인했다. 모든 운영체제의 글꼴 모양,
모든 이모지 결합열, 다른 출력 엔진까지 같아졌다는 판정은 아니다.

## 2. 입력과 원인 계보

대상 파일은 다음 두 개이며, 각각 2쪽이다.

- `samples/issue3587/c-form-labnote-001-stage11-filled.hwp`
- `samples/issue3587/c-form-labnote-001-stage11-filled.hwpx`

대상은 2쪽 `실물😀\n본문`의 `😀`(U+1F600)이다. 내부 주소는 0 기반 section 0 →
paragraph 12 → control 1 → cell 5 → cell paragraph 0이다.

| 관측 | 수정 전 | 수정 후, 동일 Windows Chrome Canvas2D |
| --- | --- | --- |
| 문서 지정 글꼴 | 함초롬바탕 | 보존 |
| 폭 결정 | DB에 문자 없음 → `heuristicHalfwidth` 500 HWPUNIT | `supplementaryBackendMeasured` 약 1,372 HWPUNIT |
| 브라우저 자연 진행폭 | 글자 크기 13.333px에서 약 18.302704px | 동일 설정으로 측정·배치·그리기 연결 |
| 이모지 paint 가로/세로 배율 | 약 0.364245 / 1 | 1 / 1 |

수치는 이 환경의 관측값이지 제품에 넣은 상수가 아니다. 배율 1/1은 인위적인 가로 압축이
없다는 뜻이며, 실제 그림의 너비와 높이가 반드시 같다는 뜻은 아니다.

[Stage 1](../working/task_m100_7084_stage1.md)의 계보 조사에서 폰트 메트릭 생성기의
7개 문자 수집 범위는 초기 `f0f7f1a4b4`와 `v0.8.6`에도 있었고, 생성 DB 파일 분리는
`8f516d8413`이었다. U+1F600의 **DB 누락 자체는 기존 제한이며 #3587의 신규 회귀가 아니다.**
최초로 화면 압축이 발생한 버전까지 특정한 것은 아니다.

실행 추적은 `text_measurement.rs`의 미등록 문자 대체 폭 → 문자 위치 →
`web_canvas.rs::draw_text`의 배정 폭/실제 폭 비교 → `canvas_cluster_fit_scale` →
가로 변환으로 이어졌다. 두 포맷의 문자·서식은 보존되어 있었고, UTF-16 문자 소실이나
사용자가 36% 장평을 지정한 현상은 아니었다.

동일 입력의 한컴 PDF는 Segoe UI Emoji subset의 약 1.373em 진행폭을 사용했다.
MCP engine/profile 선택은 2020, 실제 응답 Hancom 버전은 12.0.0.4605다. PDF 파일명의 2020을
실제 제품 11.x를 확인했다는 뜻으로 해석하지 않는다.
브라우저의 실제 대체 face 이름은 확정하지 않았으며, 한컴의 face와 동일하다고 추정하지 않는다.
외부 엔진 조사도 공개 구현/API가 보여 주는 선택·측정·그리기 일치 원칙의 근거로 사용했을 뿐,
Word나 Pages의 비공개 내부 구현을 확인했다고 주장하지 않는다.

## 3. 구현과 보호 불변식

| 책임 | 구현 경계 |
| --- | --- |
| 보충 측정 계약 | `src/renderer/supplemental_metrics.rs`: 출처·문맥·유효성·상한 관리 |
| 폭 소비 | `src/renderer/layout/text_measurement.rs`: 기존 폭 결정과 보충 측정 연결 |
| 결합열 보호 | `src/renderer/composer/supplemental_clusters.rs`: 논리 문단 경계를 서식 run에 전달 |
| 측정·그리기 일치 | `src/renderer/web_canvas.rs` 및 Studio 측정 준비부: 같은 설정·단위 소비 |
| 세션 수명 | Studio `canvas-metric-session.ts` 및 bridge: 문서/폰트 세대와 출력 문맥 관리 |
| 제한된 자동 복구 | Studio `canvas-metric-recovery.ts` 및 CanvasView: 유효한 동일 세대에서 최대 1회 |

핵심 보호 규칙은 다음과 같다.

1. 기존 DB 적중, 공백·탭, 사적 사용 영역·옛한글 등 별도 폭 규칙을 보충 측정으로 덮지 않는다.
   일반 추정 반각 폭 경로의 지원 대상만 처리한다.
2. 측정 snapshot은 렌더링 세션이 소유한다. 문서 IR·원본 서식·저장 데이터·편집 취소 이력에
   브라우저 의존 측정 자료를 넣지 않는다. 장평·첨자 환산과 자간을 중복 적용하지 않는다.
3. 실제 자원까지 검증한 `verifiedSource`와, 실제 face는 미확정이지만 동일 backend에서
   측정한 `backendMeasured`를 구별한다. 이번 Studio의 결과는 후자다.
4. VS16/ZWJ 등 미지원 결합열은 서식 run이 갈라져도 전체를 기존 경로에 남긴다.
   다른 위치의 단독 문자로 cache가 채워져 있어도 결합열 일부에만 적용하지 않는다.
5. 보충 cache는 고유 항목 4,096개·문자열 key의 UTF-8 payload 4MiB 상한을 갖는다.
   비정상 수치·문맥 불일치·오래된 응답을 거부한다. 정상 paint에서 문자마다 폰트를 다시 읽지 않는다.
6. Canvas 전용 폭을 native/CanvasKit/SVG/print에 유출하지 않는다. portable 출력 때 문맥을
   전환하고 종료 뒤 기존 화면 문맥을 복원한다.
7. descriptor 불일치는 그리기를 거부한 뒤 문서·폰트 세대·view 유효성을 재확인하여 한 번만
   무효화·재준비·재그리기한다. 복구 자체가 폰트 세대를 올려 재시도 예산을 충전하지 않는다.
   지속 실패는 숨기지 않고 진단을 유지한다. 일반 렌더링 오류는 이 복구의 대상이 아니다.

중간 검증에서 두 가지 계약 누락을 발견하고 같은 이슈 안에서 보완했다.
[Stage 5](../working/task_m100_7084_stage5.md)는 서식 경계에서 결합열 일부가 적용되는 문제를
고쳤다. 중간 후보에서 기존 shaping run 보존 검사 6건이 실패하여, 무관한 run과 비활성 경로를
보존하도록 범위를 정정했다. [Stage 6](../working/task_m100_7084_stage6.md)은 불일치 거부 이후
화면 자동 재준비가 빠진 경로를 연결했다. 검사 기대값·golden·허용치를 완화해 통과시키지 않았다.

## 4. 검증 결과와 소스 대응

아래 첫 표는 **devel 통합 전 증적**이다. Rust 검증 제품은 `281fb2826`, Studio는 `230f80113`이다.
두 커밋 사이 Rust/Cargo/build 입력 차이는 없으며, 새 Docker WASM도 byte-identical하다.
**Rust 전체 검사를 `230f80113`에서 다시 실행했다고 보고하지 않는다.** 동일 입력의 Stage 5
증적과 변경된 Studio의 Stage 6 재검증을 구분한다.

| 검사 | 결과 | 근거 |
| --- | --- | --- |
| Rust release lib | 4,055 PASS / 13 ignored | Stage 5 |
| 전체 integration | 9,773 PASS / 51 skipped / 0 FAIL | `output/7084/stage5/04r-nextest.log` |
| 집중 메트릭·세션·기존 shaping 계약 | 55 PASS, integration의 부분집합 | Stage 5 |
| Native Skia | lib 4,112 PASS / 13 ignored, 집중 2+4 PASS | Stage 5 |
| fmt·세 Clippy·workspace build | 모두 PASS, `-D warnings` | Stage 5 |
| doctest·파생 suite/단위 test 정책 | 8 PASS / 3 ignored, 정책 검사 PASS | Stage 5 |
| Docker WASM | PASS, 약 2분 56초 | `output/7084/stage6/docker-wasm.log` |
| Studio TypeScript·production build | PASS | Stage 6 |
| Studio 전체 npm test | 1,687 PASS / 2 skipped / 0 FAIL | `output/7084/stage6/npm-test.log` |
| 복구·세션 집중 검사 | 15 PASS, 전체 npm의 부분집합 | Stage 6 |
| 실제 Chrome 오류 복구 | HWP/HWPX × 6조건 = 12/12 PASS | `output/7084/stage6/browser-recovery.json` |
| 원본·서식·출력 문맥 복원 | 두 원본 및 장평 64%·첨자·120% 굵게/기울임 PASS | Stage 6 `connected/runtime.json` |
| VS16/ZWJ 서식 경계 | 두 형식 × 2종, 부분 적용 0·오류 0 | Stage 6 `run-boundary.json` |
| 기본 Render Diff | 3/3 PASS; KTX 0.01593%, 나머지 0% | Stage 6 `render-diff.log` |
| 직접 PDF gate | 3/3 PASS; 1.158954% / 0.390370% / 0.676720%, 각 2% 이하 | Stage 5 |
| E2E 등록 검사 | 132 tracked / 132 등록, PASS | Stage 6 |

통합 전 WASM: **11,080,648 bytes**, SHA-256
`9a7500e19b1bf9e195eb7331697aed5edfc65214b77cda38f0ef0891ba58ca77`.
Chrome `152.0.7977.83`이 HTTP로 받은 bytes/hash와 디스크 산출물이 일치했다.
이번 보고서 작성에서도 디스크 hash와 integration/npm 로그의 최종 집계를 재확인했다.

오류 복구는 일시/지속 불일치, 준비 중 view 교체·실제 문서 재로드·외부 폰트 변경, 일반 오류를
통제하여 검사했다. 일시 오류는 대상 2쪽을 복구했고, 지속 오류는 1회 시도 후 그리기 거부를
유지했다. 후자를 성공한 화면 복구로 집계하지 않는다. 복구 연결만 메모리에서 끈 음성 대조는
예상한 종료 코드 1·22개 실패 assertion을 냈고, 연결 복원 후 12개 조건을 다시 통과했다.
이는 자연 발생한 모든 폰트 장애를 검증했다는 뜻이 아니다.

재현 명령 전체는 Stage 5·6의 게이트 표를 따른다. 추적된 자동 복구 검사는
`CHROME_CDP=http://localhost:19222 npm --prefix rhwp-studio run e2e:canvas-metric-recovery`다.
`output/`의 실행 진단·이미지·계측 스크립트는 로컬 증적이며 GitHub에서 다운로드 가능한 첨부가 아니다.

### 4.1 최신 devel 통합 후 재검증

승인 후 devel `11860a9f4`를 일반 merge했다. 충돌 3곳은 두 WASM 모듈과 양쪽 오늘할일 기록을
모두 보존해 해결했다. 자동 병합된 하이퍼링크·TAC·PDF 경로도 같은 후보에서 검사했다.
후보 **`d3a189d5d`**의 결과이며, 명령·로그·입력 hash·경고는
[Stage 7](../working/task_m100_7084_stage7.md#6-통합-후보-전체-검증-완료)에 기록했다.

| 검사 | 통합 후 결과 |
| --- | --- |
| Rust 전체 nextest `--tests` | **9,814 PASS / 51 skipped / 0 FAIL**; lib·integration 포함 |
| 별도 lib / 집중 | 4,055 PASS / 13 ignored; 84 PASS (전체 수에 중복 합산하지 않음) |
| Native Skia 3종 | lib 4,112 PASS / 13 ignored, 그림 2 PASS, PDF 4 PASS |
| fmt·세 Clippy·workspace build·doctest·검증 정책 | 모두 PASS, doctest 8 PASS / 3 ignored |
| 폰트 projection 생성기 | 14 PASS |
| Docker WASM·TypeScript·production build | 모두 PASS |
| Studio npm | **1,692 PASS / 2 skipped / 0 FAIL** |
| 실제 Chrome 원본·서식·portable 복원·결합열 보호 | HWP/HWPX PASS |
| 오류 복구 / renderer·Undo 계약 | 12/12 조건 PASS / PASS, Undo 24 assertion |
| 기본 Render Diff / 직접 PDF gate | **3/3 PASS / 3/3 PASS**, 기존 허용치 유지 |
| E2E 등록 / PDF 저장 정책 | 134/134, PASS / PASS |

통합 WASM은 **11,182,083 bytes**, SHA-256
`61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`이며 Chrome이 받은
HTTP bytes와 일치했다. Docker 전체 7분 24초는 이번 단일 실행값이고 성능 인과 비교가 아니다.
별도 report-only PDF 비교는 72 DPI 크기 차이 경고 4건이 있었으며 직접 PDF gate 실패가 아니다.
원본과 PDF 네 파일의 실제 bytes와 후보 commit이 일치했다. 기준 PDF 등록에는 쪽수 원장
신규 2/2 행 두 개만 추가했으며, 기존 기대값을 바꾸지 않았다.

## 5. 시각 판정과 증적

- 독립 한컴 기준(후보 commit에 보존): `pdf/issue3587/c-form-labnote-001-stage11-filled-hwp-2020.pdf`,
  `pdf/issue3587/c-form-labnote-001-stage11-filled-hwpx-2020.pdf`.
- 최신 통합 화면: `output/7084/stage7/connected/hwp-after.png`,
  `output/7084/stage7/connected/hwpx-after.png`.
- 실행 비교: 같은 폴더의 `runtime.json` 및 기능 비활성 대조 이미지.
- 표준 compare·overlay·review 패널: `output/7084/stage7/visual/{hwp,hwpx}/`.

기존 메인테이너의 HWP/HWPX 시각 통과 판정을 유지한다. 이후 실제 브라우저 재검사에서
원본 두 건의 배율 1/1과 비활성 대조 0.364245/1을 재확인했다. 새로운 한컴 PDF를 생성하거나
자동 픽셀 검사로 메인테이너의 최종 판단을 대신하지 않았다. 흑백/컬러 글리프 모양 차이,
표 간격 등 별도 결함까지 해결했다고 주장하지 않는다.

통합본 두 화면은 각각 Stage 6의 승인 유지 화면과 PNG hash가 동일하다. 실제 Canvas와 기준 PDF를
형식별 2쪽 한 장씩 비교해 pixel match 90.78317%, 내용 중심 보조값 6.68869%를 얻었다.
대표 패널을 직접 확인했다. 보조값이 낮은 것은 표 간격·글꼴·글리프 모양 등 잔여 차이도
반영하기 때문이며, 이모지 성공률이나 전체 fidelity 통과율이 아니다. 대표 PNG의 PR 번호 기반
안정 경로 보존은 Open PR 채번 뒤 self-review 절차에서 한다.

## 6. 성능·자원 비용

Stage 5에서 빌드·회귀 검사가 끝난 뒤 동일 Windows Chrome과 입력으로 계측했다.
폰트 로드는 끝난 상태다. cold는 보충 cache만 비운 1회, warm은 5회 중앙값,
paint는 2쪽 5회, print는 portable 전환·전쪽 SVG 확보·문맥 복원 3회 중앙값이다.

| 입력 | cold / warm 준비 | portable / 활성 paint | 전쪽 print SVG | cache 항목 / JSON bytes |
| --- | ---: | ---: | ---: | ---: |
| HWP, 2쪽 | 1.2 / 0.2ms | 5.3 / 6.1ms | 3.2ms | 1 / 752 |
| HWPX, 2쪽 | 0.9 / 0.2ms | 6.1 / 6.4ms | 2.2ms | 1 / 752 |
| synam-001.hwp, 35쪽 | 130.5 / 51.9ms | 123.7 / 123.8ms | 574.5ms | 10 / 8,165 |

원자료/방법: `output/7084/stage5/studio-cost.json`, `measure-studio.mjs`.
warm 추가 측정은 모두 0건이다. JSON bytes는 직렬화한 자료 크기이며 실제 heap 메모리 크기가 아니다.
큰 문서의 warm 51.9ms·동기 print 574.5ms는 UI 응답에 영향을 줄 수 있어 미미하다고 평가하지 않는다.
Stage 4 대비 warm +4.3ms·print +23.9ms였지만 교차 반복 A/B 검사가 아니므로 전부 이번 코드의
인과적 증가율로 환산하지 않는다. 실제 heap과 엄밀한 전후 비용 분리는 미계측이다.

WASM은 Stage 4 대비 13,980 bytes 증가했고 Stage 6에서 추가 증가하지 않았다.
Stage 6 복구는 실패 처리 경로에 연결했으며, 이를 정상 paint 비용 0의 실측 증거로 쓰지는 않는다.

## 7. 지원 제한과 남은 절차

완료 범위는 승인한 **Canvas2D의 보충 진행폭 일치 및 보호 계약**이다. native/CanvasKit의
보충 폭 활성화, 모든 플랫폼의 동일 face 보장, VS16/ZWJ 전체 조형, 새 폰트 배포는 포함하지 않는다.
미지원 묶음은 기존 경로를 유지하므로 해당 묶음의 기존 표시 문제까지 고쳤다는 뜻이 아니다.

검증 경고도 남겼다. nextest 버전/지원하지 않는 JUnit 설정 경고, production build의 CanvasKit
`fs/path` 외부화·큰 chunk 경고, 별도 report-only PDF의 72 DPI 크기 차이 경고를 통과 수치와
구별한다. 경고를 숨기려고 설정·허용치를 바꾸지 않았다.

남은 순서는 다음과 같다.

1. 메인테이너의 최종 보고서·완료 범위 승인 — 2026-09-14 완료.
2. 최신 `upstream/devel` 통합 및 같은 후보 검증 — **Stage 7에서 완료**.
   파생 suite/manifest·로컬 폰트·접속정보·pkg/dist/output을 제출 대상에서 제외한다.
3. 별도 승인 후 원격 push·PR 생성·트리야지, 최종 HEAD의 CI 확인과 self-review.
4. 승인된 병합 절차 후 #7084의 종료 근거 게시·close 및 로컬 동기화.

승인된 통합 절편을 완료했고 결과는 로컬 문서 commit으로 보존한다. 원격 push·Open PR 생성은
다음 승인 대상이며, 댓글·원격 병합·이슈 상태는 변경하지 않았다.

## 8. 용어·약어 미주

- advance(진행폭): 다음 문자의 시작까지 배정하는 거리. 실제 칠해진 그림의 너비와 다르다.
- glyph / face: 그리는 글자 모양 / 그 모양과 메트릭을 제공하는 개별 글꼴 면.
- fallback(대체): 지정 글꼴이 문자를 제공하지 않을 때 다른 글꼴을 선택하는 처리. 추정 폭과 구별한다.
- em: 글꼴 크기의 상대 기준 단위. 실제 그림 높이가 항상 1em이라는 뜻은 아니다.
- HWPUNIT: 한글 문서의 길이 단위, 1/7,200인치. HWP/HWPX는 각각 한글의 바이너리/XML 문서 형식이다.
- DB (Database): 데이터베이스. 이 보고서에서는 내장 글꼴 메트릭 자료를 뜻한다.
- IR (Intermediate Representation): 입력 형식을 공통 엔진에서 처리하기 위한 중간 문서 표현.
- Canvas2D / backend: 브라우저의 2차원 Canvas 그리기 경로 / 실제 출력을 담당하는 엔진.
- descriptor / generation / snapshot: 해석된 글꼴 설정 / 문서·외부 폰트 변경 세대 / 해당 문맥의 고정 측정 자료.
- backendMeasured / verifiedSource: 같은 출력 엔진에서 실측한 출처 / 실제 글꼴 자원까지 검증한 출처.
- grapheme cluster / shaping / run: 사용자가 한 문자로 다루는 묶음 / 글리프와 배치를 구하는 조형 / 같은 처리 조건의 텍스트 구간.
- VS16 (Variation Selector-16): U+FE0F, 앞 문자의 이모지 표현을 요청하는 선택자.
- ZWJ (Zero Width Joiner): U+200D, 인접 문자의 결합 표현을 요청하는 문자.
- UTF-8 / UTF-16 (Unicode Transformation Format, 8-bit / 16-bit): Unicode 문자 인코딩 방식.
- WASM (WebAssembly): 브라우저 등에서 실행하는 바이너리 코드 형식.
- SVG (Scalable Vector Graphics) / PDF (Portable Document Format): 벡터 그림 형식 / 페이지 문서 형식.
- JSON (JavaScript Object Notation): 구조화한 자료의 텍스트 표현. 그 크기는 실행 중 메모리 사용량과 다르다.
- CI (Continuous Integration) / E2E (End-to-End): 지속적 통합 검사 / 사용자 경로를 연결해 실행하는 검사.
- CDP (Chrome DevTools Protocol): Chrome을 외부에서 제어·관찰하는 통신 규약.
- SHA-256 (Secure Hash Algorithm, 256-bit): 파일 내용의 동일성을 확인하는 해시 알고리즘.
- DPI (Dots Per Inch) / MiB (Mebibyte): 인치당 점 수 / 1,048,576바이트 단위.
- cold / warm cache: 재사용 자료가 없는 상태 / 자료가 준비되어 재사용되는 상태.
- portable: 브라우저 전용 측정에 의존하지 않는 출력 문맥. 모든 환경의 픽셀 동일성을 뜻하지 않는다.
- 음성 대조(negative control): 필요한 연결을 의도적으로 제거하여 검사가 결함을 실제로 검출하는지 확인하는 절차.
