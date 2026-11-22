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
최신 사용자는 준비 완료 후 PR 생성·병합·후속 처리까지 승인했다. 새 head의 필수 검증과
보호 조건을 완료한 뒤 Open PR을 제출한다. 현재 새 검증은 진행 중이며 완료로 표시하지 않는다.
