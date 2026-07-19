---
kind: reference
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# HNC Flow-Owner And Pagination Formal Semantics

This document gives a typed, partial semantics for the HNC flow-owner path. It
formalizes the canonical model of how a flow cursor resolves break directives,
commits, selects a successor frame, and reserves space, and it does not promote
an unresolved construction or successor-selection choice.

The model deliberately has one theorem: the terminal-column reset. Frame
construction, successor selection, page-control selection, reservation, table
fragmentation, and successor-width endpoint closure remain partial relations
with explicit proof obligations.

## 1. Model Scope And Claim Roles

Every assertion carries one of these roles:

- **[MODEL]**: a defined behavior of the canonical flow-owner model.
- **[FORMAT]**: the serialized HWP/HWPX input declares the value.
- **[HYP]**: one member of a live competing model family.
- **[OPEN]**: the model does not select one total behavior.
- **[REQ:stage]**: a proof obligation that must be discharged from the named
  roadmap stage before the relation may be promoted to canonical.

Model behaviors describe the flow-owner instance under study. A model role,
routine name, or normalized key is meaningful only inside the model; it is not a
transferable identity.

## 2. Typed Domains

The domains are disjoint even when a concrete run represents two values with
equal integers or pointers.

```text
RunId, RevisionId, ParagraphId, SourceId
SectionKey, PageKey, FrameKey, OwnerKey, AnchorKey, ObjectKey
TableKey, RowKey, CellKey, FragmentKey, ControlKey

HU          = signed HWPUNIT coordinate
SourcePos   = non-negative source cursor
AddressState= opaque cursor-state byte
ColumnIndex = non-negative ordinal
LineIndex   = non-negative ordinal
RawWord     = unsigned 32-bit value
Nat         = non-negative integer

ConstraintState, ReservationState, NoteState, LineContext
EndpointResult, EventWindow = opaque typed domains
```

A concrete pointer witnesses an identity only within one `RunId`. A normalized
key is a typed structural tuple, never a runtime address reused across runs.

### 2.1 Directives And Neutral State

```text
RawDirective = Raw(RawWord)
CommittedDirective = CommitWord(RawWord)

NoDirective     = Raw(0)
PageDirective   = Raw(0x200)
ColumnDirective = Raw(0x400)

AxisTuple = (axis20: RawWord, axis24: RawWord, axis28: RawWord)
FlowAxis  = OpaqueAxis(RawWord)
```

`axis20`, `axis24`, and `axis28` remain neutral field names. In particular,
`OpaqueAxis(0)` is not called "first column" or "next page." **[OPEN]**

### 2.2 Owners, Frames, Slots, And Pages

```text
OwnerKind = Body | Header | Footer | Note | TableFragment
          | CellText | FloatingObject | TextObject | UnknownOwner

Owner = {
  key: OwnerKey,
  kind: OwnerKind,
  parent: Option<OwnerKey>,
  constraints: ConstraintState
}

Slot = { left: HU, width: HU }

Frame = {
  key: FrameKey,
  section: SectionKey,
  page: PageKey,
  column: ColumnIndex,
  owner: OwnerKey,
  y_origin: HU,
  y_limit: HU,
  slots: HU -> OrderedSet<Slot>
}

FrameGraph = {
  frames: Set<Frame>,
  successor: FrameKey -> Set<FrameKey>
}
```

`OwnerKey`, `AnchorKey`, `ObjectKey`, a placement-record identity, and
`FrameKey` are different types. Numeric equality cannot coerce between them.

### 2.3 Flow Cursor And Descriptor

```text
FlowCursor = {
  run: RunId,
  revision: RevisionId,
  paragraph: ParagraphId,
  source: SourceId,
  position: SourcePos,
  address_state: AddressState,
  frame: FrameKey,
  owner: OwnerKey,
  axes: AxisTuple,
  flow_axis: FlowAxis
}

LineDescriptor = {
  source_start: SourcePos,  // descriptor start-of-source role
  slot_left: HU,            // descriptor slot-left role
  slot_width: HU,           // descriptor slot-width role
  source_end: SourcePos     // descriptor end-of-source role
}
```

The descriptor fields and the fixed byte descriptor extent are model roles.
They do not reveal which routine selected the owner or frame.

### 2.4 Page Controls And Table Fragments

