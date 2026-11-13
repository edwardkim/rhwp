# planet6897 원 PR 5개 통합 검토 증적

원 PR #8972·#9030·#9034·#9036·#9041의 고유 commit 20개를 누적 체리픽했다. **원 PR이 주장한 문제를 확인하고 제품 메인터너 보정 없이 수용했다.** 충돌한 #8937·#8976은 제외했으며 `jangster77`의 reviewer 요청만 취소했다.

- 최종 검증 제품 code: `c6e852a92b3661136552a432d649e168e27f88d4`; base: `00d09f773a2f6c21604a4e6394df7152bbb26e8c`.
- 시각 producer: `605740c4363e9e6bacd84895d93abbc7d7bf62c3`; 고정 devel producer: `1003349996897c6ff097271e7296ae4d1a3d807f`.
- #9050의 테스트·문서 추가를 리베이스했고20개 patch·제품/출력 read path·입력 51개가 동일하다. 기존 산출물의 producer를 새 head로 바꾸어 표시하지 않았다. [재사용·최신 재검증](rebase-validation.json), [원 commit→통합 commit](source-plan.json).
- 이 문서는 완료된 로컬 검증 기록이다. 원격 통합 PR의 최신 CI·merge는 게시 이후 별도로 확인한다.

## 원 PR의 주장 페이지

같은 입력·독립 데스크톱 한컴2022 Print PDF,96 DPI, compat2022, canonical PDFium, 색상 경계 보정 전 2px raw 실루엣이다. 원본/PDF는 원 contributor commit에 포함된 파일을 재사용했다. 파일명2020은 engine bucket이며 실제 Creator는 `Hwp 2022 12.0.0.4547`, Producer는 `Hancom PDF 1.3.0.547`이다. 웹한글기안기/Cairo PDF를 기준으로 쓰지 않았다.

