# #8946 최신 devel 검증 증적

코드 `b4855616375c4a8fd1cdd6083d85cd77d655ad26`, 기준 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`.
[작업 기록](../../../../working/task_m100_8946_stage6.md)의 완료 결과와 남은 조건을 함께 확인한다.

**제출 준비 미완료**: 정본 보호 18입력·3,542쪽의 전 조건 검증이 남았다. 이 폴더가 존재한다는 사실을 PR 제출 가능 판정으로 해석하지 않는다.

- `local-verification-results.json`: 최종 로컬 검사, 로그 SHA, 97개 호스트 계약.
- `producer.json`: 실제 Native·최적화 fresh WASM 및 소스 object 신원.
- `*-full.tsv`: HWPX/HWP3의 전 763쪽, 후보와 base 및 실제 Host 측정.
- `devel-full-comparison.{json,tsv}`: 동일 조건 3,052쪽의 정밀 raw Decimal 하락 비교.
- `comparison-summary.json`: 기본 모드 9조건, 이전 #8946 출력 및 incoming 기능 보존.
- `incoming-index.json`, `incoming/`, `rowbreak/`: 새 devel 기능의 조건·입력/PDF·전체 쪽수·TSV.
- `reviews/`, `incoming-reviews/`: 독립 Print와 후보의 review·standalone overlay.
- `host-*-first-paint.json`, `edit-verification.json`: 첫 선택·프로그램·실제 편집 및 재열기 관계.
- `protected-comparison.{json,tsv}`, `submission-readiness.json`: 별도 정본 보호의 실제 진행 상태와 미완료 조건.

원시 로그·브라우저 프로필·비공개 글꼴 프로그램은 ignored T7 output에만 둔다. 기록의 `$USER_HOME`·`$T7`는 출처 경로의 기계별 부분을 치환한 표기다.

현재 보호 측정은 1,544/3,542쪽이다. WASM 편람 HWP 30·168·194·258쪽 및 HWPX 258쪽의 0.01%p 하락이 남았다. 최신 base와 후보의 해당 5쪽 PNG는 동일하다. `protected-residual-investigation.json`·`handbook-halfwidth-pdf-fonts.json`·`protected-residual/`에 글리프와 Print 프로그램 근거를 보존했다.
