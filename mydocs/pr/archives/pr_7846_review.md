---
kind: investigation
status: active
canonical: mydocs/pr/archives/pr_7846_review.md
last_verified: 2026-10-13
---

# PR #7846 리뷰 — Zeroize lock 갱신

## 최종 판정

**메인터너 보정 후 수용 가능 — Dependabot 통합 범위의 필수 로컬 검증 완료.**
원 PR 단독 결과와 누적 보정 결과를 구분합니다. GitHub 통합 CI와 병합 조건은 아직 미확인입니다.
사용자의 2026-10-13 지시에 따라 review·오늘할일을 PR 생성 전에 함께 포함하며,
CI 성공 후 같은 PR에 정확한 run/head/result를 보완합니다.

## 접수·적용 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7846](https://github.com/edwardkim/rhwp/pull/7846) / dependabot[bot] |
| base | devel / `bfa28a77a2714482f647ee6a7b9aa7f7876d8404` |
| 원 head | `1fdb145906cf7f603d34263f341f3c0a7e0211f6` |
| cherry-pick -x | `25c026891a37411a829c2cd1f396402777172ca7` |
| 로컬 검증 코드 후보 | `d6d8f5bc2031a175c350e9be5daffe0525c05e4a` |
| branch | `review/dependabot-20261012` |
| 원 PR 상태 | OPEN, 적용 당시와 head 동일; 원 branch 수정 권한 false |
| 경로 | 사용자 승인 별도 통합; owner reviewer 자동 지정 없음 |

## 변경과 메인터너 보정

zeroize 1.9.1 lockfile 갱신을 적용했습니다. 암호 알고리즘이나 키 수명 정책을 변경하지 않았으며 암호 helper·workspace와 전체 회귀 검증에 포함했습니다.

공통 보정으로 네이티브 Rust GPU 폐기·CLI 교본 정합, ZIP 9 조회/오류 호환,
CodeQL init/analyze 동일 버전 그룹, 기존 IME 검사의 OS별 예약 키 가정을 처리했습니다.
사용자가 추가로 승인한 #7445 브랜치의 Visual Sweep 지침과 Gym·Rust·Studio 시험지 제외도
함께 이관했습니다. 원본 파일과 #7445 브랜치의 조판·PDF 개선은 보존했습니다.
원 브랜치의 이관 역변경 commit은 `28be39176`이며, 이 통합을 먼저 반영한 뒤 별도로 검증합니다.

## 검증 입력과 결과

공용 target `/Users/tsjang/rhwp/target/pr-review`를 사용해 Cargo를 순차 실행했습니다.
명령·source 신원·WASM 해시·원 PR별 출처는 [공통 검증 증적](../assets/dependabot_20261012/final_validation.json)에 있습니다.

- 전체 release-test nextest: **10,217 PASS / 0 FAIL / 45 SKIP, 443.078초**.
- Native Skia: 라이브러리 4,046 PASS / 0 FAIL / 12 IGNORE, PNG 2 PASS, 직접 PDF 4 PASS.
- fmt·Native/WASM32/workspace all-target Clippy·workspace build·base 고정 manifest/unit-tier 통과.
- fresh WASM: Mac 로컬 no-opt 빌드와 Studio public SHA-256 일치. Docker 최적화 빌드가 아닙니다.
- Studio 1,834 PASS / 0 FAIL / 2 SKIP. Studio/Chrome/Firefox build·dist 계약 및 실제 Chrome
  iframe embed·Canvas legacy/layer 3개 입력·패키징 확장 smoke 통과.
- Gym 3,170건 실패 0 / skip 1, Visual Sweep 단위 83 PASS, font decision trace 2 PASS,
  renderer 계약의 시험지 제외 후 건수·미포함 검사 통과.
- workflow 186 PASS, CI 정책 89 PASS, CodeQL 그룹 22 PASS, 명령 교본 14 PASS.
  actionlint 통과 (ShellCheck는 명시적으로 제외).

추가 글꼴 증적 감사는 18건 중 17 PASS / 1 FAIL입니다. base에도 없는 두 이전 문서 경로
참조가 원인이며, 해당 감사 코드·candidate·검사는 base와 동일합니다. 이 결과를 전체 통과로
세지 않았고 상세 근거를 공통 증적에 기록했습니다. 통합 CI 결과를 원 PR CI로 대체하지 않습니다.

## 조판·시각 영향과 범위

살아 있는 layout·paint·Skia 및 브라우저 production renderer는 base와 동일합니다.
ZIP은 Unicode/CP437 이름·크기·내용 조회와 자원 예산을 검사했습니다. GPU 제거는 별도 폐기
출력 경로이며 Vello 0.11 런타임 채택·성능 검증을 주장하지 않습니다.
절대 px/쪽수 기대값·golden·래칫 허용치를 완화하지 않았습니다. 시험지 제외는 사용자 승인
검사 범위 분리이고 원본 파일은 유지했습니다. 새로운 실물 배치 회귀를 추가하지 않았습니다.

Canvas legacy/layer 비교는 KTX p1 209/891,662 pixels 차이(0.02344%), biz_plan·tac-case-001 p1
차이 0입니다. 이는 두 브라우저 경로 비교이며 독립 한컴 PDF 일치율이나 전쪽 90% 통과 증거가
아닙니다. 독립 Print PDF에 대한 조판 개선 수용을 주장하지 않습니다. Firefox는 build/dist만,
실제 런타임은 Chrome만 검증했습니다. Firefox/Edge 실제 동작은 미검증입니다.
Visual Sweep은 PDF thin-line shape rasterizer와 provenance를 보완했고 90%/2px gate는 유지합니다.

## Merge 후 contributor PR comment 계획

사용자가 2026-10-13 통합 CI 성공 후 병합과 후속 처리를 승인했습니다. 최신 통합 head의 필수
CI·보호 요건과 MERGEABLE/CLEAN을 확인해 정상 경로로 병합하고 admin 우회를 사용하지 않습니다.
실제 merge SHA·CI run URL·본 검토 기록·공통 증적을 한국어 존댓말로 원 PR에 설명합니다.
원 기여와 보정 이유/범위, 시험지 분리와 알려진 미검증 범위를 구분하며 원 head 재조회·기존 댓글
확인 후 `--body-file`로 게시하고 API로 본문을 재확인합니다. 통합 대체 병합 사유로 원 PR을
종료하되 contributor branch와 미완료 관련 이슈를 보존합니다. #7445 전체를 닫지 않습니다.
시각 개선 판정에 Print PDF/PNG를 사용하지 않았으므로 merge SHA 고정 raw PNG는 비해당입니다.
병합 후 duration refresh 결과, devel 동기화와 소유 정리를 확인하고 공유 target·기존 worktree는 보존합니다.

## CI·병합 상태

통합 PR 생성 및 code head CI는 대기입니다. 병합·원 PR 댓글/close·devel 동기화는 아직 실행하지 않았습니다.
