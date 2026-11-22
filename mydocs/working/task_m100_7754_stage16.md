# 리베이스 후 실제 producer와 Windows 직접 PDF 보존 검증

Stage15의 호출 보정은 `8f9fd0414ffcfb720fa4a601c79c4590480de0e8`에 커밋했다.
새 base `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`와 이 후보의 독립 Native·최적화
fresh WASM producer를 기존 소유 worktree 두 곳에서 순서대로 빌드한다. Cargo target은
기본 작업 공간의 `target/pr-review` 하나를 재사용하며 동시 Cargo 실행은 하지 않는다.
생산 소스의 전후 SHA·바이너리·실제 deps rlib·WASM 네 파일의 SHA를 고정한다.

최소 글꼴 환경에서 집중 회귀·Native lib·기존 Native CI 검사·전체 Rust·lint를 재실행한다.
현재 작업 9개 이슈의 원본 전체 쪽, 기존 유입 기능, 편람과 정본 보호 조건은 새 producer로
재측정한다. 설치 글꼴 순서는 기존 TTF → 확인된 TTC → CFF 계약을 유지한다.

사용자의 후속 지시에 따라 #9316의 Windows 직접 PDF 기능 테스트는 범위에서 제외했다.
Windows 준비용 bundle·controller만 복사됐고 원격 worktree·빌드·측정은 생성되지 않았다.
해당 경로와 프로세스·파일 SHA를 확인한 뒤 작업 전용 폴더를 직접 삭제했고 기본 저장소의
head·clean 상태는 유지했다. 이 제외는 기존 Native 전체 회귀 실행 범위를 바꾸지 않는다.

완료된 구 head의 편람 383쪽과 시행령 74쪽 네 조건, rebase·range-diff·사용 종료 증분 캐시
정리 근거를 `mydocs/pr/assets/issues7754_7765/source46/`에 보존했다. 구 결과를 새 head의
검증으로 대체하지 않는다. #9293은 병합 없이 닫혀 있어 그 코드는 포함하지 않았고 PR
최신 사용자는 다시 PR 준비까지만 완료하고 대기하도록 지시했다. 원격 push·PR 생성·병합·
후속 처리는 보류하고 새 head의 필수 검증과 로컬 제출 준비를 완료한다. 현재 새 검증은 진행 중이며 완료로 표시하지 않는다.

독립 중고딕 전 글자표의 원 윤곽 공급으로 WASM 보호 문서 89쪽의 99.94% 차이가
100.00%로 개선됐다. 원본 SPJGT의 cmap 결측과 다른 family 대체 윤곽의 차이를 확인한
자료는 `mydocs/pr/assets/issues7754_7765/source47/`에 보존했다. 다른 정본 미달 쪽이
남아 있으므로 전쪽 보호 완료로 판정하지 않고 신명조 글자표의 독립 인쇄도 재시도한다.
리베이스 전 Source37 진단 결과와 새 Source88 검증을 구분한다.

## 후보 producer의 캐시 재사용 거부와 재검증

Source88의 후보 빌드가 base와 다른 52개 생산 파일을 컴파일하지 않고 같은 Native·WASM
바이트를 재사용한 것을 SHA 대조로 확인했다. 해당 후보 측정과 Source91/101/102/103은
완료 증거에서 제외하고 작업 소유 프로세스만 종료했다. 실제 base 수집은 계속 보존한다.
`mydocs/pr/assets/issues7754_7765/source48/`에 재사용·종료·정리 근거를 기록했다.

Source105는 소스 바이트를 유지하면서 생산 파일의 수정 시각을 갱신하고 실제 재컴파일과
서로 다른 최종 artifact SHA를 검증한다. 원래 target·일반 최적화·incremental 정책은
유지한다. Source107의 작은 gate에 base에서 미달하는 실제 HWPX 경계 4쪽을 추가했다.
그 뒤 Source106의 전체 회귀·lint, Source108의 필수 편람, Source109의 전쪽 보호,
Source110의 9개 이슈·유입 기능 전쪽 비교로 새 검증을 완료한다. 현재 단계는 미완료다.

## 실제 Source105의 완료된 원 글꼴 검증

실제 재컴파일한 Native·최적화 fresh WASM과 작은 경계 8개씩을 통과했다. 화학물질
103쪽은 양쪽 전쪽 100.00%, 개요 9쪽은 새 base·후보 양쪽 backend의 4조건 전쪽
100.00%다. 원본 자격이 없는 같은 이름의 TTF와 검증된 TTC를 실제 glyph 프로그램으로
구분했다. 383쪽 필수 Native 편람도 통과했다. 완성 증거는
`mydocs/pr/assets/issues7754_7765/source49/`에 보존했다.

현재 전체 Native 원본 583쪽은 전쪽 98% 이상을 확인했다. 전체 Rust는 최종 exit를
기다리며, 763쪽 보호 문서의 일부 TSV 쓰기는 디스크 부족 후 같은 조건·checkpoint에서
재개한다. 사용 종료 producer·중복 사본만 SHA와 사용 여부를 확인해 정리했다.
Studio 14개 여정 중 후보 호스트 입력의 준비 호출 수가 21/20으로 달라 원 호출 stack을
대조하고 있으며, 그 미해결 상태를 준비 완료나 무회귀로 표시하지 않는다.
