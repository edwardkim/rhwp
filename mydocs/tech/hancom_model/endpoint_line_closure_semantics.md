---
kind: reference
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# Hancom Endpoint And Line-Closure Semantics

This document gives a typed, partial semantics for HNC's horizontal line
closure. It formalizes the observed endpoint/line-closure behavior of the
Hancom word processor and is intended as a canonical model of that behavior.
It is not an RHWP implementation specification. Related models in this
directory are the [unified layout model](unified_layout_semantics.md), the
[object placement model](object_placement_semantics.md), and the
[flow/pagination model](flow_pagination_semantics.md); the index is the
[README](README.md).

The model records that some behaviors are supported only by partial evidence,
and no full closure model is yet accepted end-to-end. Accordingly, local
findings below may be tagged `PROVED` without implying that the corresponding
end-to-end stage is presently accepted.

## 1. Evidence Grades

Every assertion has exactly one grade.

| Grade | Meaning |
| --- | --- |
| `PROVED` | Behavior inputs, outputs, and committed descriptors were causally joined and restored in the observed evidence named by the reference. |
| `STATIC` | Exact control/data flow of the behavior is established, but semantic names beyond the structural relation are not assigned. |
| `OBSERVED` | A value or behavior occurred, but the available evidence does not discriminate its general meaning. |
| `HYPOTHESIS` | A model compatible with current evidence and awaiting a stated proof obligation. |

`PROVED` is local to the stated lemma. It is not transitive through an
unaccepted dependency. Evidence locates a behavior; it does not assign a
semantic type by itself.

## 2. Typed State Space

Let `U16 = {0,...,65535}` and `Byte = {0,...,255}`. Source text is modeled
as UTF-16 code units, not Unicode scalar values:

```text
Paragraph   = { text : U16*, length : Nat, identity : ParagraphId }
Position(T) = { i : Nat | 0 <= i <= |T| }
Cursor(T)   = Position(T) x Byte
Endpoint(T) = Cursor(T)
Interval(T) = { [a,b) | a,b in Position(T), a <= b }
```

For `c = (i,q)`, write `pos(c)=i` and `state(c)=q`. The byte is an address
mode, not a geometric scalar. `q=0` addresses the current text node locally.
`q=1` interprets `i` over a flattened logical chain: the accessor follows the
forward node-successor link while the next node's continuation bit marks a
continuation and subtracts each preceding node's length. The inverse conversion
follows the node-predecessor link backward and adds preceding lengths. The
accessor treats every value other than `1` as node-local, although the
constructors explicitly produce only `0` and `1`. A source position is a
boundary between UTF-16 units. Therefore a surrogate pair, combining sequence,
or control unit may occupy several units and several positions without creating
a legal internal endpoint.

The canonicalizer converts `(root,i,1)` into `(resolved_node,local_i,0)`. The
UTF-16 unit identity is invariant under this full tuple conversion; mode itself
therefore contributes no width and does not define a separate geometry policy.

A layout-side node-local occurrence is observed directly: a single-node
`Active-Active` paragraph generated 223 mode-`0` character accesses over
positions `0..210`. Because that node had no eligible continuation successor,
the run establishes address provenance but cannot exercise the cross-node
identity invariant. The exact outer caller is not yet dynamically proved; the
high-confidence static match is the interval-scan formatter caller, because it
preserves the cached header mode across an interval scan.

The chain-membership producer is closed. The continuation writer sets a
successor's continuation flag bit only when the anchor query resolves either a
Note anchor or a HeaderFooter anchor to a tracked-change entry whose hide field
is nonzero. The gate runs as a member of the track-change manager. This makes
the mode-1 chain a hidden-revision anchor-continuation coordinate domain rather
than a visual-wrap domain.

The writer is geometry-adjacent in the strongest static sense: it executes in
the per-line formatter, after a fresh hide recomputation and immediately before
the line-record constructor that creates line records and commits
endpoint/available-width state. The continuation bit therefore changes which
physical-node source span participates in line geometry, while remaining only
an address-domain marker rather than a width.

The hidden state is view-derived, not an intrinsic anchor geometry field. In
the track-change manager, a dedicated flags word carries an original/final
polarity bit: clear hides Delete (final-content view), while set hides Insert
(original-content view). A separate bit is modification-password/protection
state and does not request layout invalidation by itself. The action
dispatcher proves the remaining reviewer/display flags: one bit set means plain
Original/Final without changes and memo, another means non-inline/balloon
presentation, another enables format/shape changes, and another enables
insert/delete changes. Author `mark == 1` means selected/included in the
reviewer filter. These display bits can trigger invalidation and alter which
logical content is included.

The resulting invalidation code is routed through the document-list invalidator.
It first resolves the active list owner; for the common marker-list,
document-list, and root-list dispatch family, the target is the resolved list's
state field. A constructor helper makes the resolved list refer to itself, so
the state is the resolved list itself. The invalidator then ORs the masked
invalidation code into the list's dirty-state word. The downstream dirty-state
dispatcher reads that exact word, tests the layout-invalidation bit, and calls
the cache-reset routine. The callee resets two ten-entry 16-bit list-scan cache
arrays and rebuilds section-dependent state; the overlapping combined branch
also invalidates header, footer, and section-definition cache state. Thus the
view policy is not itself width, but its inclusion decision reaches a real
layout recomputation path and can alter committed geometry.

Within the layout-invalidation branch, the effective-paragraph-shape resolver
resolves the paragraph shape at each scanned cursor. Its tracked-change path
selects the tracked prior `ParaShape` in original-view polarity; the fallback
selects the current base shape. That exact resolver is also called by
line-record construction, sizing, and spacing-distribution routines.
Consequently the view flag is only a selector, while the selected `ParaShape`
is part of true geometry.

This list dirty bit is nominally distinct from the flow-class column-break bit
that happens to use the same integer value. The former is stored in the
resolved list's dirty-state word and drives cache/effective-format
recomputation; the latter is a break candidate dispatched through the
column-transition family. No geometry rule may infer one from the other merely
because both use the same integer bit.

Additional types are:

