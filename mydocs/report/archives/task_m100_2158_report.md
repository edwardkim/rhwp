# 최종 보고서 — Task M100 #2158: HWPX lineseg 재계산의 저장 쪽-상대 vpos 리셋 보존

- 이슈: #2158 / 브랜치: `fix/2158-hwpx-vpos-reset-preserve` / 작성일: 2026-07-10
- 계획서: `mydocs/plans/task_m100_2158.md`

## 원인

HWPX 로딩(`reflow_zero_height_paragraphs`)은 lineSegArray 부재 문단이 있으면 구역
전체 vpos를 누적 좌표로 재계산하는데, 이때 **원본(비합성) lineseg 문단의 저장
쪽-상대 vpos 리셋(쪽나눔 인코딩)**까지 덮어썼다 (sample16 pi88: 저장 568 →
208008 변조). typeset의 vpos-reset 쪽나눔(#321/#1921)이 무력화되어 동일 문서가
HWP5로는 64쪽(한글 정합), HWPX로는 63쪽.

## 수정 (`src/document_core/commands/document.rs`)

기존 #1920 예외(쪽 하단 고정 틀 host + 저장 vpos==0 한정) 뒤에 일반화 분기 추가 —
원본 lineseg 문단의 저장 first vpos가:

- 직전 저장 vpos > 60000HU (한 쪽 분량, #1921 near-top 임계 동일)
- **0 < first < 5000HU** (sb 반영 양수 쪽 상단 좌표)

이면 `running_vpos = first`로 리셋 신호를 재계산 좌표계에 보존.

**first==0 제외가 핵심 게이트**: mid-doc vpos=0은 생성기 노이즈
(task1749 pi2/27/47 실측 — 포함 시 HWP 참조 컷(end_cut=[3]) 회귀)라 기존 #1920
틀-host 규칙에만 맡긴다. 정당한 텍스트 쪽나눔 리셋은 양수 좌표(568=sb)로 저장됨.

## 검증

| 문서 | 수정 전 (#2154 스윕) | 수정 후 (오라클 재검) |
|---|---|---|
| hwp3-sample16-hwp5.hwpx | PAGE_DELTA 63vs64, n_mm 808 | **쪽수 64=64 회복** (pi88 vpos 568 보존, HWP5·한글 삼자 일치), n_mm 808→**109** 잔존(산발, 별도 서브축) |
| [2027] 온새미로 1 본교재.hwpx | PAGE_DELTA 46vs47, n_mm 59 | **완전 MATCH 47=47, n_mm 0** |
| 80168/issue1891 hwpx | 157 vs 158 | 불변 — 본 기제 아님 (#1921 계열 잔존) |
| hwp3-sample10-hwpx.hwpx | 764 vs 763 | 불변 — 본 기제 아님 (#2151 변환본 축 잔존) |
| task1749 픽스처 (노이즈 반례) | — | end_cut=[3] HWP 참조 유지 (first>0 게이트) |

- 오라클 재검: `output/poc/task_pipage_sweep/hwpx_vpos_retest.tsv`
- 게이트: `cargo test --release` 전체 실패 0 / clippy 0 / rustfmt 0
- 핀 테스트: `tests/issue_2158_hwpx_vpos_reset_preserve.rs`
  (sample16 hwpx=hwp5=64 삼자 핀, 온새미로 hwpx 47)
