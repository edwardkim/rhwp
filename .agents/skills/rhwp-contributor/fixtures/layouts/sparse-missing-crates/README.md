# 스파스 체크아웃에 crates/ 가 없는 레이아웃

본진 sparse 규칙이 새 워크트리에 상속되면 `crates/` 가 빠질 수 있다.

닫는 명령:

```
git sparse-checkout add crates
cargo fmt --all -- --check
```

`crates/` 가 생긴 뒤에는 HARD GATE 가 반드시 통과해야 한다.
이 폴더는 그 상태를 설명하는 픽스처이며 실제 crate 소스를 복제하지 않는다.