```text
PageClass = First | Odd | Even

PageControls = {
  headers: PageClass -> Option<ControlKey>,
  footers: PageClass -> Option<ControlKey>
}

SourceCell = (TableKey, RowKey, CellKey)

TableFragment = {
  key: FragmentKey,
  table: TableKey,
  frame: FrameKey,
  source_cells: OrderedSet<SourceCell>,
  presentation_rows: OrderedSet<RowKey>
}
```

`presentation_rows` is separate from `source_cells`: a repeated header may be
presentation without duplicating source ownership. **[OPEN]**

## 3. Evidence Relations

The model consumes correlated events rather than timestamp adjacency:

```text
Observed(run, revision, thread, parent_serial, event_serial, value)
SameChain(e1,e2) :=
  same run/revision/thread and explicit parent-child call serial relation
```

Accepted pointer-derived identities require a typed normalization function:

```text
Normalize_run(pointer, witnessed_role) -> TypedKey
```

`Normalize_run` is intentionally undefined when the role was inferred from
paint, dimensions, type-name strings alone, or cross-run pointer equality.

## 4. Raw Directive And Commit Relations

Let these partial relations describe the flow boundary:

```text
ResolveRaw : ParagraphId x FlowCursor ⇀ RawDirective
TerminalPredicate : FlowCursor x RawDirective ⇀ Bool
CommitCursor : FlowCursor x CommittedDirective x RawWord ⇀ FlowCursor
```

The model names these roles:

```text
the raw directive resolver          participates in raw directive resolution
the frame-axis availability predicate  evaluates a frame-axis availability test
the flow-cursor commit routine       commits a flow cursor
the flow-axis resolver               resolves an internal flow-axis value
```

The observed raw directive is not the committed directive. The following
rule is therefore invalid:

```text
Raw(d) = CommitWord(d)                         // invalid in general
```

### 4.1 Terminal Reset Theorem

**Theorem T-RESET-400 [MODEL].** If the terminal-axis predicate called with
`axisKind=0` returns true and the same correlated paragraph-flow decision has
raw directive `0x400`, then the terminal branch commits directive zero and
resolves the internal flow axis to zero:

```text
TerminalPredicate(c, Raw(0x400)) = true
SameChain(predicate, terminal_branch, commit)
------------------------------------------------ T-RESET-400
CommitCursor(c, CommitWord(0), 1) = c'
and c'.flow_axis = OpaqueAxis(0)
```

The model witness for the reset is the ordered chain:

```text
predicate(axisKind=0)
compare raw directive with 0x400
terminal branch
push committed directive 0
call the flow-cursor commit routine
pass resolved internal flow axis 0
```

Correspondingly, the terminal decision carries raw `0x400`, terminal
`axis28=1`, commit directive argument equal to zero, and commit axis argument
equal to one. The theorem does **not** entail any of these statements:

```text
c'.frame is a next-page frame
c'.axes denotes (next page, first frame, first column)
CommitWord(0) is semantically equivalent to CommitWord(0x200)
the commit routine owns successor selection
```

Those are deliberately outside the theorem.

## 5. Successor Selection Is A Partial Relation

Define:

```text
SelectSuccessor :
  FlowCursor x RawDirective x FrameGraph x EventWindow ⇀ FrameKey
```

The model establishes a nonterminal observation:

```text
Raw(0x400), column field 0
  -> commit
  -> next body composition with column field 1                  [MODEL]
```

It does not identify the field as a globally valid `ColumnIndex`, nor does it
select a terminal successor rule.

Exactly three terminal-selection hypotheses remain live:

```text
H2a [HYP]: commit-owned selection
  the committed cursor/owner identity itself becomes the next-page/first-frame
  identity before CommitCursor returns, without a distinct carrier owning the
  transition.

H2b [HYP]: owner-composition-owned selection
  CommitCursor resets only local axis state; the first subsequent owner
  composition acquires the next-page frame.

H2c [HYP]: intermediate-chain selection
  CommitCursor resets local state; an intermediate successor-chain object
  changes identity before the next owner composition.
```

Their discriminating conditions are:

```text
Accept(H2a) iff the committed cursor/owner identity acquires NextPageKey before
                 commit_leave and no distinct intermediate carrier owns it.
Accept(H2b) iff commit_leave retains CurrentPageKey, no intermediate carrier
                 acquires NextPageKey, and the first correlated compose_enter
                 acquires NextPageKey.
Accept(H2c) iff the committed cursor/owner identity retains CurrentPageKey while
                 one causally distinct intermediate carrier acquires
                 NextPageKey before the first correlated compose_enter.
```