```text
Run          = (runId, dispatchId, modeBits, styleId, languageId)
Slot         = (slotId, left : HwpUnit, width : HwpUnit)
Advance      = Int32
CumAdvance   = Position(T) -> Option<Advance>
Adjustment   = (fontId, matrixId, charSpacing, languageSpacing,
                boundaryAdjustment, cumulativeRevision)
LineContext  = (paragraphId, sourceStart, run, slot, advances, adjustment,
                ownerRoleId, frameRoleId)
Candidate    = (segmentOrigin, generatorOffset, endpoint)
AnchorClass  = HeaderFooter | Note
Anchor       = (class, sourceSpan, objectToken, hidePayload)
Descriptor   = (sourceStart, sourceEnd, slotLeft, slotWidth, flags,
                descriptorSerial)
HeaderEndpoint = (position, addressMode, headerSerial)
CallKey      = (runId, processStart, threadId, wrapperSerial,
                selectorSerial?, childSerial?, descriptorSerial?)
```

Raw object, run, anchor, owner, and frame pointers are tokens valid only inside
one process. Cross-process identity is a normalized role or fixture identity,
never pointer equality.

## 3. Cursor And Native-Span Algebra

### 3.1 Partial predecessor

The predecessor relation is a partial function parameterized by source and
control metadata:

```text
pred_T : Cursor(T) -> Option<Cursor(T)>
```

When `pred_T(i,q)=(j,q')`, define the crossed span
`cross_T(pred,c)=T[j..i)`. Required structural laws are:

```text
P1  0 <= j < i <= |T|
P2  pred_T is deterministic for one acknowledged revision and context
P3  j and i are legal source boundaries
P4  repeated pred_T strictly decreases position and therefore terminates
```

`P1-P4` are proof obligations unless a fixture demonstrates the relevant
source class. The state transformation `q -> q'` remains an unknown partial
function `sigma_pred(T,j,i,q)`.

### 3.2 Atomic spans

Let `Atomic_T` be a set of nonempty, nonoverlapping source intervals. A legal
cursor may lie at an atomic interval boundary but not strictly inside one:

```text
legal_T(i) iff not exists [a,b) in Atomic_T . a < i < b
```

`PROVED`: TAB in the accepted fixture occupied an atomic eight-unit source
span, and the selected endpoint was after the entire span.

No general eight-unit rule is assigned to every control. The mask `0x00ffdbfe`
is retained as a `STATIC` control classifier, not a public object type.

### 3.3 Ordinary stop set and ownership

Define the exact recovered stop set:

```text
StopUnit = { 0x0020, 0x3000, 0x000D, 0x000A, 0x0009, 0x001F }
           union { 0x2000,...,0x200B }
```

For cursor `c` and `pred_T(c)=p`, `ordinary_stop(T,p)` is true when the first
unit of `cross_T(p,c)` is in `StopUnit`, subject to atomic-span parsing.
The ownership convention is:

```text
                 pred_T(c) = p    ordinary_stop(T,p)
STOP-PREVIOUS  ----------------------------------------
                 ordinary_endpoint(T,c) = c
```

Thus the crossed stopping separator belongs to the preceding committed
interval `[lineStart,pos(c))`. This is `STATIC` for the complete stop set and
`PROVED` for SPACE, TAB, and U+2000. U+000A was `PROVED` to move the successor
start beyond the line-break unit before width resolution. The other members
remain stage obligations; see EP-06 below.

## 4. Relational Closure Pipeline

The top-level closure is deliberately partial:

```text
close : Paragraph x LineContext x Endpoint -> Option<Endpoint>
```

It is factored into relations rather than collapsed into one function:

```text
Predecessor -> Predicate -> Selector -> Generator -> Measure -> Scale
            -> Finalizer -> Wrapper -> DescriptorCommit
```

Any relation may be bypassed by a branch. Absence is meaningful only inside a
complete causal window.

### 4.1 Wrapper

`Wrap(T,L,c0,e)` is the relation implemented by the wrapper routine between
overflow probe `c0` and final wrapper pair `e`.

```text
WRAP-DIRECT       direct_policy(T,L,c0)
                -------------------------
                 Wrap(T,L,c0,c0)

WRAP-SELECT       Select(T,L,c0,s)  AcceptIntermediate(T,L,s)
                  Finalize*(T,L,s,e)
                -------------------------------------------
                 Wrap(T,L,c0,e)
```

`Finalize*` is identity when the branch does not invoke the finalizer. The
following parameters remain uninterpreted:

```text
policyPtr, policyLoopControl, policyBit02,
formatterEndpointOrigin, formatterDCSign,
modeClass in {KEEP_WORD=0x00,HYPHENATION=0x20,BREAK_WORD=0x40},
reservedModeClass=0x60, signedModeBit,
acceptSecondPredicate, validCumulativeSlot
```

`PROVED`: wrapper output is a `(position,state)` pair. `PROVED`: it can differ
from selector output (`211 -> 302`) and the committed descriptor began at
`302`.

`FORMAT_MODEL`: HWP 5.0 ParaShape bits `5..6` and HWPML define the three values
as word, hyphen, and character. Therefore mode class `0x40` is `BREAK_WORD`,
not an unknown fourth policy. The wrapper agrees: when the contextual predicate
is nonzero, class `0x40` bypasses the selector, retains the direct overflow
cursor, and invokes the finalizer when the cursor remains after line start. The
remaining bit-pattern `0x60` is unassigned by the format and remains reserved
rather than receiving a behavioral name.

### 4.2 Contextual predicate

`PredQ(T,L,c) in {0,1}` denotes the contextual predicate routine. It is not
named "breakable" or "unbreakable."

`PROVED` only in captured contexts:

```text
SPACE, TAB                     -> 0
NBSP, FEFF, nonbreaking hyphen -> 1
Latin A/B                      -> 1
captured Korean units          -> 0 in another context
```

Because results vary by context, the semantic signature is at least
`Paragraph x LineContext x Cursor -> Bool`, not `U16 -> Bool`.

### 4.3 Selector and generator

`Select(T,L,c0,e)` is the conditional relation implemented by the endpoint
selector. `Generate(T,L,segment,k)` returns either a generator offset `k` or
`NONE`, where the sentinel `0xffffffff` denotes `NONE`.

```text
GEN-ENDPOINT     Generate(T,L,segment,k)    k != NONE
                 e = segment.origin + k + 1
               --------------------------------------
                 Candidate(T,L,segment,k,e)
```

`PROVED`: the conversion is `origin + offset + 1`; width `9236` used generator
`15` and endpoint `16`, while width `8104` enumerated `15,13` and accepted
endpoint `14`.

