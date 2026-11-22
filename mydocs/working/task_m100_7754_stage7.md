# 9개 이슈 소스 37: 논리 별칭 회귀와 1-up 정답지 계약 보정

소스 36의 종료 근거는 `task_m100_7754_stage6.md`와
`mydocs/pr/assets/issues7754_7765/source36/`에 커밋했다. 기준은 계속
`1a27faacff48ea7329e76ce8eca15aa9ca8aa877`다. 필수 검증을 줄이거나 정본 하한을
변경하지 않는다. 미시작 소스 36 전체 보호 WASM은 알려진 회귀를 가진 생산물의
중복 실행이므로 보류했고 새 최종 생산물에서 다시 수행한다.

## 원인과 수정 범위

1. `lookup_font_name_in_environment`가 선택한 프로그램의 존재를 원본 HFT 논리 이름의
   실제 family 동일성으로 간주해 새 fallback을 추가했다. 그 논리 이름을 가진 별도
   프로그램의 빈 U+2027 glyph가 두 점을 가렸다. 같은 정본 글꼴의 기준 100.00%와
   후보 99.98%, 같은 글줄 위치·21개 픽셀 차이로 입증했다. 실제 선택한 family만
   조회하는 진단은 기준 PNG를 복원했다. 안내문 2쪽의 Native·stock WASM은 모두
   99.99%를 유지한다. 별도 private context Chrome 155의 중첩 표도 100.00%다.
   이 값은 진단이며 실제 코드 보정 후 원래 조건의 재검증을 대신하지 않는다.
2. 선택된 프로그램의 실제 family·명시적 다른 조회 이름은 유지한다. 근거 없는 논리
   HFT 이름의 새 fallback 추가만 제거한다. 같은 family의 분할 프로그램·옛한글은
   기존 공통 `source_program_glyph` 경로를 계속 쓴다. 원 TTF → 검증된 TTC → CFF
   순서와 원문·문자 주소를 유지한다. 문서 ID·좌표 조건은 추가하지 않는다.
3. 전체 nextest의 두 실패는 시장구조조사·수출입의 선언된 모아 찍기를 기존 oracle가
   무조건 제외하는 규약이다. 새 독립 Print 참조는 실제 전쪽 1-up 내용·쪽수를 이미
   대조했다. 원본·PDF SHA와 검증한 1-up 참조 계약을 함께 읽어 이 둘을 비교한다.
   모아 찍기의 일반 제외·미등록/변경 바이트 거부·정확한 쪽수·내용 보존은 유지한다.
   기존 생성기와 실제 검사기를 같은 계약으로 보정하며 baseline 값을 낮추지 않는다.

## 검증과 후속 순서

- 최소 글꼴의 이름 해석 관계 검사로 논리 HFT 이름의 자동 승격 거부, 실제 명시 조회와
  원 TTF 우선 보존을 확인한다. 원 모아 찍기 제외의 원 변경과 독립 Print 근거도 추적한다.
- 실제 코드의 최적화 Native·fresh WASM을 새 SHA로 생산하고 작은 경계부터 확인한다.
  안내문 1·2·20쪽과 중첩 표 16쪽의 독립 PDF·기준·전후 출력을 대조한다.
- 전쪽 원본 583쪽씩·암호 원본·실제 편집/저장/재열기·유입 기능을 새 후보에서 완료한다.
  변경 없는 고정 기준의 완료 결과는 입력·설정·생산물 SHA가 같을 때만 재사용한다.
- Native 공식 검사·전체 nextest·fmt·Clippy·workspace build와 브라우저·공통 보호·편람·
  정본 전쪽 보호를 최종 코드에서 완료한다. 별도 프로세스의 원본 글꼴 문맥을 보존하고
  시스템 글꼴을 바꾸지 않는다. 알려진 미확보 조건과 표시값 하락을 통과로 바꾸지 않는다.
- 최종 검증·본인 열린 PR CI 우선 처리·병합 시뮬레이션 뒤 승인된 9개 이슈의 PR과
  병합 후속 처리를 계속한다. 다른 PR의 미승인 원격 보정을 임의로 수행하지 않는다.

## 계약 보정의 선행 분석과 첫 검사

- 기존 쪽수 규약은 #6337 → 원 PR #6338의 head
  `940f63d94430d5e95cb8bbf8835a93b0da85ea7d` → 통합
  `9b878414fc6bcf5142ca058b454e842f8fb9a160`에서 추적했다.
  `mydocs/pr/archives/pr_6338_review.md`와 통합 검증 기록은 566개 PDF 짝,
  focused 통과·전체 8,576건 통과를 남긴다. 현 Git에 기록된 적용 단축 SHA 두 개의
  object는 없고, GitHub plugin과 로그인된 gh의 현재 원 PR API는 모두 404다.
  원 PR 논의를 읽었다고 주장하지 않는다. 현재 base의 실제 규칙과 통합 원문을 대조했고,
  PrintMethod 4·5의 물리 장수 제외는 그대로 유지한다.
- 독립 논리 쪽 참조는 `oracle_page_count_verified_references.json`의 기존 원본·PDF
  SHA에 Native/fresh WASM 전쪽 TSV SHA를 연결했다. 생성기·독립 sweep은 같은
  Python 검증 함수를 쓰고, Rust 원장도 같은 등록과 바이트·연속된 전쪽·raw 98%를
  직접 확인한다. 등록은 검증한 PDF와 논리 쪽 대응의 증거이며 원본의 PrintMethod 값을
  바꾸거나 실제 인쇄 작업 설정을 추정하지 않는다. 기준선 쪽수와 내용 sentinel은 유지한다.
