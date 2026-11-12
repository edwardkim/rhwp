# PR #9024 리뷰 — 명시적 호스트 글꼴 조판 참여

## 최종 판정

**승인 — #8946 범위의 구현·로컬 검증과 code head CI를 확인했다.** 작성자 `postmelee`의 self-review이며 GitHub 승인 review나 merge 승인을 뜻하지 않는다. 다음 문서-only trailing head의 CI·병합 가능성 확인과 작업지시자의 merge 승인은 별도 조건이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9024](https://github.com/edwardkim/rhwp/pull/9024) / postmelee / devel |
| 기준 devel | `a85d3265dd10b316d1a0ae029ca3f67245b38b77` |
| 로컬 코드·회귀 검증 head | `8af32b9d991ee6b6c7bf082aaf6cc6caac1b86ab` |
| 원격 code candidate | `f7eb7547e7d853394a7ae33976e41d5a74c917d0`; 위 검증 head와 제품·테스트 소스 동일 |
| 이슈·범위 | Closes #8946; Refs #9012; Canvas2D의 명시적 선택·환경 갱신·수명 및 회귀 보호 |
| 작성 시점 상태 | Open, non-Draft, mergeable, 요청된 reviewer·제출 review 없음 |
| 규모 | 26 commits, 1,131 files, +306,552/−60. 대부분 전쪽 TSV·조사 증적이며 제품·회귀 변경을 별도 검토했다. |

기본 절차는 collaborator_self_merge, 보조 절차는 intake_and_review·local_validation·visual_fixture_evidence·review_only_fast_pass·대형 PR 검토다. merge나 admin 우회는 실행하지 않는다.

## 구현과 수용 조건

`setProvider(provider, { layout: true })`를 연결 단위 opt-in으로 추가했다. 옵션 생략·false·공급자 없음의 기존 동작을 보존한다. 선언 프로그램 준비는 목록의 이름만 신뢰하지 않고 실제 SFNT/TTC를 읽어 SHA와 faceIndex를 원본 선택 신원에 연결한다. 자동 환경은 원본 메트릭 보존 정책을 유지하며 사용자 명시 환경과 내장 글꼴 우선순위를 침범하지 않는다.

초기 문서 열기와 변경 알림은 `prepareDeclaredFontEnvironment`를 공유한다. `prepareDeclaredHostFontPrograms` → 공통 환경 생성·적용 → `NativePuaFontSession` → Rust의 원본/선택 digest → Canvas2D paint까지 실제 파일을 추적했다. 문서·공급자 세대가 바뀌면 오래된 응답을 거부한다. 활성화된 CFF paint 윤곽은 `withPageOutlines`의 try/finally로 현재 paint 동안만 공급하며 측정/coverage와 수명을 구분한다. 일반 쪽·filtered·patch·머리말 preview·portable 경계의 E2E를 확인했다.

[수용 조건별 구현·회귀](../../working/task_m100_8946_stage5.md)와 [최종 실행](../../working/task_m100_8946_stage8.md)에 연결한다. TTC 교체, 같은 ID/revision의 공급자 교체, 제거·재공급·해제·실패 복구, dirty/undo/redo·원본 이름·portable 저장을 포함한 97개 E2E 조건이 모두 통과했다.

## 조판 원칙 준수

| 항목 | 판정 | 실제 확인 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증·PR 상태 | 충족 | 최소 글꼴 전체 회귀, Skia·PDF·lint·Studio·호스트 최적화 fresh WASM·편람 gate를 제출 전에 완료; Open PR |
| 기존 이슈·해결 PR의 선행 분석 | 충족 | #7403/#7405, #8371/#8397과 후속 #8958/#8984/#8956/#9017의 계약·통합 SHA·독립 증거를 단계 1·5~8에서 추적 |
| 구현 근거·일반성 | 충족 | HFT 선언과 실제 CFF의 glyph 공급·확인된 굵기 전진 계약을 공통 punctuation/legacy helper에서 소비; 독립 Print TJ/Widths 근거, 비 HFT·내장·TTF·미확인 family 대조 |
| 측정·배치 일관성 | 충족 | 원본/선택 digest와 faceIndex를 보존; compact SFNT는 측정 표만 줄이고 원본 신원·paint 공급을 유지; 최종 run 소유와 순서 검사 |
| 리베이스 반영 기능 | 충족 | #8945 경계 5개·8쪽, #8958/#8984 원 승인 범위, #8956 실제 ByPath API, 최종 #9017 13개 입력·52쪽과 Studio 6조건/반복 저장을 각 base/head에서 검증 |
| 최종 backend 소비 | 충족 | Native·fresh WASM·실제 Host 전쪽 및 실제 수신 WASM SHA; CanvasKit 기존 계약, 화면/portable 자원 분리 확인 |
| 분할·이어받기 변경 | 비해당 | pagination·rowspan·clipping·continuation 생산/예약/컷 로직은 변경하지 않음; 보호 문서 전쪽 결과는 별도 보존 |
| 줄 소속·점유 | 충족 | 저장 줄 재사용·편집 후 재조판을 각각 실행; 문장부호 소유, 내용 순서·누락·중복·포함·겹침과 후속 빈 줄 점유를 검증 |
| 독립성·반례 | 충족 | 같은 입력·독립 Print; 합성 계약과 실물 출력을 구분. 단계 5의 기존 base 14 PASS/신규 2 FAIL → 후보 16 PASS 음성 대조 |
| baseline·허용치 | 비해당 | 보호 TSV·golden·허용치 변경 없음. 문서용 비교 집계 TSV의 CRLF만 LF로 정규화 |
| 회귀 추가 선행 조건 | 충족 | HWPX 전 763쪽 Native/WASM 최저 96.35%/99.56%와 확대 판독 뒤 관계형 회귀 추가. 공유 딸기 표본도 양쪽 100.00% 확인 후 기존 검사 수용 |
| 90% 이후 확대 판독 | 충족 | 표지 문구·그림, 제목 23곳, 350/608/686쪽 문장부호, 729쪽·보호 기호의 Print/Native/WASM 같은 영역 대조. 최종 전쪽 PNG·tree 동일성으로 이전 판독 연결 |
| 주장·검증 범위 | 충족 | Canvas2D opt-in 범위 유지. 전체 공급원/출력 통합·HFT 원본 메트릭 전면 교체를 완료로 주장하지 않음 |

