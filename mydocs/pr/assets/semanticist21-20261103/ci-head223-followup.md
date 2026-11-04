# 제출 head 223eb2361c의 실제 CI 보정

## Native CLI 계약의 누락 확인

Native CI의 `cli_exit_codes_native::png_json_metadata_and_render_preserve_feature_and_provenance_contracts`가 `untrustedContent=true`, 이전 기대값 false로 실패했다. 원 #4132 파일 게이트(`1ed97ba991`)와 #7903의 검사 추가(`116b2b6ae0`, 2026-10-14T22:12:10+09:00), #3787 실제 값 표지 계약과 #8565의 글꼴 보고를 대조했다. PNG metadata-only와 실제 render의 글꼴 보고는 문서 파생 값이므로 false 고정이 잘못됐다. 기존 종료 코드·feature·schema·쪽수·파일 생성 검사는 유지하고, 실제 실린 네 종류 글꼴 보고의 출처 경로와 true 표지를 검사한다. MAP을 검사에서 재인용하지 않으며 경로 순서를 계약으로 고정하지 않는다.

이 Native CLI target을 앞의 로컬 Native 3종 목록에서 빠뜨렸다. 원 검사의 FAIL을 확인했고, 이후 `.github/workflows/ci.yml`의 release-test Native 실행 목록 전체를 로컬에서 검증한다. Native 기존 라이브러리 4,040건·기본 출처 11건의 통과가 이 실행을 대신하지 않는다.

## CodeQL의 세 좌표 진단

새 Rust 분석 `2093009475`(tested merge b0165b43aa4f746bc4dc8fa0ea4d14d6ad8d4598)는 총 129개 결과이며 PR scope open alert 15개의 ID는 H4 분석과 같다. check는 이 중 이번 diff의 세 sink를 high로 판정했다. 새 alert ID가 없다는 이유로 성공이나 false positive로 분류하지 않는다.

| alert | 제출 head sink | code flow / 단계 합 | 실제 출력 |
| --- | --- | ---: | --- |
| #493 | paragraph_layout.rs:3672 | 4 / 274 | 인라인 표 문단 원점·너비·간격 |
| #494 | paragraph_layout.rs:6991 | 4 / 334 | 본문 overflow 좌표 |
| #496 | paragraph_layout.rs:11739 | 4 / 310 | raw fallback의 overflow 좌표 |

모두 NativeSession → from_bytes_with_password에서 열린 문서의 수치 배치가 진단 stderr로 흐른다. 기존 #8517 원 보정 `57ce496ef7`과 이번 앞선 보정 `3af2224e4e`의 정책을 적용한다. 인라인 표의 주 진단과 같은 블록의 저장 줄·표 진단, 본문/raw fallback의 overflow 출력만 제거한다. 계산·실제 y/높이·분할·배치·cell overflow 집계·paint·저장은 유지한다. 로그 전용 순수 조건·수치 계산만 함께 제거한다. 보안 query·언어·경로·분류·workflow·검증 기준은 바꾸지 않는다.

기존 실제 암호 HWPX CLI 회귀에 RHWP_LAYOUT_DEBUG와 제거한 채널을 추가한다. 정상 SVG 산출은 유지하며 보정 전/후를 실제 binary로 확인한다. 시각 증거는 조판 계산 동일성을 확인해 이미 검증한 Native/fresh WASM producer와 구분해 재사용한다.

## 검증 상태

현재 로컬 보정 중이며 아직 새 head를 push하지 않았다. 정확한 Native CI 목록·Rust lint·workspace·stock 회귀와 보호 문서 실제 CLI 검증을 완료한 뒤 같은 PR의 새 head를 제출한다. 원 PR의 기여 commit·CI 시간 예산·기존 검사를 유지하며, 기존 223 head의 다른 CI도 완료까지 관찰한다.
