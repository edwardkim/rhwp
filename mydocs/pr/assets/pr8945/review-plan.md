# PR #8945 검토 계획

2026-11-11 KST. 작업지시자의 “8945 PR 처리를 합니다”에 따른 검토다.
reviewer `edwardkim` 지정·API 확인을 완료했다. 공개 댓글·승인 리뷰·push·merge는 별도 승인 대상이다.

- 역할: maintainer_general. 보조: intake_and_review, review_template, local_validation,
  visual_fixture_evidence, post_merge, visual_verification_governance, visual_sweep_guide.
- 원 PR: sacru2red/rhwp, `06f47c8a518f542a80d38e5a398ed73fed7dcc64`.
- 최신 base: `99b9d7ec5ca36884ea8fd981ac815c1edc34c653`. 개별 merge-tree exit 0.
- 원 head CI `42991534935`와 Render Diff `42991534852`, CodeQL·Adapter·Proptest·정책 감사가 성공했다.
- #7620은 #7644 → #7681(`4c7092397e6fb6d1d16e5521bb9049ef25814d5a`)의 부분 개선 뒤
  남은 바닥 예산·footer·CellBreak 범위가 있다. 이번 PR은 바닥 예산만 다루며 `Refs`를 유지한다.
- #8149의 빈 host 첫 조각 예약과 #7147의 선언 높이/첫 조각 계약을 기존 구현·회귀와 함께 추적한다.

## 실행 범위

1. 원 head와 current-base 병합 결과를 구분한다. 두 production 파일의 최종 원점·점유·이월 소비
   경로와 실제 분할 컷/행 소유/후속 본문 보존을 검토한다. 원 기여 코드를 보정하지 않는다.
2. 같은 최신 base와 후보에서 Native CLI·최적화된 fresh WASM producer를 순차 빌드하고 source·도구·해시를 남긴다.
   기본 작업공간 `/home/edward/mygithub/rhwp/target/pr-review`를 공용 target으로 쓴다.
3. 새 b·c·d HWP/PDF(각 2쪽), 원래 a·None 대조군을 같은 글꼴·96dpi·print·compat 2022로
   전쪽 비교한다. 본문 바닥/새 쪽 표 시작/뒤 문단을 직접 확대 판독한다. 합성 입력과 독립 한컴독스 Print를 구분한다.
4. 최소 글꼴 환경에서 신규 3건과 원 #7620·#8149 관련 focused 검사를 실행한다. 새/기존 회귀의
   입력 계약과 기대값이 source 또는 독립 Print와 맞는지 대조한다.
5. 편람 383쪽과 정본 보호 TSV 3,542행의 실제 Native/WASM 검증을 수행한다. 원본 글꼴은 로컬에서만
   공급하고 보호값·PDF·측정 기준을 바꾸지 않는다. 불일치가 있으면 원인과 해제 조건을 기록한다.
6. 정확한 원 code head CI가 녹색이고 maintainer source/test/fixture 보정이 없으므로 정본 3.2.2에
   따라 이미 성공한 전체 Rust·Native Skia 회귀는 중복 실행하지 않는다. focused·실제 시각 검증은 별도 수행한다.
7. 실패·미검증이 있으면 원인·반례·필요 보정을 구체화한 후 로컬 최종 판정을 기록한다.
   필수 검증을 통과한 경우에만 원격 처리안을 준비하며 승인 없이 댓글·push·merge하지 않는다.

임시 산출물은 기본 작업공간 `output/pr-review/pr8945/`에 둔다. 정식 증적은 검토 문서와 asset에
보존한다. 기본 작업공간과 다른 PR·WIP worktree는 변경·삭제하지 않는다.