#9012와 공유하는 원 커밋 `4e66e76981`의 딸기 계약·회귀·증적만 이력에 수용했다. 제품 소스는 수용 전후 동일하고 원 PR head `71c0024956`와 병합 시뮬레이션이 통과했다. 연결 그림 인쇄 변경은 포함하지 않았다.

## 시각 검증과 입력 커밋 확인

[최종 원장](../assets/issue8946/stage8-20261112/README.md)의 Native/fresh WASM 전쪽 TSV·대표 PNG·producer를 확인했다. [입력 audit](../assets/issue8946/stage8-20261112/final-input-commit-audit.json)은 실제 입력 67개 중 66개의 커밋 바이트 일치를 확인한다. 기존 devel의 96,185,197바이트 HWP3 PDF는 원 blob을 변경 없이 재사용한다. 새 HWPX 독립 Print는 90,804,612바이트·763쪽·SHA-256 `45bb4d29b0afea67fb4818a9b605a84201cbdf2998b6c31310d36fb85294cad8`이며 로컬 `pdf/issue8946/hwp3-sample10-hwpx-images-embedded-2024.pdf`에만 보관한다. 출처·명령·동일 그림 운반본 근거는 [단계 4](../../working/task_m100_8946_stage4.md)다. **충족 — 새 대용량 PDF 로컬 보관.**

| 763쪽 입력 | Native 최저 raw | fresh WASM 최저 raw | Host 최저 raw |
| --- | ---: | ---: | ---: |
| HWPX | 96.35% | 99.56% | 99.36% |
| HWP3 | 98.77% | 99.55% | 99.35% |

모든 쪽 90% 이상이며 쪽수·대응 내용·세 외부 그림을 포함한다. 최신 기준 대비 네 Native/WASM 조합의 최대 하락은 0%p, HWPX 기존 18쪽 미달은 해소됐다. 정본 18개 입력·양쪽 3,542쪽의 공식 표시값 하락 0, 편람 Native 383쪽 gate 통과다. 기본 9조건의 전쪽 PNG·tree와 실제 합성 첫 화면은 기준/후보 동일하다.

표지와 딸기 12/18/24pt 여섯 행, 새 HTML 붙여넣기 결합 표본을 직접 판독했다. 350쪽 괄호·물음표의 모양과 앞뒤 간격, 보호 문서 삼각형·주변 본문도 기존 확대 증적에서 확인했다. 최종 출력의 전쪽 동일성 근거를 함께 사용했으며 재사용 자료를 새 300 DPI 실행으로 표시하지 않았다. raw 100%를 RGB 동일로 해석하지 않는다. 글꼴 프로그램과 서비스 인증 자료는 로컬에만 유지한다.

첫 기준 WASM은 후보 캐시를 잘못 재사용해 무효화했다. 기준을 재컴파일하고 무효 범위 전부를 재실행했다. 서로 다른 실제 SHA·후보 전용 export 유무·브라우저 수신 body로 출처를 확인했으며 최초 잘못된 실행은 판정에서 제외했다.

## 로컬 및 원격 실행

로컬: Rust 전체 13,341 PASS(기존 skip 45), Native Skia library 4,034 PASS, placeholder 2·직접 PDF 36·HFT 집중 22 PASS. fmt·Native/WASM/workspace Clippy·workspace build·suite 정책, Studio 2,789 PASS(기존 skip 2), TypeScript·frontend 계약·Canvas2D/CanvasKit E2E 통과.

정확한 원격 code candidate `f7eb7547e7d853394a7ae33976e41d5a74c917d0`의 [CI 원장](../assets/issue8946/stage8-20261112/code-ci.json):

- [CI](https://github.com/edwardkim/rhwp/actions/runs/43246442646): success. A 3,796 + B 2,211 + C 2,637 + D 2,295 + E 2,217 = 13,156 PASS; 예상·실행 수 13,156 일치. Native Skia·frontend·lint·Build & Test 성공. 로컬 전체 suite 수와 CI 선택 수는 구분한다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43246442710), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43246442598), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43246443631), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43246442642): 모두 success.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/43252485911): success.

## 남은 merge 조건과 후속 기록

문서-only trailing head의 실제 fast-pass·required checks·mergeability를 확인하고 작업지시자의 별도 merge 승인을 받아야 한다. 승인·merge·이슈 종료는 실행하지 않았다.

merge 후 댓글은 별도 승인된 후속 처리에서만 게시한다. 그때 최종 merge SHA·CI URL, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), `mydocs/pr/assets/issue8946/stage8-20261112/`의 대표 PNG를 merge SHA 고정 raw URL로 연결한다. 한국어 존댓말 문안을 UTF-8 파일로 게시하고 API로 재조회한다. 이 계획은 댓글 게시 승인이 아니다.
