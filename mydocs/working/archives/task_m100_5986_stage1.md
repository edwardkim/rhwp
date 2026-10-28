# Stage 1 사후 재구성 보고 — Task M100 #5986: 착수·계획·실패 계약

- **일자**: 2026-08-24 KST
- **브랜치**: `codex/issue-5986-save-protection`
- **기준 commit**: `upstream/devel` `ad2867708`
- **이슈**: [#5986](https://github.com/edwardkim/rhwp/issues/5986)
- **문서 성격**: 작업 뒤 감사 증거로 재구성

이 문서는 Stage 1 당시 작성·승인된 보고서가 아니다. 대화 기록, 최종 diff, 테스트 결과를 대사해 당시의
착수 조건과 실패 계약을 보존한다. 사용자의 `진행해줘`는 구현 착수 요청이었지만 계획서 작성 뒤의 별도
승인은 아니었으며, 그 절차 누락은 그대로 남긴다.

## 착수 조건

- #5986이 열려 있고 담당자가 `postmelee`임을 확인했다.
- 관련 open PR이 없음을 확인했다.
- 전용 브랜치를 최신 `upstream/devel` `ad2867708`에서 만들었다.
- 범위는 Studio의 저장 보호 의도 보존으로 제한하고 embed RPC는 #5987로 분리했다.

## 실패 계약

구현 전 계약 테스트는 다음 간극을 드러냈다.

1. 평문 load와 암호 load가 같은 atomic load 경로를 사용해 성공 뒤 보호 의도를 구분해 commit할 입력이
   없었다.
2. Save As fallback은 download 성공 전에 보호 상태와 파일명을 바꿔, download 실패가 기존 암호 보호
   의도를 잃게 만들 수 있었다.
3. 실제 HWP3/HWP5/HWPX 암호 fixture를 연 뒤 새 문서·release까지 이어지는 상태 수명주기 회귀가 없었다.

## Stage 1 산출 판단

보호 의도를 atomic load의 명시적 입력으로 만들고, 저장 fallback 상태 commit을 성공 뒤로 옮기는 설계로
진행했다. 다만 이 판단과 red test는 구현 전 별도 Git 커밋으로 보존되지 않았고 구현 commit
`bdc90ded9`에 함께 들어갔다. 이는 하이퍼 워터폴 단계 경계 이탈이다.