Source unit `0x18` is HWP 5.0 paragraph-text code 24, the one-unit explicit
hyphen. Let `h` be the cursor immediately after the nearest explicit hyphen
found by the backward scan. The selector establishes:

```text
modeClass != 0x20 and explicit hyphen encountered -> direct endpoint h
modeClass == 0x20 and generated endpoint e < h     -> endpoint h
modeClass == 0x20 and generator exhausts           -> fallback h
```

The immediate-predecessor case also returns `h` directly. For a non-immediate
hyphen in class `0x20`, the selector preserves both `h` and its address-state
byte while continuing candidate generation. Thus code `0x18` is an explicit
source-authored endpoint floor/fallback in the hyphenation path, not the
optional rendered decoration produced for a dictionary candidate.

Unknown selector parameters now include generator/helper class labels,
enumeration order across mixed scripts, fallback selection when no explicit
hyphen exists, non-`1` output states, and adjacency to atomic spans.

### 4.4 Candidate receiver, numeric production, and fit

The canonical paragraph measure slot does not return an `Int32` measure. It
selects and returns an opaque array element used as the receiver of the
downstream numeric-producing pipeline:

```text
candidate_receiver : Run x Candidate x StateByte x MeasureContext -> OpaqueReceiver
produce_i32         : OpaqueReceiver x ScaleContext x StateByte
                      x FixedGlobal x SharedPair -> Int32
```

`MeasureContext`, `ScaleContext`, `FixedGlobal`, and `SharedPair` are
structural placeholders. Their meanings and the receiver's runtime class remain
unknown. The accepted fit rule recovered by static data flow is:

```text
              A[e - state.sourceStartAtOrigin] = a
              candidate_receiver(...) = r
              produce_i32(r,xi) = s
FIT-ACCEPT  ---------------------------------------------------------
              fits(L,e) iff signed_i32(a + s) <= signed_i32(L.slot.width)
```

The addition is implemented modulo `2^32`, followed by a signed comparison.
This relation is `STATIC`. A joined receiver/output row remains an EP-02
obligation. Observed behavior does not expose a separate numeric `raw -> scaled`
quantity or justify the former decoration/spacing/remeasurement hypotheses.

### 4.5 Finalizer

`Finalize(T,L,e0,e1)` is the relation implemented by the finalizer routine.
Static flow proves that it may walk backward, stops at a zero cumulative entry,
and calls the paragraph line-end membership slot on the adjusted-position unit
before the line-start membership slot on the pre-adjustment unit.

```text
FINALIZE-STEP   adjust(T,e0,state)=p   CumAdvance(pos(p)) != 0
                V54(unit(T,p),policy)=b54
                V50(unit(T,e0),policy)=b50
                b54 or b50
              --------------------------------------------
                Finalize(T,L,e0,e1) if Finalize(T,L,p,e1)
```

`V50` and `V54` are statically closed as configurable UTF-16 membership tests
for line-start and line-end prohibition respectively. They are the two sides of
kinsoku (금칙) boundary protection and can change the committed horizontal
endpoint.

`PROVED`: the line-start slot resolves to the line-start prohibition test and
the line-end slot resolves to the line-end prohibition test. `OBSERVED`: both
returned zero in inserted and restored canonical mixed-text states, in line-end
then line-start order, without endpoint movement.

### 4.6 Descriptor commit

`Commit(L,e,d)` is the descriptor-commit relation.

```text
COMMIT   Wrap(T,L,c0,e)   d.sourceStart = L.sourceStart
         d.sourceEnd = pos(e)   d.slotLeft = L.slot.left
         d.slotWidth = L.slot.width
       --------------------------------------------------
         Commit(L,e,d)
```

The emitted line descriptor stores, in field order, source start, slot left,
slot width, and source end (`PROVED`); the controlled fixture observed a runtime
stride of `0x60`. The separate paragraph layout header has a `0x3c` stride and
stores endpoint position and state byte in adjacent fields (`OBSERVED`, joined
static/dynamic). No emitted-descriptor state offset is proved. Revised EP-07
must join header state and descriptor position without treating them as one
structure. Wrapper, selector, finalizer, and descriptor are distinct relations
even when their position values happen to agree.

## 5. Ordinary And Control-Anchor Transitions

The control-anchor query is modeled without assigning ownership:

```text
query : AnchorClass x Cursor -> Option<(objectToken,hidePayload,sourceSpan)>
```

Static flow yields:

```text
ANCHOR-CROSS    pred_T(c)=p   query(k,c)=a   a.hidePayload != 0
              ------------------------------------------------
               scan_step(T,L,c)=p     endpoint_selected(a)=false
```

The shared anchor gate has one deliberate class asymmetry:

```text
AnchorGate(Note,a,allowUnlinkedNote) =
  (a != none) and (allowUnlinkedNote or a.hidePayload != 0)

AnchorGate(HeaderFooter,a,allowUnlinkedNote) =
  (a != none) and (a.hidePayload != 0)
```

The document supplies `bypassNoteHideGate` from a document flag while the
continuation writer and ordinary object-control suppressor supply false. This
gives the flag the high-confidence local behavioral name
`bypass_note_track_change_hide_gate`. Its request provenance is static
high-confidence: the flag owner is the document object. A document routine
temporarily sets a context bit when that document's flags intersect a mask,
invokes the request translator, and restores the saved context word. The
translator maps that context bit to one request flag, and a second routine adds
another; the downstream state exposes these as two adjacent request bits. The
higher bit enables flattened-to-local canonicalization during the same scan.

One source flag is the internal `em%%`/`rm%%` master-page control mode. Another
source flag is synchronized from an embedded track-change manager slot, whose
implementation tests for an entry after reserved index zero with a nonzero
inherited reference count in the document's track-change table. The change-type
discriminator is separately stored on the track-change entry. The behavioral
labels are `master_page_control_mode` and `has_referenced_track_change_entry`.
The option cannot relax the header/footer `hide` requirement and carries no
horizontal advance; it can only alter note-anchor endpoint eligibility. That
topology change can indirectly change committed geometry by changing which
content enters the logical range.

This rule is `STATIC` for both the HeaderFooter and Note classes; a matching
dynamic scan was absent from the observed evidence. It proves neither that `a`
owns the line nor that its payload is an owner pointer.