All three require an FL-01-proved page/frame identity. An axis reset, visual
page movement, or committed zero cannot distinguish them. **[REQ:FL-01,FL-02]**

### 5.1 The Commit Subobject Selector

The commit subobject selector is a read-only selector, not an intermediate
state writer. **[MODEL]** Its typed shape is:

```text
SelectCommitSubobject :
  CommitState x SelectorFamily x AxisCandidate ⇀ SubobjectPointer

SelectorFamily = RawWord
AxisCandidate  = ExplicitAxis(RawWord) | ResolveAxis
```

The selector consumes a commit-state object, a selector-family word, and an
axis candidate; a candidate sentinel means `ResolveAxis`. It returns a selected
subobject pointer or null.

When the axis candidate is explicit, the selector uses it unchanged. For
`ResolveAxis`, let:

```text
flags = the flag word reached through the state's axis-flag link
lo    = flags & 0x3
hi    = flags & 0x3fc
```

The axis resolution relation is:

```text
ResolvedAxis(state, ResolveAxis) =
  state.axis24  if lo = 2 and hi > 4
  0             otherwise
```

The preceding `state bit 0`, `lo = 0`, `hi > 4` branch also resolves zero; it
does not write any field. After resolution:

```text
selector-family >= 2
  => null

selector-family < 2
  => state.subobject_table[selector-family]
     indexed by resolved-axis
```

Apart from an ordinary stack-frame write, the complete selector contains loads,
comparisons, arithmetic, branches, and returns only. It has no direct or
indirect call and no non-stack/object-state store. Therefore an "exact selector
object-state write" does not exist.

There are four direct callers in the model:

```text
caller A  selector-family 0, ResolveAxis; then object queries
caller B  selector-family 0, explicit candidate; then object query
caller C  explicit family/axis; then the outward dispatcher
caller D  commit postlude; then the outward dispatcher
```

At the commit postlude, the commit routine pushes the remaining update
arguments around the selector's two consumed arguments. The returned pointer
becomes the receiver for the outward dispatcher before the commit routine
returns. The dispatcher writes only its stack frame and pushed call arguments
directly; it contains no direct non-stack/object-state store. It synchronously
invokes:

```text
selected dispatch slot (early) with arg0
selected dispatch slot (late)  with arg1 and arg2
```

The concrete virtual targets depend on the selected runtime subobject and are
not statically recoverable from this heap-relative table. Any effect of those
two calls occurs within the dynamic extent of the commit routine, before its
return. Consequently:

- if a proved next-page identity is present at commit return, the observation
  satisfies H2a's timing condition, but control flow alone cannot decide whether
  the selected object is the full committed identity or an intermediate carrier
  written by one of the virtual targets;
- selector pointer selection alone cannot establish a page, frame, or owner
  identity;
- this chain cannot prove H2b, because H2b requires the first later owner
  composition to acquire the identity;
- H2c remains possible either in a concrete early/late virtual target that
  writes a proved intermediate carrier, or in an independently identified write
  after commit return and before the first later composition. The former is
  nested inside commit timing but remains intermediate-owned only if causal
  identity evidence distinguishes the carrier from committed cursor state.

The selector has a reviewed entry boundary, but it is not a write boundary. The
two concrete virtual targets and any exact state write remain to be identified
before a write inside commit timing can be attributed. **[REQ:FL-02]**

## 6. Frame Composition Is A Partial Relation

Let serialized section geometry be:

```text
SectionGeometry = {
  usable_left: HU,
  usable_width: HU,
  column_widths: Vector<HU>,
  gaps: Vector<HU>,
  vertical_origin: HU,
  vertical_limit: HU,
  break_kind: Continuous | NewPage | Other
}
```

Define the unknown constructor:

```text
ConstructFrames :
  SectionKey x SectionGeometry x PageControls x ReservationState
  ⇀ OrderedList<Frame>
```

For `n = len(column_widths)`, every accepted construction must satisfy these
constraints:

```text
n >= 1
len(gaps) = n - 1
forall i: column_widths[i] > 0
forall i: gaps[i] >= 0

abs(sum(column_widths) + sum(gaps) - usable_width) <= epsilon_HU

slot_left[0] = usable_left
slot_left[i+1] = slot_left[i] + column_widths[i] + gaps[i]
slot_width[i] = column_widths[i]

forall i < j:
  [slot_left[i], slot_left[i] + slot_width[i])
  precedes and does not overlap
  [slot_left[j], slot_left[j] + slot_width[j])
```

