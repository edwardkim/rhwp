# rustfmt `newline_style = Unix`

저장소 루트 `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
newline_style = "Unix"
use_small_heuristics = "Default"
```

모든 `.rs` 는 LF 다. CRLF 가 들어가면 `cargo fmt --all -- --check` 가 실패한다.

## Windows 함정

`core.autocrlf=true` 이면 checkout 이 CRLF 로 바뀌고, rustfmt 가 Unix 를
기대해 게이트가 빨개진다.

```bash
git config --get core.autocrlf
git config --get core.eol
```

이 워크트리에서는 `core.autocrlf=false` 를 전제로 한다. 전역이 true 면
이 저장소에만 끈다.

```bash
git config --local core.autocrlf false
git config --local core.eol lf
```

에디터·생성기가 `\r\n` 으로 파일을 쓰지 않게 한다. Python 생성기는
`newline="\n"` 으로 저장한다.

## 확인

```bash
cargo fmt --all -- --check
```

실패 목록에 "CRLF" / 줄끝 차이가 보이면 이 절이다. 기능을 고치지 말고
줄끝만 고친 커밋을 섞지 않는다 — 같은 커밋에서 생성기를 LF 로 고친다.

예제: [19_windows_autocrlf_unix.md](../examples/19_windows_autocrlf_unix.md).
