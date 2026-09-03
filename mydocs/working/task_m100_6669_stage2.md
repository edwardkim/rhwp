---
kind: working
status: completed
canonical: mydocs/working/task_m100_6669_stage2.md
issue: 6669
last_verified: 2026-09-03
---

# Task M100 #6669 Stage 2 — 결정론적 self-contained HTML renderer

## 1. 결과

Stage 1에서 seal한 Gym JSON 증적을 다시 실행하거나 재채점하지 않고, 사람이 판독할 수 있는 단일
정적 HTML로 변환하는 Stage 2 구현을 완료했다. renderer는 큰 로더에 합치지 않고
`gym/core/evidence_html.py`로 분리했다. CLI facade인 `gym/tools/evidence_report.py`는 seal 검증과
원자적 출력·종료 코드만 조정한다.

상태 계약은 다음과 같다.

| 조건 | HTML | 종료 코드 |
| --- | --- | --- |
| seal이 유효하고 전체 PASS | 생성 | 0 |
| seal이 유효하고 FAIL 또는 INCOMPLETE | 비녹색 보고서 생성 | 1 |
| 입력·신원·manifest가 무효 또는 혼합됨 | 새 출력 없음 | 2 |

실패와 미완료도 조사 증적이므로 유효한 seal이면 보고서를 남긴다. 구조적으로 신뢰할 수 없는 입력만
출력을 거부한다.

## 2. 표시 범위

HTML 한 파일에 다음을 표시한다.

- JSON이 유일한 기계 판독 정본이고 HTML은 비권위 파생 뷰라는 경고
- 전체 상태와 unit/audit/oracle 2종/authority/positive/discrimination/trajectory 상태 카드
- 각 역할의 핵심 수치와 process seconds
- `trajectory.ok`와 `trajectory.trusted`의 독립 표시
- pack별 task/reference, positive pass, control rejection, load-bearing 및 단일-step N/A 분포
- positive 실패·누락·skip, false-pass, 미설명 score error, trajectory theater·예외·skip 상세
- run ID, runner head/tree, product source, binary version/name/hash, 실행 시각과 안전화한 환경
- 41개 입력의 이름·kind·mode·schema·bytes·SHA-256와 generator version

막대는 CSS 표현과 정확한 `성공/전체` 텍스트를 함께 제공한다. 상태도 색뿐 아니라 기호와
PASS/FAIL/INCOMPLETE 문자열을 함께 사용한다.

## 3. 결정성·오프라인·보안

- 생성 시각이나 난수를 넣지 않아 같은 seal과 같은 generator에서 byte-identical HTML을 만든다.
- 외부 URL, CDN, 웹폰트, JavaScript가 없다.
- CSP는 기본 resource를 전부 막고 inline CSS만 허용한다.
- HTML의 모든 자유 문자열은 escape한다.
- POSIX/Windows 절대경로와 HWP/HWPX/PDF/Office 문서명 token을 마스킹한다.
- 마스킹 또는 400자 상한이 적용된 문자열은 원문 길이와 SHA-256만 남겨 동일 진단인지 대사할 수
  있게 한다.
- `uname -a`의 hostname은 `[host]`로 치환한다.
- raw JSON, stderr, binary 절대경로는 포함하지 않는다. binary path는 fingerprint만 표시한다.
- `--out`은 필수 입력과 `evidence-manifest.json`을 덮어쓸 수 없고 symlink 출력도 거부한다.
- seal 검증 뒤 렌더링 중 입력 또는 manifest가 바뀌어도 원자적 교체 전에 다시 감지한다.

## 4. 검증

집중 시험 6개를 더해 Stage 1과 합쳐 19개 계약을 고정했다.

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts.tests.test_gym_evidence_report` | 19/19 통과 |
| evidence report + audit/build-baseline/discrimination/trajectory/authority 묶음 | 575/575 통과 |
| `python3 -m py_compile ...` | 통과 |
| `git diff --check` | 통과 |

추가 시험은 byte-identical 생성, 외부 resource·script 부재, raw path·hostname 부재, 종료 0/1/2,
실패 HTML 생성, 변조 시 기존 출력 보존, HTML injection/path/문서명 redaction, `ok=true`이지만
`trusted=false`인 trajectory의 INCOMPLETE 표시와 보호 파일 덮어쓰기 거부를 확인한다.

작은 합성 입력 smoke에서 PASS HTML은 18,911 bytes였고 SHA-256은
`5d6cafa05c7991fd857412ca215f1553869fa722d472f9152ad2ec13919d8aa8`였다. 이는 임시 입력 검증값이며,
Stage 3에서 저장할 공개 fixture·샘플의 정본 hash는 아니다.

기존 build-baseline 시험의 `ResourceWarning`은 기존 helper가 닫지 않은 임시 파일 경고이며 실패가
아니다.

## 5. 비범위와 다음 단계

- #6628의 1,035건 positive/discrimination/trajectory 전수 실행은 반복하지 않았다.
- 공개 고정 fixture와 저장되는 샘플 HTML은 Stage 3 범위다.
- 브라우저 시각 판정과 offline request 0 확인은 Stage 5 범위다.
- 수동 운영 매뉴얼과 AI 에이전트 지침 현행화는 Stage 4 범위다.

## 6. Stage 3 진입 조건

1. 전체 상태·역할 카드·pack 분포·진단·계보 표시 구성을 승인한다.
2. 유효한 FAIL/INCOMPLETE는 HTML을 남기고 종료 1, 무효 seal은 출력 없이 종료 2인 계약을
   승인한다.
3. 경로·문서명 마스킹과 원문 길이·hash 보존 방식을 승인한다.
4. 승인 뒤 Stage 2 변경을 커밋하고 작은 공개 fixture와 재현 가능한 샘플 HTML을 고정한다.

## 7. 승인 기록

- 2026-09-03: 메인테이너가 Stage 2의 표시 구성, 종료 코드 0/1/2 계약, 민감정보 마스킹과
  Stage 3 진입을 승인했다.