These are acceptance constraints, not a recovered construction algorithm. The
following alternatives remain observationally equivalent until FL-03 closes
them:

```text
H3a [HYP]: eager materialization of every column frame.
H3b [HYP]: lazy materialization as the flow cursor advances.
H3c [HYP]: equal and unequal columns use distinct construction branches but
           share one normalized successor relation.
```

Allocation count alone cannot choose among them. Each accepted frame requires a
join from section geometry through owner composition to a descriptor whose slot
left/width matches that frame. **[REQ:FL-03]**

## 7. Rejection And Successor Recomposition

Let horizontal endpoint resolution be abstract:

```text
ResolveEndpoint :
  SourcePos x AddressState x Slot x LineContext ⇀ EndpointResult

Candidate = {
  descriptor: LineDescriptor,
  vertical_extent: HU,
  paragraph_lines: Nat
}
```

Let owner constraint adjustment be partial but type preserving:

```text
AdjustOwner : Owner x Candidate x ConstraintState ⇀ Nat
```

The model fixes the ordering boundary:

```text
ResolveEndpoint(current slot)
  -> compose/admit current flow owner at the owner-composition routine
  -> rejection path at the owner-rejection routine
  -> successor placement composition at the owner-composition routine   [MODEL]
```

The recurrence is therefore partial:

```text
ResolveEndpoint(b, state, slot(F), ctx) = e
CandidateFor(e,F) = k
Admit(F,k) = false
SelectSuccessor(cursor,directive,graph,window) = F'
---------------------------------------------------------------
Recompose(cursor,k,F) ⇀ (cursor',k',F')
```

When `slot(F) != slot(F')`, the destination descriptor must be attributable to
the successor slot. The amount of replay remains one of:

```text
F-CARRY       [HYP]: carry endpoint state without endpoint child stages.
F-REFINALIZE  [HYP]: rerun a finalizer/predicate subset under successor width.
F-REPLAY      [HYP]: rerun the full endpoint wrapper chain under successor width.
```

No rule may choose among these from equal-width fixtures. The causal witness
must join current rejection, successor frame/slot, endpoint child stages, and
the committed destination descriptor. **[REQ:XL-01]**

## 8. Header, Footer, And Note Reservation

Define two partial relations:

```text
SelectPageControls :
  PageKey x PageClass x PageControls ⇀ (Option<ControlKey>,Option<ControlKey>)

ReserveBody :
  Frame x SelectedPageControls x NoteState ⇀ BodyInterval

BodyInterval = [body_origin, body_limit)
```

Every accepted reservation must satisfy:

```text
frame.y_origin <= body_origin <= body_limit <= frame.y_limit
reservation_top    = body_origin - frame.y_origin >= 0
reservation_bottom = frame.y_limit - body_limit >= 0
```

A nested note owner and a fixture-specific body-limit reduction of `65762 -
63345 = 2417` are part of the model. **[MODEL]** This establishes that note
geometry can participate in owner admissibility. It does not make `2417` a
constant and does not prove header/footer selection.

Header/footer alternatives remain:

```text
H5a [HYP]: select first/even/odd controls during page construction and reserve
           body space before body admission.
H5b [HYP]: select controls during construction; compute reservation later.
H5c [HYP]: controls are overlay owners; page margins alone determine body
           limits for the tested geometry.
```

Selection requires a source-control-to-owner join on a proved page identity.
Reservation requires a one-variable height pair whose body-limit delta is
isolated from page-margin, note, table, and floating-object changes.
**[REQ:FL-04,XL-03]**

## 9. Table Fragmentation Is A Partial Relation

Define:

```text
FragmentTable :
  TableKey x OrderedRows x FrameGraph x TablePolicy
  ⇀ OrderedList<TableFragment>

TablePolicy = {
  page_break_mode,
  repeat_header,
  header_rows,
  split_policy
}
```

An 84-row fixture declares `pageBreak=CELL`, `repeatHeader=1`, and `noAdjust=1`.
**[FORMAT]** Six outer `transitionState=2` events from one table owner,
interleaved with nested cell-owner composition, are part of the model.
**[MODEL]** The accepted boundary is therefore:

```text
table pagination includes outer table continuation
and nested cell-owner composition
```

The materialization function remains open:

```text
PresentHeader : TableFragment x HeaderRows ⇀ OrderedPresentation
```

with three live alternatives:

```text
H6a [HYP]: continuation references the original header-row owner.
H6b [HYP]: continuation clones an owner linked to the same source row.
H6c [HYP]: fragment-local presentation state is not a normal row owner.
```

