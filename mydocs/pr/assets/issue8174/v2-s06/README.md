# #8174 v2 S06 감사 증적

고정 source는 `695cea7b973fc54fc5ffc7f6d414f7ecf403b882`, upstream devel은
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다.
[실행 보고](../../../../working/task_m100_8174_v2_stage6.md)와 `validation.json`의 scope를 따른다.
이 자료는 계상 누락과 현재 격리 경계를 재현한 **감사 결과**다. 보정 완료·비신뢰 업로드의
공식 지원·실제 crates.io 게시 준비 완료의 증거가 아니다.

- `source-boundaries.json`: 실제 소비 경로 24곳의 파일·line·blob·해시와 관찰.
- `native-audit.json`: 직접 backend의 폰트/memo·clone, 요청 분리, 일반 supervisor 실행.
- `cli-audit.json`, `browser-audit.json`, `studio-audit.json`: 같은 font cap의 실제 제품 결과와 복구.
- `cli-environment.json`: 실제 CLI가 관련 없는 소유 환경 변수를 제거하는 실행 증거.
- `jpeg-summary.json`, `jpeg-native.json`, `jpeg-wasm.json`: 52개 입력과 실제 H 증가 예약·한도 결과.
- `fixtures/`: 실제 H 증가에 도달한 최소 baseline/progressive JPEG와 독립 `djpeg` RGB 참조.
- `dependency/`: 현재 선택된 feature, chacha20 0.10.2의 공식 checksum·source 변경 검토.
- `product-producers.json`: 재사용한 최적화 제품과 새 JPEG 감사 모듈의 구분·해시.
- `manifest.json`: 이 디렉터리의 증적 파일별 SHA-256. manifest 자신은 포함하지 않는다.

`probes/`는 실제 실행한 진단 소스다. `.rs.txt`·`Cargo.toml.txt`는 제품의 테스트 suite에 자동
등록하는 코드가 아니다. 당시 절대 workspace 경로와 ignored 경로를 그대로 보존했다.
다른 checkout에서 재현할 때는 복사본의 manifest 경로와 작업 디렉터리를 맞춘다.

1. ignored `output/pr-review/issue8174/v2-s06-20261025/` 아래에 probe 소스를 복사한다.
   `prepare_inputs.py`로 resource PDF를, `prepare_jpeg_probe.py`와 `prepare_mixed_jpeg.py`로
   JPEG 입력과 관측 사본을 생성한다. Pillow·`cjpeg`·`djpeg`가 필요하다.
2. `native-Cargo.toml.txt`·`native-main.rs.txt`를 `probe/Cargo.toml`·`probe/src/main.rs`로 복사하고
   root lock의 복사본에서 dependency 버전을 보존한다. `release-test`, 공용 `target/pr-review`,
   jobs 4로 빌드해 3개 폰트 입력으로 실행한다. `cli_audit.py`·`env_audit.py`는
   `product-producers.json`의 CLI snapshot을 사용했다.
3. JPEG probe의 `src/lib.rs`에는 **최종 `jpeg-lib.rs.txt`**를 넣고 `jpeg-main.rs.txt`를
   `src/main.rs`로 넣는다. 생성기의 관측 코드만으로는 최종 실행 harness가 완성되지 않는다.
   Native를 실행한 뒤 같은 shared target에서 `wasm-pack build --target web --release`와
   wasm-opt를 완료한다. `jpeg-browser.mjs`가 실제 Chromium에서 같은 52개를 실행한다.
4. `browser-audit.mjs`는 보존한 S04 제품 bundle과 공통 Worker를 사용했다.
   `studio-audit.mjs`는 소유한 Vite 7704 서버를 사용했다. 제품 코드 변경 뒤에는 이 과거
   bundle을 새 제품 head의 검증으로 사용하지 않는다.

resource PDF는 조판 기준 입력이 아니므로 생성기·해시를 보존하고 중간 PDF는 ignored output에
둔다. JPEG 참조의 RGB 일치는 최소 1×1 codec 경계의 검증이다. 전체 문서 시각 일치나 모든
혼합 progressive 지원으로 확대하지 않는다. 6개 원본 decoder 거절은 집계에 별도 기록했다.