Two transition families must remain separate:

```text
SettledAnchor(T,L,a): owner/frame/reservation tuple is invariant
OwnerChangingAnchor(T,L,a,L'): owner/frame/reservation changes and the
                               successor committed descriptor is in L'
```

EP-06 may prove endpoint behavior only for settled geometry or must report the
owner tuple as part of its causal key. Note/header owner-changing cases belong
to XL-03 and require FL-03/FL-04 roles. Anchor identity alone never selects a
page owner.

## 6. Global Invariants

An accepted trace must satisfy all of these invariants.

1. **Boundary:** every endpoint is a legal UTF-16/source-span boundary.
2. **Atomicity:** no committed interval bisects an acknowledged atomic span.
3. **Progress:** predecessor steps strictly decrease position; descriptor
   intervals are nonempty unless a separately proved empty-line rule applies.
4. **Separation:** selector, wrapper, finalizer, and descriptor outputs are
   stored separately and equated only by evidence.
5. **Fit:** a selected measured candidate satisfies the exact `A(e)+s<=width`
   row; a rejected candidate does not become accepted by name or order alone.
6. **Causality:** child calls inherit active per-thread serials. Timestamp
   proximity and equal pointers are insufficient joins.
7. **Owner locality:** raw owner/frame/anchor pointers are compared only within
   one process; cross-run comparison uses accepted normalized roles.
8. **Descriptor completeness:** committed intervals are ordered, contiguous
   through paragraph CR, and tied to the acknowledged paragraph revision.
9. **Single mutation:** changed-state input is one UTF-16 unit and no inverse
   is sent before descriptor acknowledgement.
10. **Restoration:** text, complete intervals, slot, endpoint state,
    owner/frame roles, and line-local adjustment tuple return exactly to the
    baseline before a run contributes evidence.

These are executable verifier conditions, not claims that evidence is currently
present for every case.

## 7. Proven Lemmas And Limits

### L1 — Output non-substitutability (`PROVED`)

There exists a trace with selector position `211`, wrapper position `302`, and
descriptor boundary `302`. Therefore `Select.position = Wrap.position` is not
an invariant.

### L2 — Candidate conversion (`PROVED`)

For accepted hyphenation candidates, `endpoint = origin + offset + 1`. This
does not prove generator class names or fallback order.

### L3 — Ordinary separator ownership (`STATIC` + scoped `PROVED`)

Returning `c` after detecting the stop in `cross(pred(c),c)` includes that stop
in the preceding interval. Dynamic scope is SPACE, TAB, and U+2000 only.

A repeated discovery sweep additionally observed U+2001..U+200A with equal
drawing-backend/cache advance `1300`, and U+200B with cache advance `0`; all
produced the same wrapper endpoint `212`, committed interval `[159,212)`, and
exact restore to `[159,211)` in the captured fixture. The sweep inserted text
through a single process, so it remains `OBSERVED` and does not enlarge the
scoped `PROVED` set above.

The fixed-width-space follow-up repeated literal U+001F ownership in three
fresh processes with the same endpoint and restoration tuple. A later
formatter-state capture repeated two more fresh runs and joined the numeric
term: the formatter's target cache entry was `0`, but committed advance changed
from restored `42812` to inserted `43084`, for fixture-local geometry delta
`+272`. The style-metric branch has a dedicated U+001E/U+001F arm; U+001F selects
the divide-by-16 arm and multiplies its twice-scaled result by four. A U+2007
comparator had cache entry `1300` but zero net committed delta between the
existing adjacent spaces. A function-entry follow-up isolated
`U+0020,U+2007,U+0020`: the ordinary spaces each committed `544`, while U+2007
skipped the ordinary-space cache accessor, reached boundary adjustment with
`hasAdvance=false`, and committed zero. The separator loop admits U+2000..U+200B
but calls the cached per-position accessor only for U+0020 and U+3000. These
remain scoped `OBSERVED` facts and do not enlarge the lemma's formally proved
dynamic set.

The U+001F style term is additionally resolved within that observed scope. The
branch reads `CharShape.Height`, `CharShape.SizeHangul`, and
`CharShape.RatioHangul`; it applies relative size, divides by 16 with signed
truncation, applies Hangul ratio, and multiplies by four. Single-field
Hangul/Latin pairs show that the Hangul slots alone alter the direct control
metric. The baseline direct `324` subsequently receives character-spacing
adjustment `-52`, giving the observed committed `272`. This closes field
identity and local arithmetic, but does not promote U+001F ownership to formal
EP-06 acceptance. A further paired trace shows that this downstream `-52` uses
`SpacingLatin`, not `SpacingHangul`; the direct branch still uses Hangul size
and ratio. The current separator is classified as Latin language index `1`;
the `SpacingLatin` lookup therefore resolves independently of the neighboring
seed in this non-grid path.

The captured character-grid arm replaces that non-grid spacing step rather than
modifying its result. Parameter-set readback joined `HSecDef.CharGrid=1000` and
`HParaShape.SnapToGrid=1` to formatter `gridOrPitch = 1000`. With the formatter
Wongoji bit clear, the observed rounding agrees on:

```text
gridAdvance(raw) = ceil(raw / gridOrPitch) * gridOrPitch
gridAdvance(324) = 1000
```

The target source span included U+001F plus the following separator and moved
`currentAdvance` from `22000` to `24000`, exactly two grid cells. This is
scoped `OBSERVED` evidence for the grid-clear arm.

`HSecDef.WongojiFormat` selects the Wongoji (원고지) arm. With the same grid and
paragraph snap setting, `WongojiFormat=1` produced formatter flags `0x86` and
delegated target alignment to the Wongoji cell aligner:

```text
(current, item, grid) = (16000, 324, 1000) -> (16000, 1000)  # U+001F
(current, item, grid) = (17000, 648, 1000) -> (17000, 1000)  # U+0020
```

The ordinary arm quantizes the item. The Wongoji arm may first align the
cumulative position and then align or fit the item relative to that position;
an independent unaligned separator moved `13324 -> 15000`. These remain scoped
`OBSERVED` facts and do not promote EP-06.

