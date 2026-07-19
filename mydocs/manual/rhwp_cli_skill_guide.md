---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-07-16
---

# rhwp-cli Skill 사용 가이드 (사람용)

Claude Code 에서 `rhwp-cli` skill 로 HWP/HWPX 문서를 분석·내보내기·디버깅하는 방법을 예제와
함께 정리한다.

- **이 문서**: 사람(메인테이너·기여자)이 읽고 따라하는 가이드.
- **`.claude/skills/rhwp-cli/SKILL.md`**: Claude 가 읽는 트리거·지침(수정 시 동작 변경). 직접 읽을 필요 없음.
- **명령 레퍼런스**: [`cli_commands.md`](cli_commands.md) (전체 옵션).

---

## 1. Skill 이란

`rhwp-cli` 는 Claude Code 가 사용자의 자연어 요청을 적절한 `rhwp` CLI 명령으로 바꿔 실행하도록
돕는 skill 이다. 사용자는 명령·옵션을 외우지 않고 **하고 싶은 일**만 말하면 된다.

```
사용자: "3-09월_교육_통합_2023.hwp 4쪽을 SVG로 내보내줘"
   ↓ (rhwp-cli skill 트리거)
Claude: rhwp export-svg samples/3-09월_교육_통합_2023.hwp -o output/svg -p 3
```

## 2. 호출 방법

세 가지 중 아무거나:

1. **자연어** — "이 hwp 페이지네이션 보여줘", "두 파일 IR 비교해줘" → skill 자동 트리거.
2. **슬래시 명령** — 채팅에 `/rhwp-cli` 입력.
3. **명시 요청** — "rhwp-cli 로 …", "rhwp export-svg 실행해줘".

> skill 은 Claude 가 명령을 대신 실행해 준다. 사람이 터미널에서 직접 쓰려면 아래 4절의 명령을
> 그대로 입력하면 된다(동일하다).

## 3. 무엇을 시킬 수 있나 — 요청 예시

| 이렇게 말하면 | Claude 가 실행 |
|--------------|---------------|
| "sample.hwp 1쪽 SVG로 빼줘" | `export-svg sample.hwp -p 0` |
| "전체를 PNG로, Claude Vision 입력용" | `export-png sample.hwp --vlm-target claude` |
| "PDF로 변환" | `export-pdf sample.hwp -o out.pdf` |
| "PDF 변환 시 한글 fallback 폰트 지정" | `export-pdf sample.hwp -o out.pdf --fallback-serif "Noto Serif CJK KR" --fallback-sans "Noto Sans CJK KR"` |
| "본문 텍스트만 뽑아줘" | `export-text sample.hwp` |
| "5쪽이 어떻게 배치됐는지" | `dump-pages sample.hwp -p 4` |
| "3번 문단 조판 구조" | `dump sample.hwp -s 0 -p 3` |
| "이 파일 한컴 버전이 뭐야" | `info sample.hwp` |
| "a.hwpx 와 b.hwp 가 어디서 다른지" | `ir-diff a.hwpx b.hwp --summary` |
| "4쪽 글자 좌표(bbox) 뽑아줘" | `export-render-tree sample.hwp -p 3` |
| "이 문서 간격 버그 좀 디버깅해줘" | 디버깅 워크플로우(5절) 순서대로 |

## 4. 직접 실행 예제 (터미널)

먼저 빌드(최초 1회 또는 소스 변경 후):
```bash
cargo build --release          # → ./target/release/rhwp
```
이후 `rhwp` 대신 `./target/release/rhwp` 를 쓰거나, 빌드 없이 `cargo run --quiet --bin rhwp -- …`.

### 예제 A — 파일 정보
```bash
$ rhwp info samples/3-09월_교육_통합_2023.hwp
파일: samples/3-09월_교육_통합_2023.hwp
크기: 1423360 bytes
버전: 5.1.1.0
압축: 예
암호화: 아니오
배포용: 아니오
```

### 예제 B — 페이지를 SVG 로 (시각 확인)
```bash
$ rhwp export-svg samples/3-09월_교육_통합_2023.hwp -o output/svg -p 0
  → output/svg/3-09월_교육_통합_2023_001.svg
내보내기 완료: 1개 SVG 파일 → output/svg/
```
- `-p 0` = 1쪽(0부터). 생략하면 전체 페이지.
- `--debug-overlay` 추가 시 문단/표 경계와 인덱스가 겹쳐 그려져 디버깅에 유용.

### 예제 C — 페이지네이션 배치 덤프 (레이아웃 분석)
```bash
$ rhwp dump-pages samples/3-09월_교육_통합_2023.hwp -p 0
=== 페이지 1 (global_idx=0, section=0, page_num=1) ===
  body_area: x=34.0 y=90.7 w=725.7 h=1001.6
  단 0 (items=76, used=650.1px, ...)
    FullParagraph  pi=0  h=21.3 (sb=0.0 lines=21.3 lh=15.2 ls=6.0)  vpos=0  "의 값은?"
    Shape          pi=0 ci=4  수식  vpos=0
    FullParagraph  pi=1  h=33.6 ...  vpos=1594  "① ② ③ "
```
- `pi` = 문단 인덱스, `vpos` = 세로 위치(HWPUNIT), `lh`/`ls` = 줄높이/줄간격.

