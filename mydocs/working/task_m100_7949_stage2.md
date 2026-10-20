# Task #7949·#7950 — 2단계 PDF 기본 feature 전환

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 이전 구현: 마스크 보정 `2a1243d7d`; [Native 선행 결과](task_m100_7949_stage1.md)
- 상태: 기본 Native 빌드·실제 PDF 열기 통과. 최종 Docker WASM·명시적 off·정식 회귀·시각 gate 확인 중

`Cargo.toml` 기본 feature에 `pdf-import`를 추가했다. 실제 cfg를 조회하는 CLI/WASM capability는
유지하고 도움말·미지원 빌드의 오류·장기 매뉴얼·공용 Worker 모듈 안내를 새 기본값에 맞췄다.
WASM 계약 검사의 off bundle 입력은 `PDF_IMPORT_DISABLED_PKG`로 구분했다.

Docker `wasm` 서비스와 wrapper, CI·npm-publish·deploy-pages의 WASM 빌드와
release-binary의 `--features native-skia` 빌드에는 기본 feature를 끄는 옵션이 없었다.
해당 소비자는 기본 feature를 상속하므로 별도 workflow 변경은 하지 않았다.
패키지의 PDF UI 채택과 실제 npm/스토어 배포 완료를 주장하지 않는다.

- 추가 PDF feature 없이 `cargo build --locked --profile release-test --bin rhwp --features native-skia`,
  공용 target·jobs14: **성공**, 2분31초.
- 그 바이너리의 `info /mnt/e/tmp/261016-mof-69009.pdf --json`: format `pdf`, **1쪽**.
  stderr의 `LAYOUT_TABLE_OVERLAP` 진단은 보존하며 전체 출력 판정과 구분한다.
- Cargo metadata에서 default가 `console_error_panic_hook`·`pdf-import`임을 확인했다.
- 공용 Worker 세션 단위 검사와 변경한 WASM 계약 스크립트의 JS 구문 검사 통과.
- 변경한 매뉴얼·공용 모듈 4문서의 내부 링크 검사와 `git diff --check` 통과.

마스크 진단 PDF 5입력의 실제 저장 PNG alpha/색상 검사는 통과했다.
재현 PDF의 별도 정확한 글꼴 공급 대조에서도 Native 실루엣은 **82.7978%**였으며,
배경 채움·글꼴/간격의 남은 차이는 해소되지 않았다. 전체 시각 gate는 미충족으로 유지한다.
로그·TSV·입력과 실행 증거는 ignored `output/pr-review/issue7949/`에 보존한다.

다음은 표준 Docker에서 **추가 feature 지정 없는 기본 WASM**과 **명시적 off WASM**을
생성해 실제 UI·API·Worker를 확인하는 통합 단계다. 기존 7700 서버와 사용자 문서는 보존한다.
아직 PR 준비 완료·원격 반영·이슈 종료 상태가 아니다.