The internal predicate is now behaviorally named the Wongoji independent-cell
predicate. Its zero set is U+0027, U+002C, U+002E, ASCII digits, ASCII
lowercase, `U+00C0..U+024F`, `U+1E00..U+1EFF`, `U+FB00..U+FB06`, and
`U+FF60..U+FFEF`; every other BMP code unit returns one. Result one aligns
current position and item advance independently, while result zero fits the
item to the current grid cell. The cell aligner has one direct override: U+201D
after period behaves as result zero. U+2019 after period remains result one in
that helper. The enclosing bulk builder later groups both quote characters for
a separate adjustment split. Joined capture resolves the rule: period plus
U+201D is centered within one cell by splitting the 28-unit residual into two
14-unit shares (`338 + 662 = 1000`), while period plus U+2019 retroactively
closes the period to 1000 and retains a separate 1000-unit quote cell. This
caller-side rule is scoped `OBSERVED` (joined static/dynamic); it does not
promote the formal EP-06 contract or establish cross-version behavior.

### L4 — Numeric fit form (`STATIC`)

Candidate acceptance uses
`cumulativeAdvance[candidateEndpoint - state.sourceStartAtOrigin] + scaledMeasure
<= availableWidth`, with a signed 32-bit comparison. Neither addend's semantic
name is promoted beyond its structural source.

### L5 — Finalizer call identity and order (`PROVED`/`OBSERVED`)

The two dynamic membership targets and line-end-before-line-start order are
proved for the named canonical paragraph. Their zero results and lack of
movement remain fixture-local observations; the predicate identities are now
statically closed.

### L6 — Historical restoration (`PROVED`, local)

The named finalizer capture restored the exact text suffix and six descriptor
intervals with slot left `4000` and width `46024`. This proves that capture's
restoration only; it is not an accepted EP-05 verdict.

## 8. Countermodels For Successor-Width Closure

Let trace events be:

```text
W(e,L) wrapper result       F(e,e',L) finalizer
P predicate                S selector
G generator                M measure
K scale                    R current-frame rejection
O(L,L') owner transition   C(L') flow commit
D(e,L') destination descriptor
```

All three models below satisfy current evidence because XL-01 has no accepted
matrix. They are mutually discriminated by successor-context events.

### F-CARRY (`HYPOTHESIS`)

```text
W(e,L) ; R ; O(L,L') ; C(L') ; D(e,L')
```

There are no successor `W,F,P,S,G,M,K` events. The exact position and state
pair `e` is committed under the new slot. A missing rejected descriptor is
allowed when rejection/transition/commit serials form the causal bridge.

### F-REFINALIZE (`HYPOTHESIS`)

```text
W(e,L) ; R ; O(L,L') ; C(L') ; F(e,e',L') ; D(e',L')
```

There is no successor wrapper/predicate/selector/generator/measure/scale
replay. The successor finalizer consumes the retained pair, and its exact
configured output is committed.

### F-REPLAY (`HYPOTHESIS`)

```text
W(e,L) ; R ; O(L,L') ; C(L') ;
W(e',L') [; P ; S ; G ; M ; K ; F as selected by the branch] ; D(e',L')
```

The successor wrapper receives the same source start, the proved successor
slot, and the acknowledged line-local adjustment context. At least one
applicable successor endpoint child occurs and the resulting pair is committed.

A fourth state, `UNRESOLVED`, is mandatory when an endpoint child appears
without its owning resolver/finalizer chain, the committed pair mismatches the
candidate model, or causal serials are incomplete.

## 9. Stage Proof Obligations

| Stage | Obligation | Promotion enabled |
| --- | --- | --- |
| `EP-01` | Two fresh processes resolve one identical active-run measure-slot target and entry signature without interposing on it; complete descriptors restore exactly. | Proves whether the accepted receiver uses the candidate measure slot or an alternative dispatch target. |
| `EP-02` | A fresh process preinstalls that target and emits one joined candidate offset/source endpoint, selected receiver identity, numeric-pipeline inputs/output, cumulative value, width, fit, selector, wrapper, descriptor, and state row. | Proves one numeric instance of `FIT-ACCEPT`; does not name the receiver or produced quantity. |
| `EP-03` | Two runs hold source, run/style, matrix, spacing, owner/frame, cumulative revision, and slot identity except width constant; compare candidate 15 and candidate 13. | Discriminates width dependence and narrows M1-M4/S1-S3. |
| `EP-04` | Fixture B joins selector `211`, second predicate, cumulative entries, optional finalizer, wrapper `302`, and descriptor boundary under two fresh runs. | Explains which relation owns the wrapper override. |
| `EP-05` | Mode `0x00/0x20/0x40` and six two-sided boundary pairs recur; line-end/line-start order and reviewed output argument remain stable. | Dynamically confirms the `BREAK_WORD=0x40` path without assigning semantics to reserved `0x60`. |
| `EP-06` | Sixteen fresh ordinary/control cases prove stop ownership, atomic spans, matched payload, predecessor movement, and settled or explicitly included owner tuple. | Extends L3 and establishes anchor relation/identity without typing ownership. |
| `EP-07` | Revision required: two discovery and two confirmation runs must join wrapper pair to paragraph-header position/state and descriptor position under exact restoration. | Defines endpoint traversal state used by XL-01 without inventing descriptor-local state. |
| `XL-01` | Six geometry/transition cases, twice each, join current closure, rejection, owner transition, commit, successor endpoint events, and destination descriptor. | Selects exactly F-CARRY, F-REFINALIZE, F-REPLAY, or `UNRESOLVED`. |

Until their obligations close, `close`, `pred_T`, `candidate_receiver`,
`produce_i32`, `Finalize`, and anchor-owner transitions remain partial at the
unproved cases listed above.

## 10. Structural Evidence Appendix

This appendix records the exact structural relations observed for HNC's
endpoint/line-closure implementation. Recovered class/identifier names are
descriptive only; they do not by themselves promote behavioral semantics.

### 10.1 Selector object and candidate dispatch

The selector loads a virtual receiver from the endpoint state's receiver field.
The candidate path reaches this receiver's measure slot.

The explicit-hyphen path is localized in the same selector. It compares the
source unit with `0x18`; requires mode class `0x20` before the scan may
continue past it; and stores the associated address state and after-hyphen
cursor once. During candidate enumeration it compares the stored cursor with
the generated endpoint and jumps to the explicit-hyphen return when the stored
cursor is greater. On generator exhaustion it chooses the stored explicit-hyphen
pair unless its cursor is still the `NONE` sentinel. These branches prove the
endpoint-floor/fallback relation independently of the HWP 5.0 format table that
names code 24 as the hyphen.

