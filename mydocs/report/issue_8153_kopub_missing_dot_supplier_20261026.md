# #8153 KoPub 결측 한점 리더의 공급자 메트릭 보정

[구현 계획](../plans/issue_8153_kopub_missing_dot_supplier_20261026.md)의 원본·독립 Print·
원 프로그램 근거를 따른다. 두 입력과 기준 PDF, 글꼴 파일을 바꾸지 않는다.

## 실제 결함과 구현

원본 368쪽의 세 U+2024는 실제로 HBATANG으로 표시되지만 KoPub의 부분 폭 표가 결측을
전달하지 않아 반각 추정 6.13333px로 진행한다. 원 프로그램 178/1024em과 기존 장치 계산의
2.08px는 독립 Print의 2.07935px와 대응한다. 공백은 원래 6.56px가 맞는다.

기존 프로그램 SHA와 이름 해석을 가진 `font_metric_cjk_punctuation`에 해당 cmap 결측을
연결했다. 다른 문자에서 불필요한 이름 정규화가 늘지 않도록 원래 블록의 빠른 탈출도 유지한다.
원 프로그램의 독립적인 named-cmap 결측을 측정의 U+2024 gate에 전달하며, 부분 폭 표 전체를
신뢰 상태로 바꾸지는 않는다. 선언한 paint 체인의 확인된 공급자가 가진 기존 메트릭을 쓰고
기존 장치 전진을 계산한다. HFT 보호·유효한 원 glyph·공백·다른 미확인 family 계약은 유지한다.

## 수정 전 재현과 현재 상태

`issue_8153_kopub_missing_dot_supplier.rs`는 두 원본 실제 368쪽의 점 전진, 원래 공백,
셀 소유·내용 순서와 KoPub 자체 U+2027의 전진을 보호한다. 최종 테스트 SHA-256은
`7c232643f184e6677ac1b2f4298c3fa6c0c6aae489cd8461435b3cb05e15b906`이다.
고정 `86e770b48fb1881f43ace48df9deaded6f970c57`에서 두 건 모두 실제 점 전진으로
FAIL을 확인했다. 주석 띄어쓰기 전 초안과 진단 probe의 잘못된 명령 인자 오류는 별도
ignored 로그로 보존하며 최종 재현 근거와 구분한다.

새 구현의 Native 빌드·76건 집중 검사와 실제 출력, 표준 최적화 fresh WASM 및 전쪽
TSV 검증은 이어서 수행한다. 이전 104조건의 통과나 원 프로그램 분석만으로 이를 대신하지 않는다.
현재 구현·근거 기록은 전쪽 100%·최종 테스트 완료 또는 PR 준비 완료 판정이 아니다.

모든 실행 로그·중간 JSON·글꼴과 SO 분석은 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/`에 둔다. upstream 동기화와 원격
게시·PR 생성은 하지 않는다.

## 고정 `7620cc75`의 실제 Native 확인

Native 빌드는 공용 `target/pr-review`, jobs 8로 729.49초 후 종료 코드 0이었다.
원래 74건과 새 두 기호 검사를 합친 76건은 221.57초에 모두 통과했다. 집중 비교 104조건은
기존 기호 보정 producer `5bcb654516` 대비 미대응 픽셀 증가가 없었다. 두 원본의 368쪽은
각각 122→0픽셀, raw 100%가 됐다. HWPX 368쪽 overlay에서 세 점·같은 공백·표 경계와
접수 문구를 직접 대조했다. 전쪽 완성이나 fresh WASM 성공으로 표시하지 않는다.

독립 정상 대조군 `form-002` 10쪽과 #5720 원본 1쪽도 실제 출력했다. 부분 표의 수평선
182.5px와 전체 쪽수가 유지됐다. 이는 두 대조군의 전쪽 시각 일치율 완료 주장이 아니다.

이후 HWPX 전 383쪽은 317쪽이 raw 100%·미대응 0픽셀이고 최저 99.45644%, 미대응
총합은 3,185픽셀이다. 같은 이전 producer의 292쪽보다 늘었으나, 46쪽은 이전 0픽셀에서
17픽셀로 증가했다. 새 기호는 그 문단에 없으며 원본은 CENTER·condense70이다. 이전
공백 압축 구현까지 포함한 전쪽 비교에서 발견한 실제 회귀이므로 100% 목표의 완료나
PR 준비 완료로 처리하지 않고 공통 fit 계약을 보정한다.

HWP 원본도 383쪽 PNG가 전부 생성됐다. TSV 계산 직전 작업이 SIGTERM(143)으로 종료돼
전체 raw/copied PNG의 해시를 다시 확인한 뒤 점수 단계만 복구한다. 신호 발신원은
확인되지 않았다. 복구 스크립트의 Python spawn 진입점 누락·불완전 TSV 상태 가정 오류는
수정하고 최초 오류 로그를 보존했다. 입력·기준 PDF·PNG·측정 임계값은 바꾸지 않았다.

표준 release fresh WASM은 같은 고정 source의 별도 검증 worktree에서 Rust 컴파일을
10분 39초에 마쳤고 현재 `wasm-opt -O` 실행 중이다. 빌드가 끝난 뒤 산출물 해시·표준
최적화 영수증과 실제 Canvas 집중·전쪽 결과를 확인한다. 기존 source의 WASM 통과로
대신하지 않는다. 원본 worktree와 upstream/devel은 동기화하지 않았다.

집중 결과는 작업 루트의 `handbook-kopub-dot-supplier-candidate/native-focus-result.json`,
`selected-regressions-run.log`, `normal-controls/result.json`에 있다. 전쪽 TSV는 같은
폴더의 `native/<문서>/whole-score/silhouette.tsv`, 신호·복구 기록과 실제 실행 로그는
`logs/handbook-kopub-dot-supplier-*`에 연결한다. 모든 1,532조건 raw 100%·미대응 0과
최종 필수 검증을 마친 뒤 PR 준비까지만 하고 대기한다.
