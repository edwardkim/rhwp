# v0.9.0 배포 최종 증적

[배포 결과 보고서](../../../report/release_0_9_0_execution_report.md)의 근거를 보존한다.
`delivery-evidence.json`은 실제 배포·공개 소비 검증이 끝난 뒤 정식 기록으로 선별한 결과이며,
`cleanup.json`은 2026-11-08 종료 정리의 실제 실행 기록이다.

| 파일·키 | 확인할 내용 |
| --- | --- |
| [delivery-evidence.json](delivery-evidence.json) → `crates_packages.packages` | 14개 패키지의 버전·source SHA·공개 archive 크기·SHA-256 |
| `host_public_consumer`, `host_cli`, `host_wasm_browser` | 공개 registry의 Rust·CLI·WASM 빌드와 실제 브라우저 소비 결과 |
| `docs_rs` | 지정 버전의 실제 문서 빌드 14건과 조회 시각 |
| `docker_consumption` | 32개 명령·추가 한글 폰트 검사, 명시/자동 탐색 PNG 해시 |
| `npm_public_pipeline`, `npm_public_browser`, `npm_provenance` | npm 공개 패키지 실행·서명 확인·호출 workflow와 source |
| `binary_*`, `pages_*`, `vscode_public`, `open_vsx_public`, `firefox_public` | 채널별 공개 배포물·CI artifact 대조와 해당 실행 범위 |
| `contributors`, `main_merge_exception`, `issue8745_flow_comparison` | 기여자 수 대조, 승인된 main 승격 예외, 보안 이슈 종료 근거 |
| `tag_ci`, `release_binary` | 태그 CI 미완료와 실제 Release Binary 성공을 구분한 상태 |
| [cleanup.json](cleanup.json) | 삭제 범위·확보 공간·소스 ref·보존한 로컬 환경의 파일 해시 |
| [closeout-review.json](closeout-review.json) | #8793 제출 head CI·실제 fast-pass와 2026-11-09 스토어 조회; trailing head CI는 병합 전에 별도 확인 |

개별 관측 시각을 그대로 보존한다. 후속 문서 PR을 작성하면서 모든 채널의 실행을 반복한
기록은 아니다. `source_record_hashes`와 명령 인자의 `output/` 경로는 검증 당시 임시 원본의
식별 정보이며, 종료 정리 뒤 존재하거나 재실행할 수 있는 경로를 뜻하지 않는다.
이 파일들은 중간 산출물 백업이 아니며 소스·공개 패키지·검증 결과를 연결하는 최종 원장이다.

Docker 환경·입력·라이선스가 있는 Noto Sans KR·상세 로그·재실행 매뉴얼은 사용자 지시에 따라
Git에서 제외된 `local/crates-io-consumption/`에 유지한다. 이 로컬 환경이나 폰트 파일은 이 PR에
포함하지 않는다. 32개 명령은 해당 폴더의 `VALIDATION-0.9.0.md`가 기록한 bootstrap 실행 결과이며,
후속 재사용 runner의 전체 실행 결과로 바꾸어 해석하지 않는다.
