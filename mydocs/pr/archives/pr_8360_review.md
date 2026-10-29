# PR #8360 리뷰 — crates.io 준비와 요청 단위 PDF 처리

## 최종 판정

**승인 — #8174의 구현·필수 로컬 검증과 정확한 code head의 CI·GHAS 확인을 완료했다.** self-review 경로이며 reviewer를 지정하지 않았다. 사용자로부터 Open PR·CI·self-review·merge와 후속 처리, 이번 보안 경고 두 분류의 명시 승인을 받았다. 문서 trailing head의 CI·최신 merge tree·일반 merge는 기록 시점의 남은 조건이다.

## 접수와 source

| 항목 | 확인 값 |
| --- | --- |
| PR / base / milestone | [#8360](https://github.com/edwardkim/rhwp/pull/8360) / devel / v1.0.0 |
| 검증한 code head | `f1437377645256367fd53bc1eae724a3c4cb5e19` |
| 고정 최신 devel | `f291d772a105d46a4d4d7f40ed13e82391ebdc1a` |
| 생산 Rust / 회귀·binding | `fa254efdfe9e33c4dcaf54c0ac3e534819111a9e` / `a4111b02fb75bc9cf869cdbf26af6d8e95ea187c` |
| 실제 package source | `27a3c1b86cc50a115aa63b9a4f97d2bd4e297a62` |
| 이전 head | `a89456207d`의 Render Diff SSR 하네스 실패를 보정했다. 이후 생산 Rust/Studio renderer·생성 JS/WASM·archive payload는 동일하다. |
| 관련 이슈 | [#8174](https://github.com/edwardkim/rhwp/issues/8174). 실제 crates.io 게시·owner·태그는 별도다. |

원 작업 브랜치의 120개 커밋을 보존하고 최신 devel의 #8348 기여 PR 20개와 #8350 기록을 실제 통합했다. 새 base의 API/저장·재열기·폰트·그림·조판을 baseline과 후보에서 대조했다. 충돌 해소나 컴파일만으로 기능 보존을 판정하지 않았다. [T09 이력](../../working/task_m100_8174_t09_completion.md)과 [통합 보고](../../report/task_m100_8174_report.md)에 source·입력·명령을 연결한다.

## 구현과 조판 원칙 검토

14개 crate, 본체 0.9.0·Rust 1.93.1·공식 API 계약과 외부 소비/수동 게시·부분 실패 복구 절차를 준비했다. 기본 기능 F01–F08과 명시 제외 빌드의 경계를 문서화했다. 같은 요청의 논리 예산을 parser/decoder → 공통 Core/조판 → 첫 저장 → Worker 전달/편집 문서 재열기에 연결했다. Linux 작업자 종료·취소·sticky 실패·이전 문서 보존·다음 정상 요청 복구를 실제 실행했다. 논리 카운터를 전체 process heap/RSS 보장으로 취급하지 않으며 `parserAllocationBoundsProven=false`와 호스트 책임을 유지한다.

PDF 편집 모드의 선행 Paper-relative 본문 owner와 첫 flow를 같은 producer 결과로 보존했다. 마지막 글줄 내용과 추론한 꼬리 간격도 분리해 관측된 다음 원점까지 실제 남은 밴드만 같은 LineSeg·누적 원점에 전달한다. 확정 본문 owner·수용한 줄 상자·50 HU 원점 검증을 유지했다. 문서 ID 분기, 잘못된 좌표 clamp, 내용 숨김, backend별 별도 보정으로 처리하지 않았다. 구현 소비 경로와 반례는 [상세 검토](pr_8360_review_impl.md)에 기록한다.

관련 세 PDF는 Native/fresh WASM 전쪽 최저 98.48% 이상·독립 원본 PDF 쪽수 일치와 확대 판독 후 회귀를 추가했다. macOS 3쪽은 99.79%/99.80%, 2020 12쪽 99.37%/99.35%, 2010 13쪽 99.53%/99.53%다. 독립 paint baseline 96 DPI ±2px·줄 순서/소유를 검사한다. 같은 새 검사 원문은 `ceda3a7953`에서 FAIL, 최종 생산 코드에서 PASS이며 기존 4개 검사를 포함한 정식 모듈 5건도 통과했다. GitHub Archive E에서도 새 검사가 실제 PASS했다. 기존 기대값·허용치·쪽수를 완화하지 않았다.

2010 fidelity 15쪽은 두 줄 라벨의 닫는 괄호 방문 순서를 별도로 판독했다. 2,055자 수량과 라벨 외 본문 순서가 같고 독립 PDF/양쪽 실행의 실제 glyph bbox 차이는 최대 1.4px다. 단순 DFS 문자열 차이를 내용 누락으로 처리하거나 제품/정식 회귀 기대값을 바꾸지 않았다.

## 입력·전쪽·직접 판독

[전체 TSV 원본과 대표 PNG](../assets/issue_8174/t09/README.md)·[검증 원장](../assets/issue_8174/t09/verification_manifest.json)·[입력 commit 대조](../assets/issue_8174/t09/input-commit-verification.json)를 제출했다. 공개 입력 279개의 실제 바이트가 검토 commit과 같다. 비공개 15쪽 문서는 식별 원본·이미지·텍스트를 공개하지 않았고 수치만 보조 증거로 기록했다.

Native와 표준 mixed rlib/cdylib·실제 wasm-opt fresh WASM은 각각 166개 검증 항목·1,682쪽을 완료했다. 같은 입력의 다른 실행 경로와 중복 대조군을 포함하며 고유 문서 수가 아니다. raw 최저 91.04%/91.15%, 90% 미만·누락·쪽수 불일치 0건이다. 기준/후보 3,156쪽 대조에서 페이지별 2%p 이상 하락도 없다. PDF 이외의 대조 PNG 2,972개 중 의도된 규제분석서 20·21쪽의 네 이미지 외 2,968개가 바이트 단위로 같다.

실제 본문 글꼴과 원본 `HYHWPEQ`·`HancomEQN`을 지정했다. 수식/미주 45쪽은 각 실행 경로의 base/후보 PNG가 같고, 수식 glyph·기준선·분수·첨자를 같은 배율로 판독했다. 본문용 설치/subset/HFT Print 프로그램과 수식 두 파일의 전체 바이트 일치를 구분했다. 글꼴 바이너리를 재배포하거나 glyph/advance를 점수에 맞춰 수정하지 않았다.

CFF/Type3, 표/캡션·중첩 셀, inline 개행/분배, JPEG alpha, 규제분석서 21·28·29쪽, PDF 3·12·13쪽을 직접 판독했다. 28·29쪽은 양쪽 98% 이상과 기존 ±2px 보호 조건을 유지한다. 기존 획/보간·수식 미세 간격/고정 폭 줄바꿈 차이는 base에도 같으며 전체 피델리티 해결로 주장하지 않는다.

## 실행 검증

| 로컬 검증 | 실제 결과 |
| --- | --- |
| 전체 Rust 회귀 | nextest 12,038 PASS·46 skip·실패 0 |
| 표준 release | opt3/LTO/cgu1 build·library 3,800 + 내부 182 PASS, 기존 12 ignore·승인한 page-border 1건 제외, CLI smoke PASS |
| Native Skia | 전체 library 3,857 + 내부 182·glyph replay·누락 그림·직접 PDF gate PASS |
| 기타 Rust | workspace build/전체 대상 Clippy·fmt·내부 248·backend 5,938·JPEG 7·feature 제외·doctest 8 PASS/3 ignore |
| Studio / editor | Node 2,665 PASS·2 skip·타입/build, 17 Workers의 271개 단언·실제 Studio 오류/취소/보존/복구·표시/편집·HWP/HWPX 저장, editor 4개 gate PASS |
| 새 devel 보존 | fontless Native 후보 111/기준 110, API 305개 단언/44개 기록·환경 설정과 AI provider/Canvas2D 경계 PASS |
| 포장 / 소비 | 14개 실제 archive·16,603,252 bytes의 strict gzip/원본/고지/digest, path/patch 없는 Rust·CLI·공개 계약 54개·WASM on/off PASS |

code head `f1437377645256367fd53bc1eae724a3c4cb5e19`의 다음 workflow는 전부 완료·성공했다.

| 검사 | 실행 | 결과 |
| --- | --- | --- |
| CI | [run 40821242533](https://github.com/edwardkim/rhwp/actions/runs/40821242533) | success |
| CodeQL | [run 40821242513](https://github.com/edwardkim/rhwp/actions/runs/40821242513) | success |
| Render Diff | [run 40821242540](https://github.com/edwardkim/rhwp/actions/runs/40821242540) | success |
| Adapter inter-diff | [run 40821242576](https://github.com/edwardkim/rhwp/actions/runs/40821242576) | success |
| Proptest roundtrip | [run 40821242569](https://github.com/edwardkim/rhwp/actions/runs/40821242569) | success |

별도 GHAS [CodeQL 보안 check](https://github.com/edwardkim/rhwp/runs/123275058671)와 Build & Test도 success다. GitHub default-feature 5개 shard는 총 11,845 PASS·45 skip이다. 로컬 승인 제외였던 page-border 검사도 Archive A에서 실제 PASS했다. 로컬/CI 실행 범위의 수치를 섞어 합산하지 않았다.

## 하네스·보안·남은 차이

이전 Render Diff 실패는 새 SVG raster wrapper를 Vite SSR에서 초기화하지 않은 하네스 문제였다. 실제 제품 module을 초기화하고 SVG image replay를 검사했다. 단색 glyph의 path 대조군으로 기존 drawPath 검사와 resource/hash·손상 이미지·readiness 단언을 유지했다. 같은 보정 하네스가 기준/후보 제품 양쪽에서 PASS, CanvasKit 글꼴 coverage도 PASS이며 새 Render Diff CI도 성공했다. 생산 renderer의 거절/예산 조건을 완화하지 않았다.

[보안 검토](../../working/task_m100_8174_security_bot_review.md)에 173건·688개 flow와 도입/분석 시점을 기록했다. 사용자 명시 승인으로 글꼴 메트릭 `Vec<bool>`의 이름 heuristic 오탐 173건을 false positive, 합성 E2E key #1을 used_in_tests로 분류했다. 기존 29건과 query/language/path·보안 설정은 유지했다. 새 Rust SARIF의 192개 결과/flow 위치는 이전과 동일하므로 이를 코드에서 경고 flow를 제거한 구현 효과라고 주장하지 않는다. Dependabot은 비활성 상태여서 취약점 부재를 판정하지 않았다.

[추가 성능 관측](../assets/issue_8174/t09/performance.json)은 로컬 글꼴을 준비한 115쪽 HWP/HWPX의 16회를 완료했다. UI 열기 중앙값은 약 12.26%/17.52% 짧았고 편집은 혼합 결과다. HWPX IME의 약 201ms와 화면 대기는 양쪽에 남는다. 415쪽은 양쪽 초기 UI 완료 지연으로 성능 미판정이며 통과·보편적인 개선으로 기록하지 않았다. 기존 #4396 field parameter 경고와 #8039 불완전 입력 저장 거절도 전체 무손실 지원으로 확대하지 않는다.

## 남은 원격 순서

이 review·구현 검토·오늘할일만 문서 trailing으로 반영하고, 최신 base의 merge tree·링크·기존 오늘할일 보존을 검증한다. trailing head의 실제 CI/fast-pass와 GHAS를 확인한 뒤 승인 범위의 일반 squash merge를 실행한다. merge 뒤 devel 동기화·issue/comment·전용 branch/worktree와 임시 output/debug 정리를 마친다. 실제 registry 게시·owner 변경·태그는 수행하지 않는다.