The following conservation and progress obligations apply to every accepted
fragmentation result:

```text
SourceCoverage:
  each source cell maps to one complete ordered set of fragment placements;
  repeated presentation does not duplicate source ownership.

OrderPreservation:
  body source order is preserved across fragments.

PositiveProgress:
  each continuation strictly advances source coverage or vertical progress.

NoLossNoDuplicate:
  source text is neither omitted nor duplicated, except presentation already
  classified as repeated-header output.
```

Repeated-header ownership needs the full table-source -> runtime object ->
outward dispatch -> fragment lookup -> placement -> proved frame/page chain.
Row spans and over-height rows additionally need source-cell coverage and
strict progress. **[REQ:FL-05,FL-06]**

## 10. Cross-Lane Join Algebra

The lanes cannot be closed by adjacent timestamps or visually equal geometry.
Define typed joins:

```text
EndpointJoin =
  (run,revision,thread,parent_serial,paragraph,source interval,
   wrapper role,descriptor serial)

ObjectJoin =
  (run,revision,thread,parent_serial,source position/address state,
   runtime object,actual outward target,placement serial,anchor key)

FlowJoin =
  (run,revision,thread,parent_serial,current frame,rejection serial,
   successor frame,commit serial,descriptor serial)
```

Then:

```text
Join_EP_FL : EndpointJoin x FlowJoin ⇀ SuccessorEndpointEvidence
Join_OP_FL : ObjectJoin x FlowJoin ⇀ PlacementOwnerEvidence
Join_EP_OP_FL : EndpointJoin x ObjectJoin x FlowJoin ⇀ CanonicalClosureEvidence
```

The joins are defined only when run, revision, thread, parent/child call
serials, source lineage, and descriptor acknowledgement agree. A shared pointer
without role evidence is insufficient.

Stage obligations are:

```text
XL-01 = EP-03 + EP-07 + FL-01 + FL-03
        -> carry/refinalize/replay classification

XL-02a = OP-02(TAC) + OP-03a + FL-01 + FL-03
         -> TAC placement/page-owner chain

XL-02b = OP-02(non-TAC) + OP-03a + FL-01 + FL-03
         -> exclusion/page-owner chain

XL-02c = accepted XL-02a or XL-02b outer mode + mapped inner text owner
         -> nested text-box ownership without coercing inner owner to outer

XL-03 = EP-06 anchor relation + FL-03 + FL-04
        -> note/header anchor, selection, reservation, and descriptor chain

XL-04 = XL-01 + XL-02a/b/c + XL-03 + FL-06
        -> line-local canonical recurrence closure
```

## 11. Global Invariants

Any future total semantics must preserve all of these:

1. **Model identity.** [MODEL] relations hold only inside this flow-owner
   model instance.
2. **Typed identity.** Placement, anchor, owner, frame, page, and descriptor
   identities are never equated by pointer or dimension coincidence.
3. **Run locality.** Literal runtime pointers do not cross run boundaries.
4. **Directive separation.** Raw and committed directives are distinct types.
5. **Reset non-entailment.** `OpaqueAxis(0)` does not imply next-page identity.
6. **Descriptor interval.** `slot_width > 0` and
   `[slot_left,slot_left+slot_width)` belongs to the committed owner/frame.
7. **Source continuity.** Within one paragraph flow lineage, the next committed
   descriptor begins at the acknowledged predecessor `source_end`, except a
   separately proved control/source-span transition.
8. **Successor causality.** A destination descriptor after rejection must join
   to the accepted successor frame and its slot.
9. **Reservation monotonicity.** Reservations cannot enlarge the body interval.
10. **Fragment progress.** A continuation cannot repeat forever with identical
    source coverage and vertical state.
11. **Restoration.** `S0r` reproduces normalized dynamic identities and
    relations from `S0`, not merely fixture bytes or visual geometry.
12. **Provenance independence.** Stored, imported, generated, or edited origin
    is not a semantic branch unless paired evidence proves one.

## 12. Countermodels That The Model Permits Or Rejects

### 12.1 Rejected Countermodel: Terminal `0x400` Becomes `0x200`

```text
Normalize(Raw(0x400),terminal) = CommitWord(0x200)
```

This contradicts T-RESET-400 and is rejected.

### 12.2 Permitted Countermodels: H2a, H2b, H2c

