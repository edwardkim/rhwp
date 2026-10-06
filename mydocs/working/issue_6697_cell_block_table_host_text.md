---
kind: working
status: active
issue: 6697
---

# 셀 문단이 블록 표를 앵커하면 그 문단 글자가 통째로 사라진다 (#6697)

작업 브랜치: `fix/6697-cell-block-table-host-text`
대상: `src/renderer/layout/table_layout.rs`

## 한 줄

표 칸 안 문단이 **블록 표(`treat_as_char=false`)** 를 품으면 `layout_composed_paragraph`
호출 자체를 건너뛰어, 그 문단의 **자기 글자**가 어느 쪽에도 그려지지 않는다.

## 이슈가 요구한 것

- 호스트 문단의 글자를 되살린다.
- 표(자리차지) 배치와 흐름 전진(`para_y`) 계약은 건드리지 않는다 — 그건 다른 축이다.

## 원인

`table_layout.rs` 의 셀 문단 루프:

```rust
if !has_block_table_ctrl {
    para_y = self.layout_composed_paragraph(...);   // 글자를 그린다
} else {
    // has_table_ctrl: 표가 포함된 문단
    // LINE_SEG vpos가 문단 위치를 정확히 지정하므로 …
}
```

`Task #573` 이 **인라인 TAC 표**(`treat_as_char=true`)에서 같은 결함
("셀 paragraph 의 surrounding text 가 layout_composed_paragraph 미진입으로 미렌더")을
고쳤고, **블록 표** 쪽은 "텍스트 흐름 외부"라는 이유로 ELSE 분기에 남았다.
대다수 호스트 문단은 글자가 없어(`txt=""`) 결함이 드러나지 않았다.

진단(임시 프로브, 커밋 안 함):

```
[6697] cp=40 block_tbl=true table_ctrl=true para_y=958.8 txt="<향후 10년간 폐농업용 지게차 해체 수익 계산>"
[6697] cp=36 block_tbl=true table_ctrl=true para_y=510.1 txt=""
```

## 만진 경로 / 만지지 않은 경로

- 만짐: `src/renderer/layout/table_layout.rs` ELSE 분기 1곳(+49줄)
- 안 만짐: `para_y` 전진(반환값 폐기), 표 배치, 파서, 직렬화기, CLI

`para_y` 를 전진시키는 변형도 시험했으나 `80550` 30쪽의 기존 표 넘침이
15.67px → 21.09px 로 악화되고 새 overlap 1건이 생겨 채택하지 않았다.

## 검증 실측 (오라클 = 저장 버전과 같은 한/글)

`origin/devel b6b9384ed` 기준. 같은 커밋에서 수정 전/후 바이너리를 각각 빌드해 비교.

| 문서 | 저장 | 오라클 | 수정 전 미출력 | 수정 후 |
| --- | --- | --- | ---: | ---: |
| `80550_(규제영향분석서) 농업기계화 촉진법…hwpx` | 2020 | 2020 `11,0,0,1623` | 1줄 21자 | **0** |
| `36296324_결재문서본문.hwpx` | 2020 | 2020 | 1줄 | **0** |
| `2803097_[별표 4] 장애인전용주차구역…hwpx` | 2020 | 2020 | 1줄 | **0** |
| `36359907_3. 다급(점검) 제출서류.hwpx` | 2020 | 2020 | 1줄 | **0** |

회귀:

- 트리거 형상(칸 문단이 글자+블록 표) 코퍼스 18문서 → 텍스트 산출 변화 **3문서뿐**,
  전부 **한 줄 추가·삭제 0**이고 그 줄이 오라클에 있다.
- 무작위 44문서 텍스트 해시 **전건 동일**.
- `layout-anomaly` 9문서(80550·81240·156745900·edu2022·edu2023·2181727·2744465·30307·156658611)
  전건 동일(overflow/off-canvas/overlap/text-overlap 수치 불변).

## 시험 명령

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --profile release-test -j 4 --lib renderer::layout::table_layout
rhwp export-text  <80550.hwpx> -o out     # '<향후 … 해체 수익 계산>' 출력 확인
rhwp layout-anomaly <80550.hwpx>          # 수치 불변 확인
```

## 남는 것

`80550` 30쪽에서는 되살아난 캡션이 그 문단이 앵커한 13×7 표의 머리행과 겹쳐 보인다.
한/글은 그 표를 31쪽으로 넘기는데 rhwp 는 30쪽에 두기 때문이며, **표 배치·쪽 넘김 축**의
기존 결함이다(이 PR 범위 밖). 표 배치가 정상인 나머지 3문서에서는 캡션이 오라클과 같은
자리에 깨끗이 들어간다.

## PR 메모

`closes #6697`, base `devel`, `--body-file`.