Immediately before that dispatch, the caller stages, in call order:

```text
arg1 = candidateEndpoint
arg2 = byte-derived state
arg3 = measure-context base + 0x24
```

Two deeper words, a deeper measure-context field and zero, are pushed first.
They are not additional arguments of the candidate virtual when the receiver
has the canonical paragraph dispatch: that callee consumes only three arguments.
They remain on the stack for the following scale call.

The preserved capture proved that the canonical paragraph's finalizer slots
resolve to the line-start and line-end membership tests. That adjacent pointer
pair occurs once in the image, in the canonical paragraph dispatch, whose measure
slot is the candidate measure target and whose complete-object locator names the
paragraph class.

The candidate measure target's proved structural relation is:

```text
t = call selectHelper(this, arg1, arg2)
q = (unsigned(t) < unsigned(*(arg3 + 0x10))) ? t : 0
receiver = MArrayBase::_GetPtr(this = arg3 + 4, q)
```

The returned pointer is therefore an opaque element pointer, not a numeric raw
measure. This is the strongest structural target for a selector receiver whose
dispatch is the canonical paragraph dispatch. It is not yet proved that every state
receiver has that dispatch. Therefore the formal target remains:

```text
measureTarget(state) = *(*[state.receiverField] + measureSlot)
candidate target when dispatch == canonical paragraph dispatch: candidate measure target
alternative target set: other runtime dispatchs admitted at the receiver field
```

The alternative set cannot be collapsed from names or layout similarity; its
membership remains a dynamic parameter.

The `selectHelper` used to obtain `t` performs no receiver-state store. Its
exact structural map is:

```text
X := this; p := arg1
if low_byte(arg2) == 1:
  while N := X.successorLink, N != 0,
        (N.continuationBit) != 0, and X.nodeLength <= p:
    p := p - X.nodeLength; X := N

B := X.recordBase + 4 * X.recordCount
C := X.tableCount16
if C <= 1 or p == 0:
  t := B.firstIndex
else:
  Q := B; i := 0
  while i < C and p >= *(Q): i := i+1; Q := Q+8
  t := *(Q-0x04)
```

The linked nodes, table records, and returned index remain heap-dependent.

### 10.2 Receiver-to-integer pipeline and exact fit index

After the virtual returns, the selector loads the zero-extended state byte at
offset 6, loads a fixed global, pushes the selected receiver, pushes one, and
calls the scale routine. Because the candidate virtual consumed only its first
three arguments, the scale callee sees six stack arguments:

```text
scale(stateByte6, fixedGlobal,
      1, receiver, scaleContext, 0, sharedValue, sharedRef)
```

The scale routine saves its register arguments and accesses the six caller
words through its preserved entry stack. The selector then clears all 24 bytes
of that argument block. The scale routine's structural relation is:

```text
out[0..n) := 0
ok := call scaleInner(register contexts, n, receiver, context144,
                      out, gate, sharedValue, sharedRef)
result := modular_i32_sum(out[0..n))
```

`n=0` returns zero without the downstream call. Allocation is stack-based when
`4*n < 0x40000` and heap-based otherwise. Summation uses SIMD `paddd` and
scalar `add`, so it is modulo `2^32`. In this endpoint caller `n=1`; a failed
downstream call leaves the zero-initialized output and the result is zero.

The receiver chain descends scale -> scaleInner -> forwarder -> receiver-method.
The forwarder installs the candidate return as the receiver-method's `this`,
and that method immediately preserves and later dereferences that receiver.
Downstream calls include two helper routines and several virtual slots on a
forwarded object. Their concrete targets and numeric meanings are heap-dependent.

For the endpoint specialization, the forwarder forwards the following
structural call to the receiver-method:

```text
this = selected receiver
stack = (scaleContext, stateByte6, 1, fixedGlobal,
         1, out, 1, 0, 0)
adjacent retained pair = (formatter retained field 1, formatter retained field 2)
```

Thus the ABI and modular output algebra are fixed, but there is no separately
proved numeric raw-to-scale transformation.

The accepted fit relation is exact:

```text
index  = candidateEndpoint - state.sourceStartOrigin
total  = cumulative[index] + producedI32
accept = signed_i32(total) <= signed_i32(availableWidth)
```

The evidence is the subtraction by the state's line-width origin field, an
indexed load from the cumulative array, addition of the produced integer, a
compare, and a signed `jle` to the accept target. The accepted path ORs
`0x80000` into a downstream dword reached from the state's flags pointer; no
behavioral name is assigned to that bit.

### 10.3 Finalizer line-end / line-start policy branches

The finalizer returns a two-dword pair through its first explicit argument.

The concrete paragraph membership functions are pure Boolean membership tests.
The line-start test (line-start membership slot) and line-end test (line-end
membership slot) are each pure Boolean tests that differ only in the sets they
select.

Let `member(S,u)` mean `wcschr(S,u) != null`. For policy pointer `P`, define:

```text
S50a = P.lineStartOverrideA if P != 0 and it is nonzero
       else (this.dispatch == canonical paragraph dispatch ? D50a : call this.dispatch lineStartSlotA)
S50b = P.lineStartOverrideB if P != 0 and it is nonzero
       else (this.dispatch == canonical paragraph dispatch ? D50b : call this.dispatch lineStartSlotB)
V50(u,P) = member(S50a,u) or member(S50b,u)

S54a = P.lineEndOverrideA if P != 0 and it is nonzero
       else (this.dispatch == canonical paragraph dispatch ? D54a : call this.dispatch lineEndSlotA)
S54b = P.lineEndOverrideB if P != 0 and it is nonzero
       else (this.dispatch == canonical paragraph dispatch ? D54b : call this.dispatch lineEndSlotB)
V54(u,P) = member(S54a,u) or member(S54b,u)
```

The canonical null-terminated UTF-16 fallback sets are exact:

