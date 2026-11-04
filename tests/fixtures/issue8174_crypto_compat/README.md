# #8174 기존 암호화 자료 호환 대조군

T05 source `f7ede1d54eb018bea2f49405b3d1cba030e9360d`와 root lock의 chacha20 0.10.1에서
공개 `agent_seal`·`security_trailer` API로 생성했다. 실제 HWP 조판 fixture가 아닌 암호 컨테이너 계약이다.
호스트·평문·암호·키는 모두 manifest에 명시한 공개 시험 데이터다. PQ 키도 이 시험용으로 생성했다.
각 자료의 SHA-256과 생산 source는 `manifest.json`에 있다.

0.10.2 소비자가 기존 자료를 열고, 새 Native/WASM 자료를 기존 생산자가 다시 여는 검증에 사용한다.
오암호·키 불일치·AAD/암호문 변조는 거절하고 평문/가시층은 보존한다. 생성용 소스·정확한 실행은
`mydocs/working/task_m100_8174_t06_completion.md`의 증적에 연결한다.
