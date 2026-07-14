## Critic Final

- [Polish] T-RPC-08 expected가 blocker를 `{code,path,preserved:false}`로 표기하지만 required wire contract는 `{ code, xmlPath, message, preserved: false }`다. 테스트가 축약 표기를 그대로 구현하지 않도록 expected를 정확한 wire field 이름과 `message` 포함 형태로 맞추면 된다.

남은 Blocker 0 / Gap 0.
