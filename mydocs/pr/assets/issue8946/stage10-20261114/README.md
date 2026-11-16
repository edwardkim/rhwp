# #8946 단계 10 — 실제 devel 통합과 최종 producer 검증

검증 source는 `a6cdfdce4cc7d360df6ee1ef8d9ea978340d1b04`, 기준 devel은 `dd7f453898776c262c00e862e10f47d22a4d6ceb`다. 이전 증적 head `7b1d8e6f6af5aa2a706f37b0883255dc33ccd5e2`와 devel의 복수 공통 조상 때문에 GitHub가 충돌로 표시했다. 양쪽 기록을 보존한 `ort` 통합의 tree는 `f27d5d673afef08a2c2e7ceddc5b65be01358c59`다. source·회귀 기대값을 추가 보정하지 않았다. #8946의 명시적 Canvas2D 호스트 프로그램 선택·환경 갱신·수명 범위를 유지한다. 이 기록은 로컬 검증이며 원격 CI·병합 완료를 뜻하지 않는다.

## 전쪽·기능 보존

| 입력 / backend | 페이지 | devel 최저 raw | 후보 최저 raw |
| --- | ---: | ---: | ---: |
| HWPX / Native | 763 | 85.45580% | 96.34886% |
| HWPX / WASM | 763 | 85.44688% | 99.56218% |
| HWP3 / Native | 763 | 98.77212% | 98.77212% |
| HWP3 / WASM | 763 | 99.54853% | 99.54853% |

[전쪽 대조](last-devel-full-comparison.json)에서 모든 후보 쪽이 90% 이상이고 독립 Print 쪽수가 같다. 최대 하락은 0%p다. [정본 보호](protected-comparison.json)는 18개 입력·양쪽 3,542쪽의 보호값 하락 0, [편람 Native 383쪽 gate](handbook-gate.json)는 통과했다. 입력·독립 PDF 190개의 실제 바이트는 [commit 감사](input-commit-audit.json)에 연결한다. 763쪽 HWPX Print의 대용량 로컬 보관 계약은 [단계 9](../stage9-20261113/local-reference-preservation.json)를 유지한다.

[기능 보존](last-incoming-comparison.json)은 148개 조합·946쪽의 PNG·배치 트리가 devel과 모두 같다. 이전 56개 입력과 CellBreak·footer·Flutter 공통 입력에 #9030·#9034·#9036·#9041·#9018 원본/편집본을 보강했다. [연결 표지](linked-comparison.json) 8개 조합·8쪽은 모두 raw 100%, 배치 트리 동일이다. 이 중 굵은체·그림 혼합 WASM 2쪽은 하단 기관명의 RGB 래스터가 달랐다. 독립 Print와 3배 확대 대조한 [차이 기록](linked-pixel-differences.json)에서 위치·내용·자간은 유지되고 RGB 오차는 감소했다. 이를 RGB 동일이나 #8957 전체 해결로 주장하지 않는다.

[직접 판독](direct-reading.json)의 32쪽은 독립 Print와 같은 96 DPI에서 확인했다. 세금 서식의 머리말·글꼴 농도, 교육 개요의 제목 굵기·사각 글머리·차트 색상은 devel과 같은 기존 잔차로 구분한다. 두 원본의 [기존 후보 대비 3,052쪽 PNG 동일성](previous-head-output-parity.json)에 따라 단계 9의 표지·문장부호 확대 판독과 대표 PNG를 재사용했다. 재사용을 새 고해상도 캡처로 표시하지 않는다.

[기본 9조건](default-comparison.json)은 각 763쪽의 출력과 전쪽 해시가 기준과 같다. [초기 합성 화면](last-initial-screen-comparison.json), [실제 호스트 두 원본 전쪽](host-rebase-parity.json), [편집·저장·재열기](edit-verification.json)를 확인했다. 실제 Host 전쪽 최저값은 동일 출력의 이전 측정 HWPX 99.35730%, HWP3 99.35495%를 연결한다. 첫 화면은 세 외부 그림이 decode된 합성 화면으로 측정했다.