| 원 PR | 전쪽 수 | Native 최저 | fresh WASM 최저 | 검토 기록 |
| --- | --- | --- | --- | --- |
| [#8972](https://github.com/edwardkim/rhwp/pull/8972) | 2 | 98.30058% | 98.50289% | [개별 리뷰](../../archives/pr_8972_review.md) |
| [#9030](https://github.com/edwardkim/rhwp/pull/9030) | 1 | 99.84609% | 99.84609% | [개별 리뷰](../../archives/pr_9030_review.md) |
| [#9034](https://github.com/edwardkim/rhwp/pull/9034) | 4 | 97.82783% | 97.82783% | [개별 리뷰](../../archives/pr_9034_review.md) |
| [#9036](https://github.com/edwardkim/rhwp/pull/9036) | 1 | 100.00000% | 100.00000% | [개별 리뷰](../../archives/pr_9036_review.md) |
| [#9041](https://github.com/edwardkim/rhwp/pull/9041) | 2 | 99.99406% | 99.99915% | [개별 리뷰](../../archives/pr_9041_review.md) |

전쪽 TSV와 각 PR 자신의 같은 쪽 raw/PDF/review/overlay는 `claims/pr번호/{native,wasm}/`에 보존했다. [이미지 hash](claim-rasters.json), [직접 판독](direct-reading.json), [원 작성자의 고정 주장·과거 실행](original-pr-claims.json)을 구분한다. 날짜 코드 표시, 제목 굵기, 일부 도표 색상 등 원 PR이 해결한다고 주장하지 않은 사항을 새 개선 작업으로 확대하지 않았다.

## base 대조와 보호 기준

- [base/후보 전쪽 비교](base-candidate-qualification.json): 비교 가능한 3,552쪽, 페이지별 2%p 초과 하락 0쪽. [전체 수치 TSV](base-candidate-pages.tsv).
- #9030·#9034는 고정 base Native와 실제 직접 로드한 WASM이 원본을 파싱하지 못한다. 가상의0% TSV를 만들지 않았다. [실제 WASM 오류·초기화](base-wasm-parser-failures.json).
- [정본 보호 기준](../visual-protected-baseline/README.md)의18조건/경로, Native·fresh WASM 각각1,771쪽에서 공식 두 자리 표시값 하락 0쪽,90% 미만0쪽, 최저 98.36252%. 후보와 고정 base TSV는 `protected/{candidate,base}/{native,wasm}/`에 구분했다.
- 필수 편람 HWPX Native 383쪽 `--handbook-gate`를 실제 실행했다. [후보 보호 검증](protected-candidate-qualification.json), [전체 후보 46개 조건 3,562쪽의 PNG hash·쪽수·누락/중복 감사](candidate-page-identity-audit.json).
- **Visual Sweep은 최소 글꼴 환경이 아니다.** 기존 보호 증적과 같은 원본 글꼴 program bytes·순서·인증된 환경을 공급했다. [글꼴 조건 provenance](font-condition-provenance.json). 비공개 글꼴 바이너리는 Git에 포함하지 않았다.
- sample16 HWPX51쪽의 원형 기호는 누락된 기존 승인 글꼴 환경을 검증 wrapper에 복원한 뒤 Native/fresh WASM 모두100.00000%다. 제품 수정으로 보고하지 않는다. [review](font-condition-repair/review-p051.png) · [overlay](font-condition-repair/overlay-p051.png).

## Rust와 실제 제품 소비 경로

- 리베이스 후 최소 글꼴 macOS 프로세스 격리에서 전체 nextest 13,430 PASS/0 FAIL/45 skip, 실제 테스트 632.595초. 원 PR 관련6건과 새 devel의 CellBreak3건도 실제 PASS. [실행 요약](individual-test-execution.json), [격리 조건](rust-font-isolation.json).
- fmt·Native/WASM/전체 target Clippy·workspace build·Native Skia lib 4,034 PASS/12 ignore·PNG 2 PASS·PDF 36 PASS·manifest/unit tiers를 통과했다. [명령·exit·로그 hash](rust-validation.json). 코드/조건이 같아 재사용한 gate와 리베이스 후 실제 실행을 구분했다.
- fresh WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` **로컬 대체 빌드**다. Docker 최적화 통과라고 표시하지 않는다.
- 파싱 원본2종의 실제 편집→저장→재열기→재저장→재열기 및 HWP/HWPX 교차 경로 62회 호출이 성공했다. 본문·표·머리말과 sentinel을 검증했다. HWP3의 기존 #4957 제어 slot 직렬화 계약을 구분했다. [실제 CLI 증거](actual-cli-qualification.json).
- [Studio 실제 소비 36건](actual-browser-qualification.json): Canvas2D 32건(기존 글꼴 준비 guard로 CK 요청이 fallback한20건 포함), 실제 지원 CK software 4건. fallback 20건을 CK 통과로 세지 않았고 guard를 우회하지 않았다. 원 PR별 주장 페이지의 실제 화면/print·본문/DOM 합성·원점·순서를 직접 확인했다. CK 기본 screen의 스캔 보간 차이는 report-only RGB 진단에 그대로 남겼다.
- [초기화 응답과 build identity](actual-initialization-qualification.json): root pkg·Studio/public·고정 package의 JS/WASM hash가 같다. 실제 Studio 36번의 초기 WASM 응답도 같다. Vite 개발 JS 응답은 변환·inline source map 때문에 원 pkg hash와 다르지만 정본 Vite의 현재 pkg 변환 코드와 같음을 별도 실제 초기화로 확인했다. 초기화 뒤 진단용 JS/WASM 재fetch로 대체하지 않았다.

## 검증 도구의 무효 실행

리베이스 checkout 중 새 입력 경로가 잠시 없어졌던5번의 base Native 호출은 제품 실패가 아닌 무효 실행으로 분리했다. 원본 hash 복원 뒤 그5건만 직렬 재실행했다. [무효 실행과 정정](invalid-operational-attempts.json). 초기 HTTP 관측의 inspector cache·URL 필터 오류도 도구 오류로 구분했으며 제품/PDF/비교 문턱을 바꾸지 않았다. 정식 증적에 `.log`, 임시 스크립트, package 바이너리, 비공개 글꼴을 포함하지 않는다.