- Python 계약 18건 통과: 일반 모아 찍기 제외, 검증한 논리 쪽 참조의 수집 유지,
  Native/WASM 누락·변경 바이트·누락/중복 쪽·97.99%·NaN 거부를 포함한다.
  최소 글꼴의 Rust helper 독립 probe 2건도 통과했고 실제 두 참조를 모두 수용했다.
  Rust SHA-256 배열의 LowerHex 미지원 빌드 오류는 byte별 hex 변환으로 보정한 후
  다시 컴파일·실행했다. 이는 정식 oracle partition 실행을 대신하지 않는다.
- 소스 36의 public API 이름 probe는 exit 101로 논리 HFT family의 잘못된 승격을
  재현했다. 실제 새 생산물의 수정 후 PASS는 아직 실행 전이다. 첫 근거는
  `mydocs/pr/assets/issues7754_7765/source37/preflight/`에 고정한다.

## 실제 새 Native의 첫 경계 완료

소스 `68b4f6bdfda465aaafd0634c587cd6a17252fed6`의 최적화 Native 빌드는 exit 0,
665.9초다. 실행 파일 SHA는
`249e11dedc91f4f2c90b145e82b9bd0102d27a0b1e1ebaa15030431178cdfa02`다.
같은 producer에서 보존한 Native 라이브러리의 public API 이름 계약은 3개 모두
수정 후 PASS다. 원 TTF 조회 우선과 다른 명시적 PostScript 조회 이름도 유지한다.

- 복구한 247개 CFF 중 현재 설치 경로가 있는 파일만 read-only namespace에서 바인드했다.
  TTF/TTC와 시스템 설치는 바꾸지 않았다. 같은 환경의 중첩 표 전쪽 17개에서 후보 PNG가
  새 기준 Native와 모두 바이트 동일하다. 정본의 99.99%·99.98% 쪽도 그대로 유지한다.
  잘못된 별칭이 있던 소스 36은 이 환경에서도 1·5·7·16·17쪽의 raw가 하락했다.
  실제 보정 후보는 다섯 쪽을 모두 복원했으며 16쪽은 99.98% → 100.00%다.
- 안내문 평문 24쪽의 Native 최저 99.48%, 6개 글꼴 우선 대조군 각각 100.00%다.
  소스 36과 23개 PNG가 동일하고 13쪽만 75픽셀, `(587,993)-(606,1005)` 영역에서
  다르다. 독립 PDF·전후를 6배 확대 판독했다. 이는 기존 옛한글 표시의 좁은 영역이며
  raw 99.50% → 99.48%다. 같은 최신 기준의 13쪽 Native도 99.48%다. 이 차이를
  숨기지 않으며 전쪽 98%와 내용 보호를 유지한다. 새 px 핀을 추가하지 않는다.
- 원본 CFF의 해당 문서별 프로세스 복원만으로 issue8040 1·2쪽 100.00%, 4쪽 99.99%,
  sample16-hwp2022 51·61·62쪽 각각 100.00%가 복원됐다. 이는 환경 진단의 소스 36
  결과이며 소스 37의 전체 보호 검증을 대신하지 않는다. 단순한 이름 일치가 아니라
  아카이브에서 복구한 기존 정본 파일 SHA의 바이트를 사용했다.

정식 근거는 `mydocs/pr/assets/issues7754_7765/source37/native-boundaries/`다.
fresh WASM 생산·전쪽 실행과 정식 Rust 게이트는 진행 중이며 아직 완료가 아니다.
최신 upstream은 `bd3cd171331e80e79881dc8e9ed37f837ef6be77`로 fetch했다.
기존 기준 이후 README·기록·이미지뿐이며 생산 코드·테스트·검증 도구는 동일하다.
현재 producer의 정확한 source SHA를 보존한 뒤 동기화한다.

## fresh WASM 생산과 통합 검사 경로 보정

같은 소스의 최적화 fresh WASM도 exit 0, 655.5초로 생산했다. `rhwp_bg.wasm` SHA는
`76f7c64d27fe8fbbd7bd5baa2fdb3d30af9c8014fd16fb6b83afc498907ba7e3`다.
실제 브라우저 경계·전쪽 측정과 Rust 검증은 계속한다.

첫 정식 unit-tier 정책 검사가 소스 내부 test 총량 4,093 > base 4,092로 실패했다.
정책·상한을 바꾸지 않고 새 이름 계약을 public `DocumentCore` 통합 검사
`tests/cases/issue_7754_font_logical_family_ownership.rs`로 옮겼다. 기존 내부 tests는
작업 전 그대로 복원했다. 같은 3개 이름 소유·우선 조건을 그대로 보호하며 기존 자동
suite 수집 경로를 쓴다. 최소 글꼴의 실제 통합 검사로 소스 36 FAIL / 새 Native PASS를
다시 확인했다. unit-tier도 base 대비 4,092건·295모듈로 PASS다. 정식 Cargo 실행은
수정된 등록을 준비한 worktree에서 재시작한다. 이 변경은 `#[cfg(test)]` 모듈과 통합
검사에만 영향을 주며 Native/fresh WASM의 생산 동작은 같다.

두 tracked WASM d.ts 생성물의 변경은 줄 순서만 다르고 전체 선언·서명 Counter가
동일함을 확인했다. 작업이 생성한 이 두 파일만 HEAD로 복원했으며 최적화 package와
실제 브라우저 공급물은 보존했다. 관련 근거는 `source37/integration-contract/`다.