All three successor-selection hypotheses produce committed zero and a later
next-page composition. Without a proved page identity at commit entry/leave,
intermediate writes, and first successor composition, all remain models of the
current behavior.

### 12.3 Permitted Countermodels: Eager And Lazy Frames

Eager construction and lazy construction can produce identical final frame
geometry. Allocation count or final paint cannot distinguish them; correlated
construction order is required.

### 12.4 Permitted Countermodels: Header/Footer Overlay And Reservation

An overlay model and a reservation model can paint identical output when page
margins already contain the control. Only isolated page-class height changes
joined to body-owner interval changes distinguish them.

### 12.5 Permitted Countermodels: Header Reuse, Clone, Presentation State

All H6 alternatives can repeat the same text on every fragment. Source-row,
owner, presentation, fragment, and page identity joins are required.

### 12.6 Permitted Countermodels: Carry, Re-finalize, Replay

Equal current and successor widths can yield the same endpoint under all three
XL-01 alternatives. Unequal-width paired fixtures and endpoint-stage traces are
required.

### 12.7 Rejected Evidence Model: Pointer Equality Implies Ownership

Allocation reuse and wrapper/component separation provide models where equal
pointers or dimensions do not denote the same typed role. Ownership is accepted
only through a correlated source/dispatch/anchor/flow chain.

## 13. Proof-Obligation Ledger

| Stage | Partial relation to close | Required discriminating evidence |
| --- | --- | --- |
| `FL-01` | semantic page/frame/column identities for neutral fields | terminal/nonterminal/page/natural controls; complete directive -> predicate -> commit -> owner windows; two fresh runs |
| `FL-02` | `SelectSuccessor` and H2a/H2b/H2c | proved next-page key at commit entry/leave, an exact post-commit intermediate write if any, and first successor compose; reopen recurrence |
| `FL-03` | `ConstructFrames` and H3a/H3b/H3c | one/equal/70-30/30-70/three-column/section pairs; ordered graph joined to descriptors; exact width reconciliation |
| `FL-04` | `SelectPageControls` and `ReserveBody` | first/even/odd present/absent and one-height-variable pairs; selected source controls and body-limit deltas |
| `FL-05` | `PresentHeader` and H6a/H6b/H6c | source table through actual dispatch and fragment lookup to proved page/frame; repeat on/off/on restoration |
| `FL-06` | row-span/over-height fragmentation | complete source-cell coverage, order, no loss/duplication, strict continuation progress |
| `XL-01` | `Recompose` as carry/refinalize/replay | unequal successor slots; rejection -> successor slot -> endpoint stages -> destination descriptor join |
| `XL-02a` | TAC placement/page ownership | accepted outer TAC dispatch plus anchor, rejection/successor, and page-owner chain |
| `XL-02b` | non-TAC exclusion/page ownership | accepted non-TAC dispatch/exclusion plus the same owner chain and TAC controls |
| `XL-02c` | nested text-box ownership | accepted outer mode plus separately mapped inner source/owner; no role coercion |
| `XL-03` | anchor-selection-reservation integration | EP-06 anchor identity plus selected page control, body interval, frame, and destination descriptor |
| `XL-04` | canonical recurrence | all consumed endpoint/object/frame/table/control identities and line-local adjustment recurrence |

No row in this table is presently promoted to canonical.

## 14. Related Models

- [unified layout semantics](unified_layout_semantics.md)
- [endpoint line closure semantics](endpoint_line_closure_semantics.md)
- [object placement semantics](object_placement_semantics.md)
- [model index](README.md)

Relation-to-role mapping within this model:

| Relation | Model role |
| --- | --- |
| explicit `0x200`/`0x400` raw classes and decision order | raw directive resolver and break-direction classifier |
| current rejection before successor composition | owner-composition routine and owner-rejection routine |
| descriptor fields and owner-relative slot interval | line descriptor slot-left/slot-width roles |
| terminal `0x400` reset witness | terminal predicate, terminal branch, and flow-cursor commit routine |
| nested-note reservation | note owner participating in owner admissibility |
| table outer continuation and nested cell owners | outer table continuation and nested cell-owner composition |

## 15. Promotion Rule

A partial relation becomes canonical only when its stage verifier emits an
accepted manifest with:

```text
model instance identity
fresh isolated composition
checksummed one-variable fixtures
complete typed causal joins
two-run recurrence where required
exact S0/S1/S0r restoration
pointer-free normalized verdict
```

Until then, implementations may expose the relation as a parameter or return an
unresolved verdict; they must not silently choose a hypothesis.
