# 00 — 한컴 기준 PDF 대조 판단 트리

이 장은 에이전트가 **어느 명령을 먼저 칠지**만 고른다. 사다리는 강제
순회가 아니다. 질문이 이미 답이면 멈춘다 (F16).

gym 경로가 아니다. 새 CLI 도 없다. 아래 상자는
`tools/fidelity_compare/README.md` 가 이미 고정한 호출이다.

```
독립 한컴 PDF 가 기록되어 있는가
  │
  ├─ 아니오 ──▶ rhwp-visual-regression (render-diff). F01
  │
  ├─ 예, 글자 후보만
  │     venv/bin/python tools/fidelity_compare/fidelity_compare.py \
  │         <키> <시작> <끝> --text-only --export-all-svg [--layout-ledger] \
  │         --out-dir /tmp/rhwp-fidelity-<키>
  │     또는 Windows: venv\Scripts\python.exe ...
  │     산출: text-report.tsv, page-count-ledger.tsv, provenance.tsv
  │     정지 F02 — 후보. 확정 아님
  │
  ├─ 예, 쪽 시트까지
  │     같은 호출에서 --text-only 를 뺀다. Chrome 필수.
  │     Chrome 없으면 F10.
  │     산출: cmp-pNNN.png, report.tsv (최악 쪽 우선)
  │     정지 F03 — 상위 쪽을 눈으로
  │
  └─ 시트/원장을 본 뒤
        두부? F14 하네스 오염부터
        쪽수 차이? F11 후보
        암호화? F13 정지
        그 외 실질 차이만 이슈. 최종 판정은 유지자 F05
```

## 축을 고르는 한 줄

| 관찰 | 축 |
| --- | --- |
| 한컴이 뽑은 PDF 와 rhwp 가 같은가 | 이 스킬 · fidelity_compare |
| 편집 전후 / 포맷 왕복이 레이아웃을 깨나 | `rhwp-visual-regression` · render-diff |
| 원인 미확정 실사용 결함 | bug-hunter 여정 (원장은 여기가 제공) |
| gym 점수 | 거절 F06 |

같은 문서에 공식 PDF 와 편집 전후 HWP 가 같이 있어도 축을 섞어 한
문장으로 판정하지 않는다. "render-diff PASS 이니 한컴과 같다"는
거짓이다. 자기 일관성과 한컴 기준은 다른 측정이다.

## 명령 상자 (발명 금지)

살아 있는 호출은 이것이다.

1. `venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작> <끝>`
2. 같은 파일의 `--text-only` / `--export-all-svg` / `--layout-ledger`
3. direct pair: `--source` `--reference-pdf` `--label` [`--reference-grade`]
4. `rhwp export-svg` (하네스가 내부에서 호출. 에이전트가 새 플래그를 만들지 않음)
5. `rhwp export-render-tree` (`--layout-ledger` 가 내부에서 호출)

없는 것: `rhwp fidelity-diff`, `rhwp pdf-compare`, `rhwp hangul-diff`,
`rhwp oracle-diff`, gym 렌더 러너. 오타 난 하위명령은 만들지 말고
거절한다 (F06).

Windows 에서는 1 의 인터프리터만 `venv\Scripts\python.exe` 로 바꾼다.
`--break-system-packages` 는 없다 (F15).

등록 키는 `plan` `manual` `bunjang` `korexam` `math` `eng` 여섯이다.
키 오타는 하네스가 `등록되지 않은 문서 키` 로 exit 한다. 새 키를
스키마에 몰래 추가하지 말고, 없으면 direct pair 를 쓴다.

## 원본 불변

하네스는 입력 HWP/PDF 를 덮어쓰지 않는다. `--out-dir` 에 시트와 TSV 만
쓴다. 원본을 비교 과정에서 저장하거나 sanitize 하지 않는다 (F18).
산출 경로가 worktree 안이면 PNG 가 디스크를 채운다. `/tmp` 또는
`%TEMP%` 를 기본으로 권한다.

## 에이전트가 하지 말 것

- 공식 PDF 없이 "한컴과 같다"고 말하기
- `samples/` 옆 동반 PDF 를 출처 확인 없이 오라클로 승격 (F17)
- diff% 0 을 merge 근거로 쓰기
- 두부 시트를 문서 회귀로 이슈화
- visual-regression / bug-hunter 스킬을 이 폴더에서 고치기
- gym/ 아래에 과제를 만들기
- `export-svg` 플래그를 이 스킬이 발명한 것처럼 문서화하기

다음: 독립 PDF 가 없으면 [01_when_to_use.md](01_when_to_use.md),
있으면 [02_setup_venv.md](02_setup_venv.md).
