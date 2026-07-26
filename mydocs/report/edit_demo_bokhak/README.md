---
kind: guide
status: active
canonical: mydocs/report/edit_demo_bokhak/README.md
last_verified: 2026-07-26
---

# 실제 CLI 편집 작동 사례 — 실물 대학 서식 채우기

> 여정: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md) 계열 — 실사례 여정을 CLI로 끝까지 실행.
> 대상: `samples/복학원서.hwp` (고려대학교 복학원서, 학부).

## 실제 사람 작업

대중이 HWP를 쓰는 가장 흔한 용도는 **문서 편집·서식 채우기**다. 이 데모는 그 용도를 CLI 만으로 재현한다 — 실제 대학 서식을 열고, 표 격자 좌표로 빈 칸을 찾아, 완전 가상 데이터로 채우고, 렌더해 결과를 확인한다.

## 원본 대비 흐름

![원본 → CLI로 채운 문서](bokhak-before-after.png)

- 워터마크(校印)·표 테두리·직인란·"복학원서 접수증" 구역이 전부 그대로 보존된다.
- **지원자가 채울 칸(대학·학과·학번·성명·휴대전화·이메일·주소·서명)만 정확히 채워지고**, 접수기관 전용 구역("접수자", "복학원서 접수증")은 원본 그대로 비어 있다 — 채울 대상과 채우면 안 되는 대상을 CLI 사용자가 표 구조로 구분해야 함을 보여준다.
- 완전 가상 데이터((주)가 아닌 개인 정보 전부 임의값, 학번 20241234·이메일 gasang@example.com 등)이며 **실제 접수는 하지 않는다.**

## 재현

```bash
# 1) 구조 파악 — 표 개수·행렬·기존 텍스트
rhwp export-tables --json samples/복학원서.hwp

# 2) 표 격자 좌표로 빈 칸을 하나씩 채움 (재현 가능한 최소 단위)
rhwp edit set-cell 복학원서.hwp --table 0 --row 1 --col 1 --text "가상공과대학" -o out.hwp --json
rhwp edit set-cell out.hwp     --table 0 --row 1 --col 3 --text "컴퓨터공학과" -o out.hwp --json
# … (총 8칸: 대학·학과·학번·성명·휴대전화·이메일·주소·서명)

# 3) 재독으로 기록값 대조 (기계 판정) — set-cell 결과 JSON의 oldText 로
#    "원래 빈 칸이었는지"를 매 호출마다 확인해 기존 문구 훼손을 방지

# 4) 실제 렌더로 최종 확인
rhwp export-svg out.hwp -o rendered
```

## 발견한 실수와 교정 (정직하게 남김)

첫 시도에서 표2(접수증 구역)의 (2,0) 셀을 "빈 칸"으로 오인해 `set-cell` 로 덮어썼는데, 실제로는 **기존 법정 안내문구**("복학원서를 접수함", "군필자는 예비군 전입신고..." 등)가 들어있는 셀이었다. `set-cell` 결과 JSON의 `oldText` 필드를 보고 즉시 발견해(비어있지 않은 원문이 응답에 찍힘) 원본으로 재작업, 접수증 구역은 손대지 않기로 정정했다.

이건 CLI 버그가 아니라 **사용자(에이전트) 판단 오류**였지만, 실제로 서식을 다루는 에이전트가 마주치는 위험 그 자체를 보여준다 — `set-cell` 이 매 호출마다 `oldText` 를 돌려주는 계약 덕분에, 훼손이 있었는지 즉시 기계로 확인할 수 있었다. 이 계약이 없었다면 조용히 법정 문구가 사라졌을 것이다.

## 관련

- 명령 계약: [CLI 명령어 매뉴얼](../../manual/cli_commands.md) §edit set-cell
- 방법론: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md)
