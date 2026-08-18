# M-rcpt: 작업 영수증·감사·계보 픽스처 고도화

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5478
브랜치: `feat/m-rcpt-fatten` (`upstream/devel` 기준 격리 worktree)
범위: `tools/work_receipt/` 만
비범위: gym · canvaskit · serializer · pdf · layout-anomaly · oracle ·
render_backend · proptest · fidelity · hwp5-inventory · inspect · page-count

## 무엇을

devel 에 이미 있는 `rhwp replay` / `audit` / `lineage` 의 **기존** 플래그와
봉투 필드를 픽스처로 닫는다. 새 CLI 는 없다.

| 단 | 명령 | 픽스처 | 고정하는 것 |
| --- | --- | --- | --- |
| 영수증 | `replay` | `fixtures/replay/cases/` | attest / verify, 3해시, 사용자 경로 무훼손 |
| 캡슐 | `replay --capsule/--parent` | `fixtures/capsules/` | workCapsule, 상대 부모, 덮어쓰기 거부 |
| 감사 | `audit` | `fixtures/audit-layouts/` | 비재귀, rate=reproduced/total, exit 3 |
| 계보 | `lineage` | `fixtures/lineage/` | parentOk · lineageOk · reproduced · brokenAt |
| 예외 | 세 명령 | `fixtures/exceptions/` | exit 1/2/3 바늘, stdout 0바이트 |

실측: replay **80**, 예외 **38**,
감사 **20**, 계보 **22**.

## 왜

에이전트 노동은 말이 아니라 재실행으로 증명한다. 같은 계획은 같은
바이트를 내고, 그 바이트의 SHA-256 이 영수증이다. 픽스처가 없으면
exit 3 을 도구 고장으로 오독하거나, 빈 폴더(exit 2)와 없는 폴더
(exit 1)를 섞거나, `parent=null` 뿌리와 `parent` 키 없음(fail-closed)을
같은 것으로 본다.

## 어떻게

1. `contracts.py` 가 `src/main.rs` 의 필드·바늘·exit 를 파이썬으로 재현한다.
2. `catalog.py` 가 한국 공공문서 가족(공문·서식·표·계약·고시·시험…)과
   예외·감사·계보 행렬을 가진다. 인덱스만 다른 복제는 케이스가 아니다.
3. `fatten_work_receipt.py` 가 planText UTF-8 의 실제 SHA-256 을 계산해
   디스크에 다시 쓴다.
4. `test_fatten_work_receipt.py` 가 라이브 분류 함수와 픽스처를 대조한다.

## 판정 규약

- 판정은 예외가 아니라 봉투 데이터: `reproduced` · `reproducedRate` ·
  `valid` · `brokenAt`.
- 재현 실패·깨진 체인 = **exit 3**.
- IO = exit 1, 사용법 = exit 2.
- 실패 경로 stdout 은 0바이트. 예외: `replay --json` 엔진 오류 봉투.

## 하지 않은 것

- 새 플래그 / 새 하위명령 없음
- gym pack 없음
- 다른 MEGA 석 파일 없음
- DocumentCore · 렌더 · serializer 없음

## 검증

```bash
python tools/work_receipt/fatten_work_receipt.py
python tools/work_receipt/test_fatten_work_receipt.py
cargo fmt --all -- --check
```
