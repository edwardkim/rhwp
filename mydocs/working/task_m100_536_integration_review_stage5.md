# seo-rii 통합 검토 5단계 — 보정 후 출력과 정식 회귀 준비

## 코드 후보와 실제 전쪽 결과

renderer 후보 `d9a0b1c7919f3d7e5f5c6fef9e01c38472f7ce80`에서 Native를 다시 빌드하고
fresh WASM을 생성했다. 두 빌드는 각각 407.7초, 393.5초에 성공했다. 모든 Cargo는 공용
`target/pr-review`를 순차 재사용했다. Docker가 없어 WASM은 문서화된 host `--no-opt` 진단 경로다.
파일별 해시와 실행 명령은 `output/pr-review/seo-rii-20261017/`의
`corrected-native-build.json`, `corrected-wasm-build.json`에 있다.

정식 검토 입력의 전체 페이지를 다시 산출했다. 아래 각 입력은 전체 1쪽이며 누락·쪽수 차이는 없다.
고정 devel은 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`이다.

| 입력 key | devel Native | 보정 Native | devel WASM | 보정 WASM |
| --- | ---: | ---: | ---: | ---: |
| lseg-01-basic | 99.74540 | 99.74540 | 99.93175 | 99.93175 |
| lseg-02-mixed | 99.80580 | 99.80580 | 99.84066 | 99.84066 |
| pic-crop-01 | 99.98500 | 99.98500 | 99.74013 | 99.74013 |
| shape-group-02 | 90.04403 | 100.00000 | 100.00000 | 100.00000 |
| equation-lim-print2020 | 88.42676 | 90.53685 | 89.77376 | 91.90909 |
| form-hwpx | 99.88426 | 99.88426 | 99.87500 | 99.87500 |
| form-hwp-manual-crossformat | 99.88426 | 99.88426 | 99.87500 | 99.87500 |

TSV는 `corrected-native-scores/<key>/silhouette.tsv`,
`corrected-wasm-scores/<key>/silhouette.tsv`이며 입력·페이지·profile·DPI·글꼴은 이전 후보와 같다.
모든 값은 90% 이상이고 각 경로·쪽의 0.01%p 이상 하락이 없다. 평균으로 미달을 상쇄하지 않았다.
폼 PDF의 수동 출력 출처와 HWPX 대응 대조라는 제한은 [3단계](task_m100_536_integration_review_stage3.md)를 따른다.

## 실제 계약 검사와 직접 판독

이 후보의 format, Native Skia Clippy, TypeScript, frontend unit 2,310 PASS / 2 skipped를 확인했다.
Native FFI의 feature on Clippy·실제 공유 라이브러리 빌드 및 Python probe 9종도 새 라이브러리로
모두 통과했다. `corrected-ffi-enabled-validation.json`과 `corrected-ffi-artifact.json`에
실제 명령·source SHA·파일 해시를 연결했다. feature off, workspace/WASM lint, 전체 회귀와
Native Skia 공식 3종의 완료를 대신하지 않는다.

원본 극한은 Native review에서 이름의 HyhwpEQ 외형·크기 개선을 확인했다. 이름 외 분자/괄호의
기존 위치·모양 차이도 남아 있으므로 수식 엔진 전체 해결로 표현하지 않는다. fresh WASM 패널은
같은 head의 실제 PNG를 canonical 합성 함수로 만들고 capture manifest·PNG 해시를 연결했다.
각 출력의 확대 판독과 최종 기록은 정식 회귀 실행 결과와 함께 마무리한다.
새 기준은 [원본의 실제 MCP Print PDF](../../pdf/seo-rii-20261017/equation-lim-2020.pdf)로
보존했다. 10,578 bytes, SHA-256
`62b6b47da9c9dde5fb44ed4e025bbb871a048f01e9c7bf51f5e93be4a1a51aae`이며 원본 HWP는 기존 경로를 유지한다.

## 진단 입력의 별도 상태

11pt·16pt 단독 극한 대조 입력은 보정 Native 52.57069%·40.52670%로 낮다.
같은 입력의 고정 devel Native도 50.51151%·30.00000%였다. 이 둘은 기존 입력 생성·단독 극한 배치의
차이를 조사하기 위한 진단 자료이며, 정상 시각 fixture·완료 페이지로 등록하지 않는다.
한컴으로 HWP 저장한 뒤에도 script·fontSize·width·height·baseline·LineSeg가 보존됐고
저장 전후 독립 Print의 페이지 내용 stream 해시도 같다. 따라서 Save만 하면 cache가 정규화된다는
가정을 버린다. 두 문서의 배치 원인은 아직 확정하지 않았다. 진단 입력의 저조한 결과를
원본 `equation-lim.hwp`의 통과 수치로 바꾸거나 새 regression baseline에 고정하지 않는다.

## 정식 회귀의 범위와 남은 실행

`tests/cases/issue_536_equation_limit_name.rs`는 양쪽 90%를 확인한 원본
`samples/equation-lim.hwp` 1쪽만 사용한다. 독립 Print의 HyhwpEQ ASCII 이름과
Tf 110/92 em 비율, 이름의 scalar 순서·누락·중복 및 공통 기준선·전진 관계를 검사한다.
해당 쪽은 98% 미만이므로 절대 px 위치를 고정하지 않는다. 위 두 진단 입력은 사용하지 않는다.

검사는 정식 suite의 CLI를 호출하며 runtime `CARGO_BIN_EXE_rhwp`를 우선한다.
같은 검사 binary에 보정 전 CLI `456ac3585`와 보정 후 CLI `d9a0b1c79`를 각각 공급했다.
보정 전에는 실제 HyhwpEQ 이름 글꼴 assertion에서 exit 101, 보정 후에는 exit 0을 확인했다.
일반 nextest 경로에서도 **1 PASS / 0 FAIL**을 확인했다. 필터로 제외된 206건은 통과 수에 넣지 않는다.
실제 명령·검사 이름·두 CLI 해시는 `limit-name-red-green.json`에 있고, build/environment 실패를
수정 전 결함으로 세지 않았다. 기존 command golden의 기대값은 유지했다.

Frontend의 기존 host-font-provider 검사에는 첨자가 없는 극한의 이름만으로 실제 face를
요청하는 반례를 보완했다. 같은 새 검사와 보정 전 helper에서 요청 누락 assertion으로 실패했고
보정 후 helper에서 통과했다. 이는 바이트를 읽지 않는 font-request 메타데이터 계약이며,
합성 상자를 독립 Print와 일치한 실물 fixture로 표시하지 않는다.
실행 기록은 `host-font-limit-red-green.json`이다.

나머지 변경 case, 전체 nextest, 필수 lint·Native Skia·FFI off·최종 브라우저 및
개별 PR review 확정은 남아 있다. 새 PR·원격 CI·승인·통합은 아직 수행하지 않았다.
