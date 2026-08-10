---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 102 — 정책연구 p120→p121 본문 소유 경계

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `e11aee33d`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 직접 비교: `/tmp/rhwp-stage102-policy-p120-p121/`

작업트리의 Stage 96·98 #4138 조사 변경과 `table_layout.rs` 진단 출력은 다른 진행
변경이다. 이 stage에서는 편집·stage·되돌리기하지 않는다.

## 재현과 PDF 판정

`fidelity_compare.py` 직접 쌍 비교에서 p120→p121에
`rhwp_earlier_than_reference` 61자가 검출됐다. PDF p120은 source `pi=1293`의
첫 4줄까지 배치하고 `A) 기증자가 …`로 시작하는 뒤쪽 줄은 p121에서 시작한다.
현재 rhwp는 같은 문단의 6줄을 p120에 배치해 reset 뒤 2줄을 한 쪽 일찍 소유한다.

`dump -s 0 -p 1293`의 저장 `LINE_SEG`는 다음 물리 쪽 경계를 명시한다.

```text
ls[0] vpos=60556
ls[1] vpos=62556
ls[2] vpos=64556
ls[3] vpos=66556
ls[4] vpos=0       <- 다음 물리 쪽
```

줄 수는 composer와 저장 `LINE_SEG`가 14:14로 일치한다. reset 직전 줄의 visible
bottom은 `67556HU`, 본문 높이는 약 `71716HU`라 저장상 본문 하단 94% 지점이다.
반면 현재 pagination은 일반 native HWP5 본문의 중간 reset을 각주 겹침 등 일부
특수 조건에서만 강제해 `pi=1293 lines=0..6`을 p120에 남긴다.

## 수정 계약

이 문단의 각주 158은 해당 쪽의 첫 각주이며 marker가 reset 전 line 1에 있다. 기존
`native_hwp5_first_footnote_overlap_break_line`은 저장 reset 직전 줄의 raw
`footnote_top` 침범만 인정해, marker와 reset 사이에 일반 본문이 있는 형상을 놓쳤다.
첫 각주 문단에 대해 다음 증거가 모두 맞을 때만 저장 reset을 물리 쪽 경계로 인정한다.

1. composer 줄과 저장 `LINE_SEG`가 일대일이다.
2. 현재 page에 아직 다른 각주가 없고 문단 control은 단일 각주다.
3. 각주 marker가 저장 reset 이전 prefix에 있다.
4. renderer와 같은 flow advance에서 reset 직전 줄은 projected FootnoteArea 안에
   들어가고 reset 다음 줄은 넘는다.
5. projected available은 실제 각주 높이, footer-band 회수, 각주 안전영역과 다른
   배타영역을 모두 반영한다.

이 조건은 저장 좌표를 무조건 따르는 옵션이 아니다. 페이지 중간 rewind와 stale
`LINE_SEG`, 표/그림 분할은 기존 경로에 남긴다. 회귀는 실물 p120/p121에서
`pi=1293 lines=0..4` / `lines=4..14` 소유를 고정하고, 전체 release-test와 PDF
직접 비교로 광역 영향을 확인한다.

## 진행 상태

- [x] PDF/rhwp p120·p121 직접 판정
- [x] source `pi=1293` 저장 reset과 현재 owner 확정
- [x] native HWP5 첫 각주 projected boundary 조건 구현
- [x] focused 회귀 28/28 통과
- [x] p120·p121 직접 비교 재생성
- [x] stage focused gate 통과 후 커밋
- [ ] 전체 release-test·clippy·Skia는 모든 잔여 stage 뒤 최종 PR gate에서 수행

## 결과

- p120 `pi=1293`: `lines=0..4`, 각주 158 소유
- p121 `pi=1293`: `lines=4..14`, 각주 158 미소유
- 문서 페이지 수: 215쪽 유지
- 직접 비교: `/tmp/rhwp-stage102-policy-p120-p121-after/`
- p120→p121 text owner/page-boundary 후보: 0건
- text 차이: p120 조합문자 표기 차이 1건만 남고 p121은 0/0
- pixel diff: p120 14.08% → 13.53%, p121 17.81% → 15.80%

PDF/rhwp review에서 p120은 모두 `…규정하고 있음.`으로 끝나고 각주 158을 가지며,
p121은 모두 `A) 기증자가…`에서 시작해 각주 159·160을 가진다. 이 stage의 본문·각주
소유 결함은 해소됐다. 다음 stage는 같은 자동 원장의 p129→p130 후보를 현재 head에서
재판정한다.
