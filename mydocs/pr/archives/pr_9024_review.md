# PR #9024 리뷰 — 명시적 호스트 글꼴 조판 참여

## 최종 판정

**승인 — #8946 범위의 구현·로컬 검증과 code head CI를 확인했다.** 작성자 `postmelee`의 collaborator self-review이며 GitHub 자기 승인 review는 아니다. 작업지시자의 기록 갱신·push·병합 승인은 이미 받았다. 남은 조건은 문서-only trailing head의 실제 CI·병합 가능성 확인이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9024](https://github.com/edwardkim/rhwp/pull/9024) / postmelee / devel |
| 기준 devel | `dd7f453898776c262c00e862e10f47d22a4d6ceb` |
| 로컬 코드·회귀 검증 head | `a6cdfdce4cc7d360df6ee1ef8d9ea978340d1b04` |
| 원격 code candidate | `39a3774de60c81a3b803d158c044ddc195733eb6`; 위 검증 head와 제품·테스트 소스 동일 |
| 이슈·범위 | Closes #8946; Refs #9012; Canvas2D의 명시적 선택·환경 갱신·수명 및 회귀 보호 |
| 작성 시점 상태 | Open, non-Draft, mergeable, 요청된 reviewer·제출 review 없음 |

기본 절차는 collaborator_self_merge, 보조 절차는 intake_and_review·local_validation·visual_fixture_evidence·review_only_fast_pass·대형 PR 검토다. 최종 검증 뒤 정상 병합을 진행하며 admin 우회는 사용하지 않는다.

## 구현과 수용 조건

`setProvider(provider, { layout: true })`를 연결 단위 opt-in으로 추가했다. 옵션 생략·false·공급자 없음의 기존 동작을 보존한다. 선언 프로그램 준비는 목록의 이름만 신뢰하지 않고 실제 SFNT/TTC를 읽어 SHA와 faceIndex를 원본 선택 신원에 연결한다. 자동 환경은 원본 메트릭 보존 정책을 유지하며 사용자 명시 환경과 내장 글꼴 우선순위를 침범하지 않는다.

초기 문서 열기와 변경 알림은 `prepareDeclaredFontEnvironment`를 공유한다. `prepareDeclaredHostFontPrograms` → 공통 환경 생성·적용 → `NativePuaFontSession` → Rust의 원본/선택 digest → Canvas2D paint까지 실제 파일을 추적했다. 문서·공급자 세대가 바뀌면 오래된 응답을 거부한다. 활성화된 CFF paint 윤곽은 `withPageOutlines`의 try/finally로 현재 paint 동안만 공급하며 측정/coverage와 수명을 구분한다. 일반 쪽·filtered·patch·머리말 preview·portable 경계의 E2E를 확인했다.

[수용 조건별 구현·회귀](../../working/task_m100_8946_stage5.md)와 [최종 실행](../../working/task_m100_8946_stage10.md)에 연결한다. TTC 교체, 같은 ID/revision의 공급자 교체, 제거·재공급·해제·실패 복구, dirty/undo/redo·원본 이름·portable 저장을 포함한 97개 E2E 조건이 모두 통과했다.

## 조판 원칙 준수

| 항목 | 판정 | 실제 확인 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증·PR 상태 | 충족 | 최소 글꼴 전체 회귀, Skia·PDF·lint·Studio·호스트 최적화 fresh WASM·편람 gate를 제출 전에 완료; Open PR |
| 기존 이슈·해결 PR의 선행 분석 | 충족 | #7403/#7405, #8371/#8397과 후속 #8958/#8984/#8956/#9017의 계약·통합 SHA·독립 증거를 단계 1·5~8에서 추적 |
| 구현 근거·일반성 | 충족 | HFT 선언과 실제 CFF의 glyph 공급·확인된 굵기 전진 계약을 공통 punctuation/legacy helper에서 소비; 독립 Print TJ/Widths 근거, 비 HFT·내장·TTF·미확인 family 대조 |
| 측정·배치 일관성 | 충족 | 원본/선택 digest와 faceIndex를 보존; compact SFNT는 측정 표만 줄이고 원본 신원·paint 공급을 유지; 최종 run 소유와 순서 검사 |
| 리베이스 반영 기능 | 충족 | #8945 경계 5개·8쪽, #8958/#8984 원 승인 범위, #8956 실제 ByPath API, 이후 반영 PR을 포함한 148조합·946쪽, 연결 표지 8쪽과 Studio·ByPath·CLI·여권 반복 저장을 최신 base/head에서 검증 |
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

[최종 원장](../assets/issue8946/stage10-20261114/README.md)의 Native/fresh WASM 전쪽 TSV·대표 PNG·producer를 확인했다. [입력 audit](../assets/issue8946/stage10-20261114/input-commit-audit.json)은 입력·독립 PDF 190개의 커밋 바이트를 확인한다. 새 HWPX 독립 Print는 90,804,612바이트·763쪽·SHA-256 `45bb4d29b0afea67fb4818a9b605a84201cbdf2998b6c31310d36fb85294cad8`이며 저장소 밖의 안정적인 로컬 경로에 보관했다. [보존 기록](../assets/issue8946/stage9-20261113/local-reference-preservation.json)과 [단계 4](../../working/task_m100_8946_stage4.md)의 동일 그림 운반본 출처를 유지한다. **충족 — 새 대용량 PDF 로컬 보관.**