[Studio](studio-comparison.json)는 실제 textarea ClipboardEvent의 6조건·각 source 24회 저장/재열기를 확인했다. [ByPath](bypath-head.json), [새 WASM 계약](new-wasm-comparison.json), 최종 Native의 [실제 CLI](api/rebuilt.json)와 [JSON 속성·중복 키 거부](pr9052-cli-rebuilt-results.json)도 확인했다. [여권 브라우저 계약](passport-contract-comparison.json)은 원 승인 표본과 같은 2,835 HU 그림·InFrontOfText 조건에서 양쪽 모두 2쪽, 본문 보존, 반복 저장 바이트 동일이다. 앞서 7,500 HU 흐름 그림으로 달라진 쪽수를 원 계약의 회귀로 해석했던 진단은 [정정 기록](passport-probe-correction.json)으로 분리했다.

## producer와 최초 실패 정정

[producer](producer.json)의 최종 Native는 `6a4fc598…`다. 첫 Native `8e9db6dd…`는 공유 target의 오래된 chart 의존성으로 값축 하한이 잘못되어 판정에서 제외했다. 추적 소스 2,381개의 바이트를 유지하면서 mtime을 갱신해 재컴파일했고, 차트와 모든 Native 검증을 다시 실행했다. 원본 차트는 82.94366%에서 기준과 같은 95.88988%로 복원됐다. [캐시 진단](native-cache-diagnosis.json)과 [소스 감사](rebuild-source-audit.json)를 보존한다.

전쪽·브라우저에 실제 사용한 최적화 WASM은 `4737807f…`다. 강제 의존성 재빌드 `06458eda…`와의 차이는 [경로·명령 감사](wasm-source-path-diagnosis.json)에 별도로 기록했다. base와 head의 vendor/crates 소스는 같으며, 22개 진단 소스 경로가 worktree 경로로 바뀌었다. opcode·분기·호출 구조 변화는 없고 숫자 주소/피연산자가 달라졌다. [9개 실제 출력 대조](wasm-rebuild-output-parity.json)도 PNG·배치 트리 동일이다. 새 해시를 기존 전쪽 검증의 producer로 바꾸어 표시하지 않는다.

초기 회귀는 잘못 고른 최소 글꼴 profile이 공개 DejaVu·Happiness fixture와 테스트 생성 임시 글꼴 읽기를 막아 실패했다. 실제 이전 통과 실행의 stage6 profile로 복원했고 개인·상용 글꼴 차단을 [감사](font-isolation-audit.json)했다. 제품·회귀 기대값을 바꾸지 않았다. 최초 실패와 최종 재실행의 로그 SHA는 [실행 원장](checks.json)에 구분한다.

## 최종 로컬 게이트

최소 글꼴 전체 nextest는 **13,483 PASS · 0 FAIL · 45 SKIP**, Native Skia library는 **4,034 PASS · 12 ignore**다. placeholder 2·직접 PDF 36·HFT 집중 22개도 통과했다. fmt·Native/WASM/workspace Clippy·workspace build·suite/unit-tier 검사, Studio 2,789 PASS(2 skip)·TypeScript·frontend binding·E2E manifest를 통과했다. 실제 Canvas suite 19개와 [호스트 환경·수명 97조건](host-lifecycle-e2e.json)이 통과했으며, Visual Sweep 도구 검사 89개도 통과했다. [브라우저 수신 자원](actual-browser-runtime.json)과 실제 Studio WASM 수신 영수증은 사용한 producer 해시와 일치한다.

## 재현과 보존

Native는 locked release-test/native-skia, WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 최적화 산출물이다. 전쪽은 `scripts/visual_sweep.py --silhouette-only`의 동일 원본·독립 PDF·96 DPI·compat·실제 font SHA를 사용했다. [출처 원장](visual-provenance.json)에 실제 산출물과 입력을 연결한다. 독립 PDF 래스터 캐시는 단계 9의 검증된 래퍼를 사용하고 제품 출력은 새 producer로 생성했다. TSV 원본은 바이트 그대로 보존했다. 문서용 비교 집계 TSV의 줄 끝만 LF로 정규화했다. 사용자 글꼴, PDF에서 준비한 글꼴, 실행 바이너리와 인증 자료는 커밋하지 않는다.