```text
D50a = {0004,0011}
D50b = {0021,0025,0029,002c,002e,003a,003b,003e,003f,005d,005e,0060,
        007d,007e,00b0,00b7,3009,300b,300d,300f,3011,3015,3017,3019,
        301b,30fb,ff01,ff05,ff09,ff0c,ff0e,ff1a,ff1b,ff1e,ff1f,ff3d,
        ff3e,ff40,ff5d,ff5e,ff65,3041,3043,3045,3047,3049,3063,3083,
        3085,3087,308e,30a1,30a3,30a5,30a7,30a9,30c3,30e3,30e5,30e7,
        30ee,30f5,30f6,3001,3002,3003,3005,30fc,309b,309c,309d,309e,
        30fd,30fe,2019,201d,2024,2025,2026,2030,2031,2032,2033,2034,
        2035,2036,2037,2103,2109}
D54a = {0003}
D54b = {0028,003c,005b,005c,007b,00a7,2018,201c,3008,300a,300c,300e,
        3010,3014,3016,3018,301a,ff04,ff08,ff1c,ff3b,ff3c,ff5b,ffe0,
        ffe1,ffe5,ffe6,0024,00a2,00a3,00a4,00a5,09f2,09f3,0e3f,20a0,
        20a1,20a2,20a3,20a4,20a5,20a6,20a7,20a8,20a9,20aa,20ab,20ac}
```

Both functions read `this`, `P`, and the selected strings but perform no
object, cursor, or state store. They return exactly `1` on membership and `0`
otherwise. The set contents and caller-side boundary orientation establish:

```text
V50 = LineStartForbidden  # right side of proposed boundary
V54 = LineEndForbidden    # left side of proposed boundary
```

The four configurable strings are not anonymous formatter state. The document's
forbidden-character policy points to a 20-byte policy object whose four fields
are indexed `0..3`. An index-mapping routine maps those indices to
`D50a,D54a,D50b,D54b`; the native archive labels are `si,ei,su,eu`. The setter
sorts and deduplicates each UTF-16 set and clears an override when it is equal
to its default.

HWP DocInfo record `0x5e` is `HWPTAG_FORBIDDEN_CHAR`. Its parser reads four
dword lengths and then four UTF-16 strings, writing the same four policy
indices. The OWPML header `forbiddenWordList/forbiddenWord` follows the same
order: the reader decodes at most four `Forbidden_%d` entries into the setter,
while the exporter emits the four document fields. This proves document-level
customization and lifetime as well as default behavior.

The HWPX layout-compatibility members controlling trailing alignment are in the
second compatibility dword:

```text
HWPX name                         doNotAlignLastPeriod
compatibility object field       second dword at base + 0x08
mask                             0x00200000 (bit 21)
HWPX name                         doNotAlignLastForbidden
compatibility object field       second dword at base + 0x08
mask                             0x00400000 (bit 22)
Document projection          document's compatibility projection == base + 0x04
```

The writer, reader, and 50-member all-properties projector establish the sparse
first-dword group, then map second-dword bits `0..23` to XML members `9..32`.
`doNotAlignLastPeriod` and `doNotAlignLastForbidden` are members `30` and `31`,
so the `0x00200000/0x00400000` tests in the trailing-alignment consumer are
their exact type-correct formatter consumer.

Its predicate is:

```text
TrailingAlignmentExempt(u) =
  (doNotAlignLastPeriod && u in {'.', ',', '?', '!'}) ||
  (doNotAlignLastForbidden && LineStartForbidden(u))
```

This does not enable or disable the endpoint finalizer's forbidden-boundary
membership test. It uses the same `LineStartForbidden` set as a character
classifier for a different operation. In the partial-fit consumer, when the
accumulated preceding extent fits but adding the current unit would cross the
horizontal interval, a true result skips the write of the formatter overflow
bit and the unit is consumed. In the paired stop-helper consumer, the same
predicate skips the ordinary stop helper under the analogous partial-fit
condition. Thus a matched unit can remain at the current line end with its
advance protruding beyond the nominal interval instead of forcing earlier
endpoint resolution.

The trailing-alignment scan supplies the alignment half of the behavior. Its
backward trailing scan stops at a non-space unit with nonzero advance only when
`TrailingAlignmentExempt` is false. A matched punctuation/forbidden unit stays
in the trailing suffix excluded from ordinary line-end alignment. The two flags
are therefore high-confidence true horizontal endpoint/alignment geometry, not
opaque serialization state.

The adjacent third-dword field is also geometry-bearing:

```text
HWPX name                         applyCharSpacingToCharGrid
compatibility object field       third dword at base + 0x0c
mask                             0x00000002 (bit 1)
```

The grid-path consumers test it whenever the formatter carries a character-grid
width. Clear selects ordinary grid-cell rounding (and the dedicated U+001F
control metric/grid arm); set selects the language-specific character-spacing
adjustment inside the grid path. This changes per-unit advance, accumulated
horizontal position, and potentially the selected endpoint. It is true
horizontal grid geometry.

The same compatibility dword has three further closed members:

```text
doNotApplyGridInHeaderFooter        compatibility-dword bit 2
applyExtendHeaderFooterEachSection  compatibility-dword bit 3
doNotApplyHeaderFooterAtNoSpace     compatibility-dword bit 4
doNotApplyColSeparatorAtNoGap       compatibility-dword bit 5
```

The type-correct consumer of bit 2 derives the active grid interval and snap
flag, then walks the owner chain when the bit is set. Encountering a `head` or
`foot` owner zeroes both outputs. This is true grid geometry scoped to
header/footer text.

Bit 3 is consumed directly after the base accessor in three header/footer
sizing routines. The middle routine provides the geometry join: it computes
positive effective-header/footer height deficits, adds them to the two reserved
edges, and subtracts them from remaining body height. This changes vertical
flow capacity and can change pagination.

Bit 4 gates header/footer visibility, not body allocation. The visibility
consumer tests the effective section reservations; with a nonpositive
reservation and the bit set, it skips the coordinate-transform/drawing path for
that owner. The paired output path repeats the gate. No body-height or
line-slot mutation is joined to the bit, so it is `PaintGeometry` on current
evidence.

Bit 5 has a different domain. The column-gap routine computes the inter-column
gap; when the bit is set and the gap is nonpositive, the separator drawing loop
is not entered. The paired print/export consumer has the same gate. This is
paint-only column-separator geometry on current evidence, with no proved effect
on content slots or endpoint selection.

