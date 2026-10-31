# #8415 암호 인증 검증 증적

공개 합성 입력은 [PDF 제어군](../../../../../pdf/issue8415/authentication/README.md),
계약과 실행 결과는 [공통 작업 기록](../../../task_m100_8413_implementation.md)에 있다.

- `native.tsv`, `wasm.tsv`: 독립 평문 PDF와 암호 인증 후 HWPX의 1쪽 96dpi 실루엣. 양쪽 raw 100.00%.
- `reference.png`, `native.png`, `wasm.png`: 같은 페이지의 직접 판독 대상. 본문과 빨간 그림을 확인했다.
- `password-required.png`, `password-invalid.png`: 암호 요청·오입력 안내와 입력 포커스.

글꼴은 Helvetica → Arial로 명시했다. 외부 글꼴 파일은 포함하지 않는다.

| 식별 대상 | SHA-256 |
| --- | --- |
| 평문 기준 PDF | `5f9356ae1ef7f84ad39fa233b9223e5af5380230789b34fdb0b8472041d7c12a` |
| 사용한 Arial 글꼴 | `b3658eadae55e682b5f69eb64c439c1ecc8f196c0bb8d4756d145d13bc86476a` |
| 최적화 WASM | `cf203025645070b658e79d048503df3030a47d5f587660cde022bbdfedff9c1f` |

실행 로그·중간 JSON은 Git에서 제외된 `output/pdf-import-errors-20261031/`에 유지한다.
테스트 전용 암호만 사용했으며 실제 사용자 암호나 사용자 문서를 포함하지 않는다.
이 1쪽 자료로 사용자 원본 127쪽의 시각 검증을 완료했다고 판정하지 않는다.

최종 재측정일: 2026-11-01T00:29:10+09:00. 추가 바이트를 포함한 `password-padded-fields.pdf`의 저장본을 대조했다.

## 재현 경로

Rust 공개 API·실제 CLI 회귀는 `issue_7911_pdf_frontend`와 `issue_7911_pdf_cli`를 기존
suite runner로 실행한다. Studio 검사는 다음 제품 E2E를 사용한다.

```bash
cd rhwp-studio
node e2e/pdf-password-issue8415.test.mjs --mode=headless
node e2e/pdf-import-dialog-issue8411.test.mjs --mode=headless
node e2e/pdf-import-busy-issue8398.test.mjs --mode=headless
```

시각 검사는 `password-padded-fields.pdf`를 `import-pdf`의 `--password-stdin`으로 HWPX에
가져온 뒤 원본 `password-control.pdf`와 비교한다. `visual_sweep.py --silhouette-only`의
`--compat 2024 --dpi 96`을 사용하며, WASM에는 현재 최적화 빌드의 `--wasm-pkg pkg`를
추가한다. `--font-path`로 Arial 파일을 명시하고 `--font-environment`의 substitutions를
`{"Helvetica":"Arial"}`로 지정한다. Arial이 없는 환경의 결과를 이 증적과 같은 조건이라고
간주하지 않는다. 명령의 입력·출력과 임시 설정은 ignored `output/` 아래에 둔다.
