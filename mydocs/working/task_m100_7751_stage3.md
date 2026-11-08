# #7751·#7494 검증 계약 복원 — 3단계

2026-10-16. 생산 보정은 `064bbcfad`로 고정했다.
[2단계](task_m100_7751_stage2.md)의 전383쪽 Native/fresh WASM90% 이상·쪽수 일치와
직접 판독을 선행 근거로 사용한다. 같은 원본의 완료 페이지3·241·289·290쪽에서만
독립 PDF 좌표±2px를 추가하고 내용·소유·순서·포함 관계도 유지한다.

## 실행 순서

1. 기존 `tests/cases/issue_4764_pdf_raster_fidelity.rs`에서 실제 HWP를 열어 전체 쪽수와
   부록 간지·빈 쪽·말미 판권의 내용 소속을 검사한다. 서문3문단의 내용과 줄 수·연결,
   표 이어받기 끝과 다음 제목/표, Q13–Q16 경계는 `issue_7751_handbook_completed_boundaries.rs`로 보호한다.
2. 최신 사용자 지시에 따라 `3522cbf01`의 `ci.yml` 추가2줄과 `63312e6f6`의 삭제2줄을
   각각 제외했다. 대체 커밋은 `d3f121641`과 `30d3bc311`이며, 두 커밋의 나머지 파일이
   원래 커밋과 바이트 단위로 같음을 확인했다. 이력 변경 뒤 `64bf1a81d`의 전체 tree는
   생산 검증 커밋 `064bbcfad`와 동일하고 검토 커밋 범위의 `ci.yml` 변경 이력은0개다.
   픽셀 검사는 기존 Native CI가 호출하는 `render_p37_direct_pdf_export`에 유지하고,
   기대값을 실제 한컴 Print의 RGB 상수로 개선한다. `issue_4764_pdf_raster_fidelity.rs`는
   실제 HWP 쪽수·내용 소속을 보완해 전체 Rust 회귀에서 실행한다.
   alpha/object opacity·배경의 저장 순서·이산 흑백/패턴·
   미지원 워터마크 경계는 계속 검사한다. 원시 PDF와 관측표는 `pdf/issue7751/`에 있다.
3. 미커밋 테스트를 보존한 채 로컬에서 수정 전 `37ad1b52b`의 라이브러리를 공용
   `target/pr-review`에 순차 빌드해 새 경계 검사를 실행한다. 소스/환경 실패는 결함
   검출로 세지 않는다. 원래 검토 브랜치로 복귀해 같은 검사의 통과를 확인한다.
4. manifest를 준비한 뒤 focused·전체 release-test·Native Skia3종과 필수 lint를
   실행한다. Cargo를 겹치지 않고 jobs8·공용 절대 target을 유지한다.
5. 같은 HWP를 쓰는 기존 검사에서 px/고정 영역 핀을 검토한다. 정상 관계·원본 크기
   계약은 유지하며 다른 HWPX·문서로 범위를 넓히지 않는다.

최종 PR 생성은 로컬 검증·upstream head 확인·필요한 rebase를 마친 뒤 사용자 승인을
받는다. CI 성공 전에는 정식 개별 PR review/오늘할일의 완료 판정을 작성하지 않는다.

## upstream 동기화와 수정 전 검출

`0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64` 위로 리베이스한 HEAD는
`19c8d2c8a300c2389a4e268eaf9d75df974049cf`다. 미커밋4파일의 SHA-256은
전후 동일하며, 현재 upstream 대비 `ci.yml` 차이는 없다. 새 base의 Rust 차이는
PDF 입력 지원 여부를 묻는 정적 WASM API뿐이며 기존 HWP 조판 호출은 바뀌지 않았다.
실제 diff는 ignored `latest-base-rendering-equivalence.json`에 보존했다.

수정 전 `37ad1b52b`를 같은 절대 target·jobs8·Native release-test로 빌드했고 exit0이다.
같은 새 경계 검사3개는 모두 assertion에서 실패했다(환경·컴파일 실패0).
서문은3줄 대신4줄, 이어받기 뒤 제목은382.453 대 독립376.448px,
Q13 종료와 Q14 시작은 비겹침 관계를 위반했다. 실행 후 검토 브랜치와 작업을 복원했다.
로그: `logs/7751-red-library-build.log`, `logs/7751-red-probe.log`.

기존 HWP 검사 감사에서 `#4753`의3개 클릭점을 실제 셀 중심과 셀 소유로,
`#5952`의 고정 영역·45px 간격을 상자 포함·사이드바 비겹침·다음 제목 순서와
독립 Print69쪽 전체 상자 내용 보존으로 교체했다. 합성 줄 재래핑2검사는 유지한다.
`#3931`의 저장 줄 높이·피치, 선언 높이/바깥여백, `#6389`의 내용/포함,
`#7387`의 독립 글꼴 공백폭 계약은 원본 메트릭 근거를 유지한다. HWPX 전용 검사는
이번 HWP 검증으로 변경하지 않는다.

## 집중 검증 결과

수정 후 실제 실행33건 PASS, FAIL0, 대상 실행 누락0이다.

| 검사 | 실행 | 결과 |
| --- | ---: | --- |
| #4764 실제 쪽수·내용 및 기존 계약 | 11 | PASS |
| Native PDF 색상·alpha·내보내기·미지원 경계 | 14 | PASS |
| #7751 서문·이어받기·Q&A 경계 | 3 | PASS |
| #4753 실제 셀 중심과 hit 소유 | 1 | PASS |
| #5952 상자·사이드바·다음 제목 및 합성 줄 경계 | 4 | PASS |

Native 대비±100의12조건은 독립 boundary Print의 픽셀 상수를 검사한다.
밝기−100/−20/0/20/100과 대비±100의 교차 및 원본128/170의 분기를 포함한다.
구현 식으로 기대값을 재계산하지 않았다. 실제 문서의 완료4쪽(3·241·289·290쪽)에
14개 좌표 기대값을±2px로 고정했고, 소유·내용·순서·포함·비겹침을 함께 검사한다.
선정 근거는2단계 전쪽 Native/fresh WASM TSV와 독립 Print 직접 판독이다.

전체 셀 보존 검사에서 제목행 판별을 원본 `Cell.is_header`와 `repeat_header`로
정했다. 원본 셀의 본문뿐 아니라 분리 저장된 중첩 표·묶음 글상자 내용도 수집한다.
최초의 ‘첫 행은 항상 반복 머리글’ 및 ‘문단 문자열만이 전체 내용’ 가정은 제거했다.
수정 전 검출에 쓴 핵심 assertion은 유지했고 이후 보강한 전체 셀 보존 검사는
수정 후 실행으로 확인했다. 이 보강분 전체를 수정 전에도 실행했다고 주장하지 않는다.

테스트 보강 뒤 파생 harness를 갱신하기 전4개 case가0건 실행된 시도는 통과로
세지 않았다. `logs/focused-before-final-prepare/`에 보존하고 manifest를 다시 준비한
뒤 양수 실행 건수를 확인했다. 최종 로그는 `logs/latest-base-focused-*.log`의
#4764/P37와 `logs/latest-base-focused-round3-*.log`의 나머지3case다.
후속 필수 lint·전체 회귀·Native lib·placeholder·fresh WASM은 다음 단계다.
