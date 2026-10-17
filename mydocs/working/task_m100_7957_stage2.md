# Task #7957 2단계 — F1 요청 정책·원인 관측

- Issue: #7957; 공동 #7958·#7959·#7960·#7961·#7962·#7963
- 작성일: 2026-10-17 (Asia/Seoul)
- 상태: F1 기반 구현·Native/JS 집중 검증 완료. 실제 새 Docker WASM/Worker와 시각 검증은 후속 통합 게이트
- 승인: 사용자의 “계획 승인·구현 시작”. [공동 계획](../plans/task_m100_7957_impl.md)의 F1

## 변경한 계약과 소비 경로

- `PdfImportPolicy`가 backend의 기본 정책과 frontend 쪽·path 한도를 묶는다. 같은 정책을
  원본/선택 쪽 검증·누적 path 검사·성공 metadata에 전달한다. 입력 상수는 backend에서
  공유하며 F1에서 기존 한도를 높이지 않았다.
- `HwpDocument.getPdfImportPolicy()` → 후보 Worker의 policy 응답 → 호스트의 복사 전
  입력 검사 → 같은 후보의 import 순서다. JavaScript의 별도 16 MiB 상수는 제거했다.
  초기화/정책 조회와 가져오기는 하나의 deadline을 소비하고 현재 문서는 성공 뒤 교체한다.
- backend의 최초 실패는 성공 used·거절 requested·limit를 보존한다. 실패한 요청량을 성공
  counter에 더하거나 나중 reason으로 덮어쓰지 않는다. 요청별 snapshot과 content parse,
  page operator/page stream/Form stream cache hit를 관측한다. 실제 예약·paint를 생략하지 않는다.
- Rust `PdfImportError.diagnostics`는 stage·0-based 원본 쪽·backend 종류/원문과 budget을
  보존한다. 기본 Display는 원래 code/message를 유지하며 source 문자열을 공개하지 않는다.
  이 절편에서 상세 실패는 opt-in Native 로컬 진단이며 일반 WASM 문자열 오류와 구분한다.

## 실제 입력의 최초 거절

아래는 F1 초기 Native library에서 같은 입력을 직접 파싱한 관측이다. 입력·소스·probe 실행
파일 hash는 ignored `output/pr-review/issue7957/f1/probe-source.json`에 보존한다.
후속 오류 객체 boxing·공통 정책 전달 정리는 이 최초 진단 producer와 구분한다.

| 이슈 | 물리 쪽 | 원인/거절 예약 |
| --- | --- | --- |
| #7957 | 전체 쪽수 검사 | 요청 162, 현재 상한 128 |
| #7958 | 1쪽 text | operators used 250,000 + requested 1 > limit 250,000 |
| #7959 | 1쪽 image_decode | `Unsupported feature: interpolated image mask` |
| #7960 | 19쪽 path | 문서 path used 79,964 + requested 86 > limit 80,000 |
| #7961 | 1쪽 image_decode | `Image error: invalid masked RGB sample length` |
| #7963 | 5쪽 lowering | decoded used 229,319,292 + requested 44,236,800 > limit 268,435,456 |

#7961 원본 객체 45의 `/DecodeParms`는 객체 59에 대한 간접 참조이며 PNG predictor 15,
Colors 3·Columns 1376·BPC 8이다. qpdf가 해제한 RGB 크기는 3,170,304 bytes로
1376×768×3과 일치한다. 실제 backend가 그 간접 파라미터를 같은 경로로 해석하는지는 후속
확인 대상이며, 마스크 길이 검사 실패만으로 원본 컨테이너 손상으로 판정하지 않는다.

## 현재 실행과 남은 게이트

- Worker 세션 집중 검사: 9 PASS. 정책 기반 거절·visible view 복사·후보 재사용·기존 문서,
  정책 조회 중 취소·deadline·malformed policy 폐기와 16 MiB 초과 정책 소비를 포함한다.
  mock Worker 증거이며 실제 새 WASM 프로토콜 실행을 대신하지 않는다.
- Native library release-test 빌드: 성공. 위 초기 원인 관측을 얻었으며 수정 후 문서 열기의
  성공이나 성능 개선을 주장하지 않는다.
- 초기 `cargo fmt --all`은 새 worktree에 generated suite 준비가 없어 실패했다.
  `--prepare` 뒤 성공했다. 이 준비 오류를 제품 결함의 수정 전 FAIL로 세지 않는다.
- 오류 객체의 상세 context는 boxing해 기존 오류 반환 경로에 큰 inline 오류를 강제하지 않는다.
- Native frontend·resource 집중 검사 114 PASS, Native library Clippy `-D warnings` 통과.
  명령·로그는 `focused-tests.log`, `native-clippy.log`에 있다. 전체 검사·WASM32 lint를
  실행한 결과가 아니며, backend public 관측 필드의 문서 보완 뒤 Native Clippy도 통과했다.
- 한도·image/parse 호환 보정, 최종 Docker WASM·실제 Studio/Worker·전쪽 TSV·전체 회귀·
  원격 PR은 아직 완료하지 않았다. 승인된 F2로 진행하며 바뀐 코드의 검증을 이어간다.

baseline·허용치와 개인 PDF는 변경하지 않았다. 기본 작업공간의 미커밋 타입 2개·Studio 7700,
공용 `target/pr-review`를 보존한다. 후속 F2–F5는 승인된 공동 계획 안에서 순서대로 진행한다.
