# #7754~#7765 단계3: 전체 검증과 최신 devel 기능 보존

## 시작점과 수용 조건

단계2의 구현과 회귀 복원을 `c7e28cee7852b8643d65831decef39a6cae23a28`에 고정했다. 생산 코드는 `2f46647df16da00365b22a1364cdc0b28b8344bc`다. Native와 최적화 fresh WASM의 원본별 모든 쪽98% 이상, 가능한 쪽100%를 유지한다. 원본 TTF → 일치하는 TTC face → OTF(CFF) 순서를 바꾸지 않는다.

## 완료한 고정 소스 검증

소스32의 Native Skia공식3종은 root lib3853 PASS/12 ignored, 내부 lib182 PASS, placeholder2 PASS, 직접 PDF36 PASS다. fmt와 workspace all-targets Clippy도 exit0이다. 최소 글꼴 suite·unit 정책 확인과 Node36개가 통과했다. 같은 fresh WASM의 Canvas 글꼴26개, Studio 별칭·Canvas 비교·CanvasKit 준비도 통과했다. 실제 명령·원본 로그는 `mydocs/pr/assets/issues7754_7765/source32/gates/`에 보존했다.

HWPX 편람 Native383쪽은 최저99.97%, 표시100.00%382쪽, 정본 하락0쪽이다. 기여자 공통4문서·두 경로8조건은 모두100.00%이며 직접 review를 판독했다. 전체 Rust nextest와 나머지 전쪽 검증은 실제 종료를 기다린다.

## 최신 devel 유입과 검증 계획

2026-11-22 KST에 `upstream/devel`이 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`에서 `5665e538292a0686a912f6a4d8950c55be89929a`로 이동했다. #9292의 편집17건 통합과 #9297의CI 요약 변경이다. 현재 후보와 최신 devel의 병합 시뮬레이션은 충돌이 없다. 기존 결과를 새로운 생산물의 결과로 바꾸어 부르지 않는다.

진행 중인 고정 소스 Cargo 종료 증거를 보존하면서 격리된 작업 트리에서 rebase를 준비한다. 새 base/head의 실제 출력, 원본별 주요 경계와 유입 PR 기능을 비교하고 원본별 전쪽 Native/fresh WASM, 필수 lint·회귀·편람·정본 보호를 새 생산물에서 완료한다. 필수 미달은 구현을 보정한 뒤 재검증한다.

기존 본인 열린 #9293은 다른 작업의 브랜치다. 읽기 전용 CI 조사에서 합성 테스트 글꼴 `tests/fixtures/hft_mapped_symbol_program_tracking/fallback.ttf`의 라이선스 원장 등록 누락으로 Frontend package gate가 실패했다. 임의로 그 브랜치를 덮어쓰지 않는다. 본 후보와 #9293의 병합 시뮬레이션은 공통 원장4파일이 충돌하므로 새 PR 생성 전 해결이 필요하다. 실제 진단과 당시 head를 출력 기록에 보존한다.