### 예제 D — PNG (VLM 입력용)
```bash
$ rhwp export-png sample.hwp -p 0 --vlm-target claude
# Claude Vision 한도(1568px)에 맞춰 자동 스케일. gpt4v / gemini / qwen / llava 도 가능.
```

### 예제 E — HWPX ↔ HWP 차이 비교
```bash
$ rhwp ir-diff sample.hwpx sample.hwp --summary       # 카테고리별 차이 건수
$ rhwp ir-diff sample.hwpx sample.hwp -s 0 -p 45       # 특정 문단만
```

### 예제 F — render tree bbox (정밀 좌표 분석)
```bash
$ rhwp export-render-tree sample.hwp -p 3 -o output/rt
  → output/rt/render_tree_004.json
# JSON: {type, bbox:{x,y,w,h}, children:[...]}  — Page→Line/TextRun/Image/Table…
```

## 5. 레이아웃·간격·겹침 버그 디버깅 (실전 순서)

코드 수정 없이 결함을 좁히는 권장 순서. (이번 프로젝트의 미주/수식/표 셀 버그를 이 순서로 잡았다.)

```bash
# 1) 문제 페이지를 디버그 오버레이로 — 어느 문단/표인지 식별
rhwp export-svg 파일.hwp --debug-overlay -p N -o output/poc/dbg

# 2) 그 페이지 배치 목록 + 높이 — 어디서 vpos/높이가 어긋나는지
rhwp dump-pages 파일.hwp -p N

# 3) 문제 문단의 조판 속성 — ParaShape / LINE_SEG / 표·도형
rhwp dump 파일.hwp -s N -p M

# 4) HWPX↔HWP 불일치면 — 어느 속성이 다른지
rhwp ir-diff 파일.hwpx 파일.hwp -s N -p M

# 5) 정밀 좌표 — 보정 전/후 또는 두 상태 bbox 비교
rhwp export-render-tree 파일.hwp -p N -o output/poc/rt
```

### 보정 전/후 시각 회귀 확인 (중요)
> 전체 테스트·golden 이 통과해도 특정 페이지 시각 회귀는 못 잡을 수 있다. **시각 판정이 최종 게이트.**

```bash
# 기준(보정 전) 상태에서
rhwp export-svg 파일.hwp -o output/poc/before
# 변경(보정 후) 상태에서
rhwp export-svg 파일.hwp -o output/poc/after
# 페이지별 비교 → 변화한 페이지·좌표 이동 확인
for f in output/poc/before/*.svg; do
  diff -q "$f" "output/poc/after/$(basename "$f")" >/dev/null || echo "변화: $(basename "$f")"
done
```
- 셀 내부/transform 그룹은 `translate(x,y)` 단위로 본다(절대 y 아님).
- 더 정밀하게는 `export-render-tree` JSON bbox 를 비교한다.

## 6. HWPX → HWP 저장이 한컴과 다를 때 (고급)

HWPX 로 연 문서를 HWP 로 저장(#178 어댑터)했을 때 한컴이 다르게 여는 경우의 record 분석.

```bash
# oracle = 한컴이 저장한 정답본, generated = rhwp 가 저장한 것
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table
rhwp hwp5-table-probe   oracle.hwp generated.hwp --out-dir output/poc/probe
rhwp hwp5-anchor-trace  파일.hwp --needle "찾을텍스트" --section 0
```
전체 hwp5-* 도구는 [cli_commands.md §4](cli_commands.md).

## 7. 자주 헷갈리는 점

- **페이지 번호는 0부터.** `-p 3` = 4쪽. PDF/한컴(1부터)과 다르다.
- **출력 폴더는 기본 `output/`** (gitignore). 분석 산출물은 `output/poc/<주제>/` 로 분리 권장.
- **공백 포함 CLI 값은 큰따옴표 권장.** 예: `--font-path "./My Fonts"`,
  `--fallback-sans "Apple SD Gothic Neo"`. 작은따옴표는 zsh/bash/PowerShell에서 literal 값이 필요할 때만 사용한다.
- **export-png/pdf** 는 release 바이너리(또는 `--features native-skia`)에서 동작.
- **Docker 는 WASM 빌드 전용** — 분석 명령에는 쓰지 않는다(로컬 cargo 사용).
- **자기검증 ≠ 한컴 호환**: 저장/렌더 결함은 한컴 2020/2022 에서 직접 열어 확인하는 게 최종 판정.
- 단위: 1인치 = 7200 HWPUNIT = 96px, 1px = 75 HWPUNIT, 1mm ≈ 283.46 HWPUNIT.

## 8. 관련 문서

- 전체 명령·옵션: [`cli_commands.md`](cli_commands.md)
- `dump` 상세: [`dump_command.md`](dump_command.md)
- `export-png` 상세: [`export_png_command.md`](export_png_command.md)
- `ir-diff` 상세: [`ir_diff_command.md`](ir_diff_command.md)
- skill 정의(Claude 용): `.claude/skills/rhwp-cli/SKILL.md`
