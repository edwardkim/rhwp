# #9017 리베이스와 기존 검증 자료 재사용

- 기존 검증 source: `30fa7739f0340f2a248b22542c7dd36d9354f056`, base `1a3ed4a7cd76701085a24ac81d30bc5fb31942c1`.
- 현재 후보: `73d06ed0b18df7c222a850bb5d9340c9b7de2bcc`, 새 base `a85d3265dd10b316d1a0ae029ca3f67245b38b77`.
- 44개 커밋을 충돌 없이 리베이스했으며 모든 patch ID가 같다. 작업 중 자료 283개를 원래 해시 그대로 복원했다.

## 재사용하는 자료

[경로·자료 동일성 감사](reuse-audit.json)에서 renderer/parser/serializer/WASM API/crates/scripts/assets/ttfs/Studio 소스 및 Cargo 입력 13개 경로가 동일함을 확인했다. 기존 samples/PDF 파일도 개별 Git blob이 모두 같다. 새 문서와 PDF가 추가됐다는 이유로 기존 자료의 동일성을 디렉터리 전체 동일성으로 표현하지 않았다.

기존 공개 34종·46쪽씩의 Native/fresh WASM 전쪽 TSV·PNG와 정본 18종·1771쪽씩의 TSV·대표 판독, 독립 Print PDF와 영수증을 재사용한다. [상위 검증 자료](../)의 원래 생산 SHA와 실행 날짜를 유지한다. 새 head에서 다시 캡처하거나 전체 회귀를 실행했다고 이름만 바꾸지 않는다. 반복하지 않는 항목은 기존 전쪽 캡처·동일 PDF 출력·변경 없는 Studio 전체 npm 테스트다.

## 새 변경과 필요한 확인

#9017은 HTML 붙여넣기의 첫 서식 앞 평문 run과 table border 속성, 속성 이름·따옴표 값 경계를 보정했다. 제품 변경은 `src/document_core/commands/html_import/inline_content.rs`, `src/document_core/helpers.rs`, `src/document_core/html_table_import.rs` 3파일이다. 변경한 helper는 HTML 속성 소비 경로에 연결된다. 저장된 기존 HWP/HWPX를 열어 조판·그리는 경로에는 이 붙여넣기 함수가 호출되지 않는다.

새 devel과 후보에서 같은 최소 글꼴 환경의 붙여넣기·반복 저장 회귀를 실행하고, 현재 WASM에서 실제 브라우저 API의 본문·셀·속성 반례와 HWP/HWPX 두 차례 저장을 확인한다. 새 Rust source에 필요한 회귀·lint/build/Skia는 별도로 기록한다. 이 실행이 끝나기 전에는 새 source의 전체 검증 완료로 표시하지 않는다. 이전 전체 13,365 PASS는 `30fa7739`의 실행 결과다.

## 현재 추가 검증 결과

새 base Native의 HTML 붙여넣기·속성 경계·반복 저장 23건은 모두 통과했다. 같은 23건과 기존 141건을 포함한 후보 Native 집중 **164건도 모두 통과**했다(실행 12.269초, 선택 밖 6,290건은 실행 수에 넣지 않음). [실제 명령·검사명·로그 해시](focused-validation.json)를 연결한다. 기준 worktree의 공유 캐시가 남은 초기 실행은 소스나 기대값을 바꾸지 않고 현재 root·chart 의존성을 재컴파일해 분리했다. 초기 실패를 최종 성공 근거로 사용하지 않는다. 최신 WASM과 나머지 필수 게이트는 별도로 확인한다.

## 최종 로컬 실행 완료

현재 source `73d06ed0b18df7c222a850bb5d9340c9b7de2bcc`에서 전체 **13,380 PASS / 0 FAIL / 45 skipped**(실행 404.945초)와 [모든 필수 게이트](gates-validation.json)를 완료했다. Native 집중 164 PASS, 새 base/후보 HTML 각각 23 PASS, 최신 WASM 실제 브라우저 6사례·반복 저장 24회·API11건도 통과했다. [현재 Native](focused-validation.json)·[실제 WASM 소비](browser-runtime-result.json)를 원래 시각 producer의 자료와 구분한다. source·테스트·기대값은 바꾸지 않았고 공유 캐시의 root/chart 산출물만 현재 source로 재컴파일했다. 원격 PR CI는 별도 조건이다.
