# V-bon — Best-of-N outcome ranking

LLM-as-verifier axis 4 (`closes #5489`). Rank N final candidate outputs
from existing `dry-run` / `--verify` / `ir-diff` envelopes.

Ranking key (lower is better):

1. `invalid` unset/empty beats set
2. `exitClass` 0 > 3 > 4 > 1 > 2
3. `verify.identical` true > missing > false
4. `|changedCount - intendedChangedCount|`, then `changedCount`
5. `candidateId` (stable)

There is no prose score. `process_steps` is V-step (`#5490`) and is refused.

```text
python test_rank.py
python generate_corpus.py
python -m tools.llm_verifier.best_of_n --check-corpus
```

Run the last command from the repository root, or:

```text
python test_corpus.py
```
