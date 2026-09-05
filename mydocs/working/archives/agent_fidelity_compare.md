# #5329 실 에이전트 한컴 기준 PDF 대조(fidelity_compare) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5329
브랜치: `feat/agent-fidelity-compare` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-fidelity-compare/` ·
`tests/agent_fidelity_compare_skill_contract.rs` ·
`scripts/tests/test_agent_fidelity_compare.py` ·
`Cargo.toml` 의 해당 `[[test]]` 한 블록 · 본 문서
비범위: `gym/` · `rhwp-visual-regression` · bug-hunter · 다른 에이전트 스킬 ·
새 CLI · DocumentCore 편집/렌더 구현 · `tools/fidelity_compare` 본문

## 무엇을

에이전트가 **한컴이 내보낸 공식 PDF** 와 `rhwp export-svg` 를 쪽별로
대조할 때, 이미 있는 `tools/fidelity_compare` 하네스를 스킬로 닫는다.

코어는 이미 있다.

- 페이지 시트 `cmp-pNNN.png`
- 픽셀 diff% 랭킹 (`report.tsv`, 최악 쪽 우선) — **후보 검출**
- `text-report.tsv` PDF 텍스트층 vs SVG `<text>` 쪽별 멀티셋
  (소실 / 과잉 / 치환)
- `--text-only` (Chrome·pypdfium2 불필요, pypdf 만)
- `--font-style` 기본, 로컬 face 별칭, 두부 오염 방지
- `RHWP_FONT_PATH_DIR`
- `provenance.tsv` (원본·오라클 경로·등급)
- 최종 시각 판정은 유지자
  (`visual_verification_governance.md`)

이 작업은 새 비교 CLI 를 만들지 않는다. 스킬·레퍼런스·예제·픽스처·
계약 시험만 추가한다.

## 왜

이슈 본문: 에이전트가 한컴 공식 출력 PDF 와 rhwp export-svg 를 쪽별로
대조하려면 하네스가 스킬로 닫혀야 한다. visual-regression(자기 일관성
render-diff) · bug-hunter(여정 방법론) 와 겹치지 않는 **한컴 기준 대조**
축이다. gym 금지.

독립 한컴 PDF 가 없으면 이 하네스는 정직하지 않다. 그때는
`rhwp-visual-regression` 으로 인계한다. 그 스킬을 여기서 재작성하지
않는다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-fidelity-compare` 에
   `feat/agent-fidelity-compare` 를 `upstream/devel` 에서 분기.
   `rhwp` · `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` ·
   `rhwp-doc-repro` 는 쓰지 않음. 디스크 부족으로 sparse checkout
   (samples/ · mydocs/pr/ · src/ 제외). 이름 있는 남의 worktree 를
   훔치지 않음.
2. SKILL.md 를 사다리·정지 규칙·인계 인덱스로 신설.
3. `references/` 28장: 언제 쓰는지, venv, Windows,
   시트, 랭킹, 텍스트 원장, 글꼴, provenance, 유지자 판정,
   Chrome/venv/쪽수/암호/두부 예외, 등록 키, direct pair, 산출,
   이웃 축, 여정, 함정, 트레이스, 인계, 예외 카탈로그.
4. `examples/` 22건: 실 레시피와 전사.
5. `_gen_pack.py` 가 `fixtures/` 에 JSON·TSV·트레이스·전사 방출.
   여정 80+, 발화 30+, 트레이스 30.
6. `scripts/tests/test_agent_fidelity_compare.py` 가 발명 명령·gym·
   이웃 스킬 재작성·픽스처 스키마를 바이너리 없이 검사.
7. `tests/agent_fidelity_compare_skill_contract.rs` 가 같은 가드.
   하네스 실주행은 CI 에 Chrome/공식 PDF 가 없을 수 있어 필수로
   두지 않음. 렌더 구현을 바꾸지 않음.

## 하지 않은 것

- `tools/fidelity_compare/fidelity_compare.py` 구현 변경
- `rhwp export-svg` / DocumentCore / renderer 변경
- 새 CLI 플래그 / `fidelity-diff` 발명
- gym pack / 과제 / 채점기
- `rhwp-visual-regression` · bug-hunter · 다른 스킬 수정
- 암호화 PDF 우회 도구
- `--break-system-packages` 우회 문서화

## 언제 쓰는가 / 언제 render-diff 가 정직한가

| 입력 | 정직한 축 |
| --- | --- |
| 한컴 도구·버전·경로가 기록된 공식 PDF + 원본 | 이 스킬 |
| 편집 전후 HWP 만 | visual-regression (`render-diff`) |
| 같은 파일 두 번 | `render-diff A A` (여기 아님) |
| 원인 미확정 실사용 여정 | bug-hunter (원장은 여기가 제공) |

`rhwp export-pdf` 산출을 한컴 오라클로 바꿔 치우지 않는다.
`samples/` 동반 PDF 는 참고 등급이다.

## Windows · venv

- POSIX: `venv/bin/python`
- Windows: `venv\Scripts\python.exe`
- `python3.12 -m venv venv` 후 `pip install pypdf pypdfium2 pillow`
- `--break-system-packages` 금지
- `--text-only` 는 pypdf 만, 시트 모드는 Chrome + pypdfium2 + pillow

## 후보이지 판결이 아니다

`report.tsv` 는 최악 쪽 우선 순위다. math 실측 6~11% 가 보여 주듯
절대값은 자간 프린지와 구조 붕괴를 구분하지 못한다.
`text-report.tsv` 는 NFC 멀티셋이라 순서·좌표를 모른다.
최종 시각 판정은 유지자가 거버넌스를 따라 내린다.

## 예외 경로

| 예외 | 정지 | 처방 |
| --- | --- | --- |
| Chrome 없음 | F10 | `CHROME_BIN` 또는 `--text-only` |
| venv 없음 | F09 | 저장소 venv. 시스템 pip 금지 |
| 쪽수 불일치 | F11 | ledger 후보. 전역 page-break 패치 금지 |
| 암호화 PDF | F13 | 정지. 우회 CLI 금지 |
| 두부 가득 시트 | F14 | 하네스 오염. 글꼴 후 재실행 |

## 검증

```bash
# 파일 계약 (바이너리 불필요)
python -m unittest scripts/tests/test_agent_fidelity_compare.py

# Rust 계약 (워크스페이스가 완전할 때)
cargo test --test agent_fidelity_compare_skill_contract -- --nocapture

# 포맷 게이트
cargo fmt --all -- --check
```

하네스 실주행은 글꼴·Chrome·공식 PDF 가 있는 머신에서만:

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --text-only --out-dir /tmp/rhwp-fidelity-smoke
```

이 PR 은 도구/문서 전용이라 거버넌스상 시각 전수가 필수가 아니다.

## 정본

- `tools/fidelity_compare/README.md`
- `mydocs/manual/verification/visual_verification_governance.md`
- `mydocs/manual/verification/hangul_pdf_baseline.md` (맞춰찍기 PageCount)
- 이슈 #5329
