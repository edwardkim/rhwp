# Bongseop-Kim PR #9043·#9048 통합 검증

- 기준 devel: `5cbcc9b451cf286e1b7c680426c857dc802e50f3`.
- 실제 검증 code head: `89701eb34af54502e94afa6319240596e98a1367`.
- [#9043 개별 검토](../../archives/pr_9043_review.md) · [#9048 개별 검토](../../archives/pr_9048_review.md).
- [집중 28건과 최종 전체 실행 내 동일 28건](focused-results.json).
- [전체 회귀](full-nextest-result.json): 최소 글꼴 13,455 PASS/0 FAIL/45 skip. 최종 정본 suite 실행 323.601초.
- [로컬 gate](local-validation.json): fmt, Native/WASM32 Clippy, workspace build, all-target Clippy, manifest, fresh WASM PASS. 같은 head의 이미 통과한 native/WASM32 Clippy와 workspace build는 기록에 구분했다.
- [환경](environment.json): macOS 26.6.2 arm64, CoreText 528 faces·개인 글꼴 경로 0, system_profiler 237개 font 항목. 최소 글꼴 회귀와 일반 Studio 화면 검증을 구분했다.
- [WASM 공개 API](wasm-api-results.json) · [실제 Studio](studio-results.json) · [producer·이미지 해시](producer.json).
- [devel baseline package 재사용 근거](base-producer-qualification.json): 기존 code producer에서 이 base까지 src/crates/Cargo/scripts 차이 0, JS/WASM 해시 일치. 현재 후보의 fresh WASM과 혼동하지 않는다.
- [소비 경로·영향 범위](scope-audit.json) · [기존 손실 보고 fixture 검토](native-save-fixture-audit.json) · [검증 도우미 정정 이력](local-validation-corrections.json).

## 각 PR의 실제 주장과 증거

| PR | 계약 | Native | fresh WASM·Studio |
| --- | --- | --- | --- |
| #9043 | 삭제한 그림의 HWPX BinData 잔존 제거, 남은 참조·원본 미참조 자원·Undo/클립보드 보존 | 원 5개 + 추가 2개 저장 검사, HWP/HWPX 두 차례 저장·재열기 및 관련 대조군 통과 | 삭제 후 저장·재저장 0개; 원본 미참조 2개 payload 보존; 로드한 그림 글머리표의 자기 바이트 보존. 실제 Studio도 삭제 후 두 번 저장해 0개 |
| #9048 | 누름틀 없는 본문 빈 문단의 캐럿이 문단 정렬을 따름 | 원 2개 관계 검사와 기존 ClickHere 대조군 통과 | 113.4 / 396.9 / 680.3; 첫 글자 앞뒤 구간에 빈 캐럿 포함, 실제 DOM 좌표·화면 확인 |

입력은 공개 API가 만드는 빈 문서와 커밋된 테스트의 69바이트 빨강/파랑 PNG, 기존 `samples/issue7816/odd_spacing_prev.hwpx`다. 해당 원본의 SHA는 scope-audit에 기록했다. 생성 저장본은 runtime 산출물이며 ZIP·manifest 참조·payload SHA를 결과에 남겼다. 비공개 문서·새 원본·기준 PDF를 사용하지 않았다.

이는 자원 보존과 인쇄되지 않는 편집 캐럿 질의의 검증이다. 원래 레이아웃 앵커와 live IR은 바꾸지 않으며, 빈 페이지의 PDF 일치율을 개선 수치로 만들지 않았다. 조판 전쪽 Visual Sweep·편람 gate·정본 보호 전쪽 재실행은 비해당이다. 화면 증적은 실제 raw Studio screenshot과 직접 screenshot crop이며 합성 캐럿 그림이 아니다.

Mac fresh WASM은 사용자가 지정한 `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 또는 wasm-opt 최적화 성공으로 보고하지 않는다. root pkg와 Studio public의 JS/WASM 실행 파일 해시를 확인했고 실제 Vite 초기화 응답의 JS·WASM을 별도 기록했다. Vite가 변환한 JS 응답은 raw JS binding과 구분한다.

`.log`, 임시 Python/Node driver, runtime 저장 ZIP, Chrome profile, package 바이너리는 ignored output 또는 로컬 빌드 경로에만 둔다. 최종 증거만 이 디렉터리에 보존한다. 원 PR fork branch는 그대로 두고 통합 PR merge 후 원 PR과 #8941·#8944를 후속 처리한다.

## 통합 PR CI

[PR #9054](https://github.com/edwardkim/rhwp/pull/9054)의 exact code head `0ae35de66d950bc58fa6c8320b0d20e944c691d4`에서 필요한 CI가 모두 성공했다. [정식 영수증](ci9054-code-complete.json): Archive A~E 합계 13,270 PASS / 0 FAIL / 45 skip, 전체 check는 29 SUCCESS / 7 SKIPPED / 1 NEUTRAL이다. `selective / rfp=0 / rust=1 / fe=none / render=0 / skia=0 / ql=rs` 실행으로, 모든 영역 Full 실행을 뜻하지 않는다. 로컬 workspace 13,455건과 별도로 기록한다.

이번 후행 기록은 mydocs 문서·증적만 바꾸며 제품·검사·CI 정의를 바꾸지 않는다. 최신 trailing head의 checks·병합 가능 상태와 실제 병합 결과는 후속 단계에서 확인한다.