Third-dword bits 0 and 6 are negative closures. Their exact HWPX names are
`baseLineSpacingOnLineGrid` and `doNotApplyLinegridAtNoLinespacing`, but no
type-correct runtime consumer appears in the complete direct compatibility
access set, the accessor census, or the three closed pointer-mirror owner
types. Round-trip and all-properties projection preserve both bits. They are
high-confidence `OpaqueCompatibilityState` and excluded from endpoint, width,
vertical flow, pagination, and paint geometry.

Fourth-dword bit 5 is positively closed as object-placement geometry:

```text
HWPX name                         extendVertLimitToPageMargins
compatibility object field       fourth dword at base + 0x10
mask                             0x00000020 (bit 5)
```

The placement consumer reads the exact field. Under the object-mode predicate
`(attr & 1) == 0`, `(attr & 0x18) == 0x10`, and `(attr & 0x2000) != 0`, it
clamps a negative object vertical value to zero when the bit is clear and
returns the negative value unchanged when the bit is set. A downstream routine
joins that result to a transformed bound delta and stores the resulting placed
coordinate. Two further routines join the same value to available vertical
extent and to a remaining-span calculation before placement dispatch. This is
high-confidence true vertical object-placement geometry. It can change object
bounds and flow availability, but it is not a horizontal endpoint or
glyph-width rule.

Fourth-dword bit 8 is positively closed as line geometry:

```text
HWPX name                         adjustBaselineOfObjectToBottom
compatibility object field       fourth dword at base + 0x10
mask                             0x00000100 (bit 8)
```

Two line-metric routines read the exact field. They keep an all-unit vertical
maximum and ordinary-text-only primary/secondary maxima. If the all-unit
maximum exceeds the text-only primary maximum, the set bit replaces the former
with `all_max + text_primary_max - text_secondary_max`. In the line-metric
writer that adjusted value is written into the line-metric result before
line-spacing and baseline offsets are derived. This is the concrete geometry
join implied by the schema name: an inline object/control that dominates the
ordinary text can change the line box and baseline. The flag is true vertical
line geometry and may change downstream pagination; it does not alter horizontal
endpoint selection directly.

The neighboring HWPX member `doNotAlignWhitespaceOnRight` is second
compatibility word bit `0`:

```text
HWPX name                         doNotAlignWhitespaceOnRight
compatibility object field       second dword at base + 0x08
mask                             0x00000001 (bit 0)
Document projection          document's compatibility projection == base + 0x04
```

Its confirmed non-layout consumers are the HWPX/OWPML round-trip path and the
all-members property projector. A complete compatibility census and all base
accessor calls produced no direct second-word bit-0 layout test. Several
initially plausible sites instead resolve to second-word bit `4`, and one
resolves to first-word bit `1`. The accessor does mirror the whole
compatibility pointer into three object fields. The mirror owners are exact:
the general-shape placement, the table placement, and the common shape-component
base. Neither placement dispatch reads the mirrored field, and a return-provenance
census of every placement lookup finds no external consumer. The type-correct
shape-component consumers read a different compatibility bit; none reads this
member. The corrected nested-field scan also produces zero bit-0 consumers. The
semantic status is therefore:

```text
class       OpaqueCompatibilityState
geometry    excluded
confidence  high
consumers   HWPX/OWPML round-trip and all-members property projection only
```

This is a negative build-specific result, not a positive interpretation of the
flag name.

The optional policy pointer is structurally gated as follows:

1. The formatter policy source must have the expected dispatch slot; otherwise
   that slot is called.
2. The list-owner resolver must return an object whose expected dispatch slot
   matches; otherwise that slot is called.
3. The resulting pointer chain is the resolved list's state field, then a
   document link, then the forbidden-character policy. Null at any gate yields
   null.

When the endpoint is above the formatter's line-origin field and that pointer
is non-null, any nonzero override string field sets the loop-policy boolean.
With the boolean clear, the loop stops at iteration count two. With it set, this
two-iteration cap is bypassed, but the endpoint bound and cumulative entry gates
still apply.

For each eligible iteration, the helper adjusts the endpoint, reads a 16-bit
source unit, and requires `cumulative[adjustedEndpoint - formatter.lineOrigin]
!= 0`. It then calls the paragraph receiver with the same two structural
arguments in this order:

```text
lineEndSlot(adjustedUnit, policyPointer)
if return == 0:
  lineStartSlot(preAdjustmentUnit, policyPointer)
```

A nonzero result from either slot takes the common branch, retains the adjusted
endpoint and low byte of the supplied state, increments the iteration count,
and retries. Two zero results leave the incoming pair unchanged for that
iteration. The preserved dynamic capture proves the call order and zero/zero
result only for its named restored paragraph; it does not name either
predicate.

The surrounding wrapper has a separate direct line-end policy call. It first
obtains the second argument through the formatter policy source's dispatch slot,
and passes the 16-bit unit read at the adjusted endpoint. A nonzero return
selects the adjusted endpoint; zero preserves the pre-branch endpoint. This
branch does not call the line-start slot.

The reviewed paragraph-receiver caller census in the layout slice is:

| Callsite | Slot | Structural use |
| --- | --- | --- |
| trailing-scan helper | line-end, then conditional line-start | Boolean helper over two supplied units and one policy pointer |
| stop-helper consumer | line-start | Conditional unit membership after external flag gates |
| finalizer | line-end, then conditional line-start | Finalizer adjusted/pre-adjustment pair |
| early scan gate | line-start | Early scan gate using one fetched unit and formatter policy |
| wrapper gate | line-end | Wrapper adjusted-unit gate |

This is a typed caller census, not a claim that every call through a
same-numbered dispatch slot targets this paragraph dispatch.

### 10.4 Remaining dynamic parameters

The structural evidence now closes the candidate ABI, scale ABI, exact fit
index, and finalizer branch order. It does not close:

- which dispatch(s), besides or instead of the canonical paragraph dispatch, occur
  at the state receiver field on an accepted selector run;
- the runtime class and validity domain of the receiver returned by the
  candidate measure target;
- the semantic transformation performed by the scale routine, including the
  meaning/unit of `out[0]` and roles of its fixed global, context pointer,
  zero, and shared pair;
- a joined numeric row connecting generator offset, candidate endpoint,
  receiver identity, produced integer, cumulative entry, width, selector pair,
  wrapper pair, and committed descriptor.

Those remain explicit dynamic parameters; they are not inferable from recovered
identifiers or helper names.
