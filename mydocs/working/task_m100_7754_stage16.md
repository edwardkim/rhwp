# 리베이스 후 실제 producer와 Windows 직접 PDF 보존 검증

Stage15의 호출 보정은 `8f9fd0414ffcfb720fa4a601c79c4590480de0e8`에 커밋했다.
새 base `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`와 이 후보의 독립 Native·최적화
fresh WASM producer를 기존 소유 worktree 두 곳에서 순서대로 빌드한다. Cargo target은
기본 작업 공간의 `target/pr-review` 하나를 재사용하며 동시 Cargo 실행은 하지 않는다.
생산 소스의 전후 SHA·바이너리·실제 deps rlib·WASM 네 파일의 SHA를 고정한다.

최소 글꼴 환경에서 집중 회귀·Native lib·기존 Native CI 검사·전체 Rust·lint를 재실행한다.
현재 작업 9개 이슈의 원본 전체 쪽, 기존 유입 기능, 편람과 정본 보호 조건은 새 producer로
재측정한다. 설치 글꼴 순서는 기존 TTF → 확인된 TTC → CFF 계약을 유지한다.

#9316의 원 이슈 #9276 보존은 win10-ted에서 공고문 11쪽·안내문 7쪽·서식 1쪽의 직접
PDF와 PNG를 새 base·후보로 비교한다. 기본 Windows 저장소는 clean 상태의
`197ce19e99438c2cba452c654ae1af504dab85ec`이며 이를 checkout하거나 수정하지 않는다.
Git bundle로 로컬 후보를 전달하고 소유한 sparse worktree만 만든다. Visual Studio 2022의
실제 MSVC link를 선택하며 Cygwin의 동명 link는 빌드에 쓰지 않는다. Windows 결과도
쪽수·텍스트·픽셀과 글꼴 임베딩을 확인한 뒤 기능 보존 여부를 판정한다.

완료된 구 head의 편람 383쪽과 시행령 74쪽 네 조건, rebase·range-diff·사용 종료 증분 캐시
정리 근거를 `mydocs/pr/assets/issues7754_7765/source46/`에 보존했다. 구 결과를 새 head의
검증으로 대체하지 않는다. #9293은 병합 없이 닫혀 있어 그 코드는 포함하지 않았고 PR
제출 보류 지시는 유지한다. 현재 단계의 새 검증은 진행 중이며 완료로 표시하지 않는다.
