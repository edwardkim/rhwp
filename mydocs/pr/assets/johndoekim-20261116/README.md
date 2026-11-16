# johndoekim #9081·#9084 최종 메인테이너 검증

현재 원격 통합과 후속 처리 상태는 [종료 원장](closure.md)을 따른다. 수용 PR #9126은 원 기여를 보존해 정상 병합됐다.

#9081은 숨긴 누름틀 안내문 뒤의 글상자 표 줄 구성을, #9084는 값이 든 누름틀 끝 표식의 HWP 저장 순서를 수정한다. 원 기여 SHA를 보존했다. 제품·assertion의 추가 보정은 없으며 #9081의 실제 측정 근거 주석과 검토 증적을 보완했다.

| 대상 | SHA |
| --- | --- |
| #9081 원 head | `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738` |
| #9084 원 head | `81dd920099bae78d8d66148ddf9614785760deac` |
| 최신 devel | `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` |
| 실제 최종 Native·최적화 WASM 생산·Rust 검사 source | `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb` |

[최신 source의 실제 검증 기록](latest-devel-revalidation.md)이 최종 판정의 근거다. [직전 단계 기록](validation-record.md)과 [직전 보호 결과](protected-validation.md)는 이전 source의 완료 증거이며 새 source의 실행을 대신하지 않는다.

## 목표 전쪽 비교

한/글 2022 `12.0.0.535`의 실제 저장본 2종과 Microsoft Print to PDF를 사용했다. Native Skia와 최적화 WASM Canvas의 색상 경계 보정 전 2px raw, 96dpi이며 입력·PDF·DPI·인쇄 프로필·제외 영역을 바꾸지 않았다. 두 문서 모두 독립 PDF와 전체 1쪽이 같다.

| 입력 | base Native / WASM | 후보 Native / WASM |
| --- | --- | --- |
| 안내문 있음 | 29.43% / 29.43% | 100.00% / 100.00% |
| 누름틀 없음 | 100.00% / 100.00% | 100.00% / 100.00% |

| 입력 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 안내문 있음 | [review](latest-devel/candidate/native/guide/review.png) / [overlay](latest-devel/candidate/native/guide/overlay.png) | [review](latest-devel/candidate/wasm/guide/review.png) / [overlay](latest-devel/candidate/wasm/guide/overlay.png) |
| 누름틀 없음 | [review](latest-devel/candidate/native/no-field/review.png) / [overlay](latest-devel/candidate/native/no-field/overlay.png) | [review](latest-devel/candidate/wasm/no-field/review.png) / [overlay](latest-devel/candidate/wasm/no-field/overlay.png) |

전쪽 TSV는 `latest-devel/<base-or-candidate>/<native-or-wasm>/<입력명>/`에 있다. 같은 실제 raster로 만든 review/overlay에서 두 표의 행·열·순서·간격·글상자 내부 포함을 직접 판독했다. base의 둘째 표 이탈·누락을 복원했고 단순 점수로 판정하지 않았다.

## 실제 저장과 기존 진단 대조군

한 글자 값·네 글자 값·안내문 3입력에서 실제 CLI와 같은 최적화 WASM의 공개 API로 편집·HWP 저장·재열기를 두 차례 실행하고 HWPX 교차 저장도 확인했다. CFB/DEFLATE/PARA_TEXT의 독립 토큰 판독을 적용했다.

| 입력 | 원본의 FIELD_END / 표1 / 표2 | base 반복 저장 | 후보 첫·둘째 저장 |
| --- | --- | --- | --- |
| 값 한 글자 | 9 / 17 / 25 | 표 9·17 뒤 끝 25 | 원본과 동일 |
| 값 네 글자 | 12 / 20 / 28 | 표 12·20 뒤 끝 28 | 원본과 동일 |
| 안내문 | 9 / 17 / 25 | 원본과 동일 | 원본과 동일 |

필드 이름·값도 유지됐다. HWPX는 base·후보 모두 `fieldBegin → 값 → fieldEnd → tbl → tbl` 순서로 기존 정상 동작을 보존했다. 실제 브라우저 WASM 시각 검증과 Node WebAssembly 저장 검증을 구분해 기록한다.

#9058의 공개 한컴 저장본 3종은 별도 줄 소속 문제의 진단 대조군이다. Native base/후보는 64.87·57.23·58.36%, WASM은 64.87·57.24·58.40%로 각각 같으며 각 backend의 PNG 3쌍이 모두 동일하다. 이 낮은 출력을 새 정상 fixture/golden으로 등록하지 않았다. #9058은 OPEN을 유지한다. 입력·PDF는 `issue9058/`에 보존하고 같은 한 글자 HWP는 기존 `samples/issue9082/`를 재사용한다.

## 공통 검사와 특별 보호

- 최소 글꼴 전체 회귀 **13,632 passed / 0 failed / 기존 skipped 45개**, 집중 30건 통과.
- Native 공식 검사 **4,073 passed / 0 failed**, 모든 Rust lint·workspace build·suite 수집 준비 통과.
- 공개 공통 4문서 × Native/fresh WASM 8개 실행은 모두 passed·raw 최저 100.00%.
- 명시적 Native 편람 383쪽 `--handbook-gate` 통과.
- 새 devel #9106의 각주 50쪽·중첩 표 17쪽은 Native/WASM 각각 base·후보의 PNG 67개가 모두 같다. Native SVG 글꼴 준비·분할 셀·Print·CanvasKit의 관련 경계도 검증했다.
- [특별 보호 전쪽 판정](latest-devel/protected/README.md): **총 3,542쪽 중 3,541쪽은 하락 없음**, Native 중첩 표 10쪽 한 행만 새 하한 99.86%에 대해 99.85%인 기존 차이를 [작업지시자가 한정 수용](native-floor-decision.md)했다. 다른 하락·누락·쪽수 불일치·측정 실패는 0건이다. 정본과 허용치는 변경하지 않았다.
- 사설 구형 수식·HFT 기호 프로그램은 별도 깨끗한 한컴 Print 표본에서 복원했다. 대상 PDF를 원료로 사용하지 않았고 프로그램·복원용 전체 PDF는 Git·배포물에 넣지 않는다. 지속 로컬 공급 위치와 SHA를 검증 기록에 남겼다.

## 원격 단계

[원격 처리안](remote-action-plan.md)의 push·Open PR·새 head CI·self-review·원 기여를 보존한 merge·댓글·이슈 종료·운영 기록·동기화·정리를 승인받았다. 현재 로컬 검증을 완료했으며 원격 단계의 성공은 실제 실행 뒤 확정한다.