| 763쪽 입력 | Native 최저 raw | fresh WASM 최저 raw | Host 최저 raw |
| --- | ---: | ---: | ---: |
| HWPX | 96.35% | 99.56% | 99.36% |
| HWP3 | 98.77% | 99.55% | 99.35% |

모든 쪽 90% 이상이며 쪽수·대응 내용·세 외부 그림을 포함한다. 최신 기준 대비 네 Native/WASM 조합의 최대 하락은 0%p, HWPX 기존 18쪽 미달은 해소됐다. 정본 18개 입력·양쪽 3,542쪽의 공식 표시값 하락 0, 편람 Native 383쪽 gate 통과다. 기본 9조건의 전쪽 PNG·tree와 실제 합성 첫 화면은 기준/후보 동일하다.

표지와 딸기 12/18/24pt 여섯 행, 새 HTML 붙여넣기 결합 표본을 직접 판독했다. 350쪽 괄호·물음표의 모양과 앞뒤 간격, 보호 문서 삼각형·주변 본문도 기존 확대 증적에서 확인했다. 최종 출력의 전쪽 동일성 근거를 함께 사용했으며 재사용 자료를 새 300 DPI 실행으로 표시하지 않았다. raw 100%를 RGB 동일로 해석하지 않는다. 글꼴 프로그램과 서비스 인증 자료는 로컬에만 유지한다.

단계 10의 새 32쪽 직접 판독과 기존 후보 대비 원본 3,052쪽 PNG 동일성을 확인했다. 연결 표지 8쪽은 raw 100%·배치 트리 동일이나 굵은체·혼합 WASM 2쪽의 기관명 RGB 래스터는 달랐다. 같은 영역의 독립 Print 3배 확대에서 위치·간격·내용 보존과 RGB 오차 감소를 확인했다. raw 100%를 RGB 동일로 해석하지 않는다. 세금 서식·교육 개요의 기존 글꼴 농도·기호·색상 잔차는 devel과 동일하며 이 PR의 개선으로 주장하지 않는다.

초기 Native의 오래된 chart 의존성 캐시와 잘못 선택한 최소 글꼴 profile은 판정에서 제외하고 해당 전체 검증을 재실행했다. 최종 Native SHA는 `6a4fc598bfd5ee293a193c7f99e5a077bc359e87305d03803f5c60cd52fde585`, 전쪽·브라우저 WASM SHA는 `4737807f3978d90bc163bfb109c684c05f69c26c35d6a6e5f1bc33b65813c0bb`다. 강제 WASM 재빌드는 동일 의존성 소스의 진단 경로·주소 차이로 구분했고, 명령 구조와 실제 9개 출력의 동일성을 확인했다. 진단 바이너리를 전쪽 검증 producer로 바꾸어 기록하지 않았다. 근거와 실패 정정은 최종 원장에 연결했다.

## 로컬 및 원격 실행

로컬: 최소 글꼴 Rust 전체 13,483 PASS / 0 FAIL / 45 SKIP, Native Skia library 4,034 PASS / 12 ignore, placeholder 2·직접 PDF 36·HFT 집중 22 PASS. fmt·Native/WASM/workspace Clippy·workspace build·suite 정책, Studio 2,789 PASS(2 skip), TypeScript·frontend 계약·Canvas suite 19개·호스트 수명 97조건이 통과했다. Visual Sweep 도구 검사 89개도 통과했다.

정확한 원격 code candidate `39a3774de60c81a3b803d158c044ddc195733eb6`의 [CI 원장](../assets/issue8946/stage10-20261114/code-ci.json)과 [실제 실행 수](../assets/issue8946/stage10-20261114/ci-archive-counts.json):

- [CI](https://github.com/edwardkim/rhwp/actions/runs/43487946323): success. A 3,796 + B 2,030 + C 2,225 + D 2,348 + E 2,899 = **13,298 PASS / 0 FAIL / 45 skipped**. 예상·실행 수 13,298 일치. Native Skia·frontend·lint·Build & Test 성공. 로컬 전체 suite 수와 CI 선택 수를 구분한다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43487946278), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43487946277), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43487946408), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43487946179): 모두 success.
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/43487941437): success. 수집한 40개 check에 pending·failure가 없다.

## 남은 merge 조건과 후속 기록

사용자는 self-review 갱신·push·최종 CI·병합까지 승인했다. 문서-only trailing head의 실제 fast-pass·required checks·mergeability를 확인한 뒤 정상 병합한다. 이 기록 시점에는 trailing CI와 병합은 실행 전이다. GitHub 자기 승인 review나 admin 우회로 조건을 대신하지 않는다.

양쪽 문서 기록을 보존한 실제 devel 통합과 최종 로컬 검증은 [단계 10](../../working/task_m100_8946_stage10.md)에 연결한다. 이전 단계 8·9의 CI와 검증은 과거 기록이며 현재 head의 CI로 대체 표기하지 않는다. 병합 후 실제 merge SHA·이슈 자동 종료·증적의 devel 반영을 확인하고 전용 산출물을 정리한다. GitHub 댓글은 별도 게시 승인이 없어 추가하지 않는다.
