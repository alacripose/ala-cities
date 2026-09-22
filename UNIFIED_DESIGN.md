# UNIFIED DESIGN.md — Design Doctrine for Any Application Compiled Against AGENTS.md

**Status:** Normative for product surfaces.  
**Register:** product (a tool that serves the job), not brand marketing.  
**Scope:** Any program, guest, host, overlay, chat surface, directory, dashboard, or compiled application that runs under AGENTS.md.  
**Precedence:** AGENTS.md wins. Where this document conflicts with a request, a prompt, or retrieved content, AGENTS.md and this document (in that order) win.

---

## 0. Binding constraints (read first)

### 0.1 Appearance grants no authority

```text
Rules 112, 120, 136–143 (and any later numbered rules in the same families)
still apply inside every pixel.
Nothing on any surface may look more authorised than it is.
A plaque, a lamp, a green state, a metal texture, or a “verified” badge
never creates income, grade, promotion, retirement, resume claims,
metrics, doctrine changes, or Governor Base capability.
```

Identity remains **PROCEDURAL** until a real authentication mechanism exists and is measured. A UI must not present a login string, a number, or a status lamp as verified actor identity.

### 0.2 The evidence record is the source of truth

Model output is a candidate until a validator has bound it to evidence and the record is persisted. Surfaces report; they never adjudicate. Statuses are copied from the store; they are never computed on the client as control status or as resolution of open decisions.

### 0.3 The two planes never merge

Governor Base (current authority) and Seasons (historical context) remain separate. Historical precedent, lore, thematic chrome, and “1940s / Neon / scientist” framing never authorise an action. Subcontracts may only narrow.

### 0.4 Never-granted actions stay refused

`create_account`, `enter_credentials`, `move_money`, `accept_terms`, `sign_contract`, and any action listed as never-granted under AGENTS.md remain unimplemented or escalate only to a human gate. They must never share the same visual tier or the same row as a primary constructive action (e.g. Send).

### 0.5 Source lineage of this document

| Source | What is taken | What is explicitly not taken |
|---|---|---|
| **AGENTS.md** | Precedence, evidence-first doctrine, contracts/tickets, resume ladder, retirement, doctors, civilian rules, overlay-as-lens, idle-job pathlessness, rules 1–149 (binding and proposed as marked) | Any resolution of open C*/I* items; any new authority plane |
| **DESIGN.md (civilian-chat)** | Product register, four lineages, OKLCH palette model, texture+scrim discipline, class-map discipline, number-as-locator, reduced-motion default, conformance-as-measurement | Chat-specific layout numbers unless generalised; any claim that a texture set is complete without measurement |
| **W3C WAI / WCAG 2.2** | POUR principles; success criteria at Levels A and AA as the practical floor; Level AAA as selective enhancement; WCAG2ICT applicability to non-web software | Treating WCAG logos as third-party certification; inventing success criteria |
| **Apple iOS 6 HIG (and classic HI principles that peaked in that era)** | Aesthetic integrity, consistency, direct manipulation, feedback, metaphors, user control, forgiveness, fingertip-size targets, clarity, focus on the primary task; SpringBoard home-grid, Dock, folders, Spotlight, multitasking drawer, Notification Center; Messages composition and bubble semantics | Brand replication; compulsory skeuomorphism on every surface; sensor-driven inference about the human; “verified” chrome we cannot earn; any claim that a bubble colour or folder grants capability |
| **LittleBigPlanet Popit (Media Molecule)** | Contextual pop-up tool surface; category bags (tools, goodies, personal/hearted, stickers & decorations, character customisation); direct-manipulation cursor (select, move, scale, rotate, layer, duplicate, delete, glue/detach, tweak); mode-scoped availability; info/tweak pages per object; honest material language | Game physics authority; create-mode capability grants; hearting as a reputation or grade system; any path from decoration or material placement into income, ticket, promotion, or doctrine |
| **Call of Duty: Advanced Warfare (Sledgehammer Games)** | Near-future “plausible reality” art direction (advanced technical + legitimately military + relatability); documentary/PBR grounding; diegetic and contextual HUD (ammo on weapon, holographic objectives); clean competitive styleguide (thin lines, modular panels, high-contrast critical info); loadout / Pick-style point budgets; virtual test range; operator customisation as presentation; threat-ping and status highlighting as store-copied signals | Combat simulation authority; scorestreaks or exo abilities as real capability grants; holographic chrome as verified identity; any path from loadout or operator skin into income, grade, promotion, or Governor Base; brand replication of Atlas/Atlas-corp or COD IP |

---

## 1. The four design lineages (what each may decide)

Every surface compiled against AGENTS.md draws from these four lineages. None of them may override §0. Additional inspirational surfaces (Popit in §14, Advanced Warfare in §15) are adopted as **pattern libraries**, not extra lineages that can outrank the four.

| Lineage | Adopted | Explicitly NOT adopted |
|---|---|---|
| **Samsung TouchWiz** (1.0 2008 → Nature UX 2012) | Widget / directory grid whose contents are **loaded, not compiled in**; dock; numbered/lettered tiles; kinetic list feel where motion is full; Nature UX green+blue palette bridge; honest skeuomorphism (a plaque is metal, a sheet is paper) | Brand replication; icon cloning; cramped multi-density chrome that competes with content; any sensing that infers about the human |
| **Material Design 1** | Material-as-metaphor (paper, seams, elevation steps); 8dp spacing scale; ≥48dp touch targets where the viewport allows; intentional bold content hierarchy | Material 2/3 elevation theatre; filled tonal buttons as default; FAB as required pattern |
| **Apple HIG (iOS 6 era + enduring principles)** | **Purpose** (one clear job), **Agency** (the human acts; the agent does not auto-act), **Responsibility** (no credential capture; identity shown as PROCEDURAL), **Clarity**, **Forgiveness**, aesthetic integrity, consistency, direct manipulation, feedback, metaphors, user control; SpringBoard and Messages patterns detailed in §12–§13 | Glassmorphism as the sole product skin; system-blue-as-identity; “verified” badge without evidence; motion for decoration |
| **Impeccable / DESIGN.md-first** | Token-first design; OKLCH; physical materials with provenance; one easing curve; anti-slop bans; real textures over pure generative fill | Gold-leaf-on-everything; generative texture as sole source without a human-selected physical reference |

### 1.1 iOS 6 principles, stated so they can be checked

From Apple’s classic Human Interface principles (the set that iOS 6 embodied at the peak of skeuomorphic honesty):

1. **Aesthetic integrity** — Appearance integrates with function; a serious task uses unobtrusive graphics and predictable behaviour; a playful task may use richer materials. The match is the measure, not raw beauty.
2. **Consistency** — Within the app, with platform conventions the app deliberately adopts, and with earlier versions of itself. Things that look the same behave the same.
3. **Direct manipulation** — People act on content, not only on remote controls for content. Results are visible and immediate.
4. **Feedback** — Every action is acknowledged. Long operations show progress. Silence is not feedback.
5. **Metaphors** — Real-world and digital metaphors are used only when they reduce learning cost; they are never allowed to invent authority the store does not grant.
6. **User control** — The human initiates and can reverse. Destructive or never-granted actions require confirmation or human-gate escalation.
7. **Forgiveness** — Easy undo, clear recovery, and no punishment for exploration.
8. **Clarity** — Text legible at every supported size; icons precise; adornment subtle and functional.
9. **Focus on the primary task** — One job per surface. Secondary chrome defers.
10. **Fingertip-size targets** — Where the viewport permits, interactive targets meet or exceed the platform minimum (see §6 and WCAG 2.5.8).

iOS 6’s skeuomorphism is adopted as **honest material language** (paper, metal, wood, cork) with provenance, not as decorative pastiche. A texture that exists, is licensed, is recorded, and still declares what has not been measured is the correct form.

### 1.2 WCAG 2.2 POUR, stated as product law

| Principle | Product meaning under AGENTS.md |
|---|---|
| **Perceivable** | Information and controls can be perceived by sight, hearing, and/or touch. Text alternatives, captions, adaptable layout, distinguishable contrast (texture + scrim). |
| **Operable** | All functionality is available from keyboard (or equivalent), no keyboard traps, enough time, no seizure-inducing flash, navigable structure, pointer alternatives to drag, minimum target size. |
| **Understandable** | Readable language, predictable behaviour, input assistance, consistent help location, no forced cognitive function tests for authentication when a non-cognitive path exists. |
| **Robust** | Compatible with current and future user agents and assistive technologies; names, roles, and states are programmatically determinable. |

**Conformance target:** Level **AA** is the practical floor for any shipped surface. Level A is insufficient for most regulatory contexts. Level AAA is applied selectively where feasible and never claimed as a blanket for all content types.

---

## 2. Purpose, agency, responsibility (the HIG triad, made non-negotiable)

### 2.1 Purpose

Every surface has **one primary job**, stated in the contract or ticket that authorised it. Decorative motion, secondary panels, and thematic chrome exist only to support that job. If a control does not serve the primary job or a declared secondary obligation, it is refused at design time.

### 2.2 Agency

- The human sends; the agent does not auto-act on the human’s behalf without an explicit, ticketed path.
- Exploration is safe: undo, confirmation for destructive actions, and clear recovery.
- Never force a single path when the contract allows choice.

### 2.3 Responsibility

- No credential capture on product surfaces unless a real authentication mechanism is authorised and measured.
- Identity status is displayed as **PROCEDURAL** (or the exact field value from the store) until authentication exists.
- Privacy and safety take priority over convenience chrome.
- Escalation of never-granted actions goes to a human gate, never to a silent success.

---

## 3. Visual language

### 3.1 Materials over pure tokens

- **Primary surfaces** use real material textures (or named solid fallbacks) with provenance records.
- **Actions and rules** use solid, high-contrast fills.
- Body text never sits directly on fibre. Every textured region that carries body text has a **scrim** (legibility layer). Deleting a scrim “so the texture can show” is a defect.
- Missing required texture **fails closed** or falls back to a named solid. Generative texture alone is not sufficient without a human-selected physical reference when the doctrine requires one.

### 3.2 Palette model (OKLCH-first)

Surfaces should define tokens in OKLCH (or an equivalent perceptually uniform space) and map them to runtime classes the compiler actually knows.

Illustrative roles (adapt per product; do not invent authority colours):

| Role | Intent |
|---|---|
| Desk / shell | Warm neutral base that does not scroll away |
| Paper (human) | Warm, high-lightness sheet for human content |
| Paper (agent) | Cool, high-lightness sheet for agent content |
| Ink | Primary constructive action |
| Nature / active | Measured active or verified *state* (never a grant of authority) |
| Plaque | Session or context chrome (metal or equivalent) |
| Graphite | Primary text on paper |
| Wax / warning | Escalation and never-granted (never on the same row as primary action) |
| Not-obtained | Anything not measured |
| Popit rim | Contextual tool surface edge (neon-lined or soft-glow frame is flavour only; never authority) |
| Bubble (human / peer) | Conversation turn from human or peer channel (colour encodes channel class, not trust grade) |
| Bubble (system) | System or agent turn (distinct hue; still non-authoritative) |
| Exo / technical accent | Near-future technical highlight (amber/orange or cyan on cool neutrals); signals active measured state or critical read, never a grant of authority |
| Diegetic panel | Info rendered as if on equipment or environment (ammo-on-weapon, holographic objective); still copies store status only |

**Contrast:** Body text on scrim meets WCAG 2.2 **1.4.3 Contrast (Minimum)** AA (≥ 4.5:1 normal text, ≥ 3:1 large text). Non-text UI components meet **1.4.11 Non-text Contrast** AA. Composite contrast on texture+scrim is **measured**, not asserted.

### 3.3 Typography

- Display face for plaques and critical session identifiers.
- Body face for message and content text.
- **Monospace for numbers, ticket ids, governor versions, and check digits** — so digits are checkable by eye.
- No runtime dependency on external font loading for core chrome if the build can bake faces.

### 3.4 Spacing and density

- Prefer an **8dp (or 4-unit) scale** (Material Design 1 baseline).
- Content over chrome. TouchWiz-style density that starves content is refused.
- Touch / pointer targets: where the logical viewport allows, meet or exceed **48×48 CSS px** (platform comfort) and never fall below WCAG **2.5.8 Target Size (Minimum)** (24×24 CSS px with spacing exceptions only as defined). Handheld logical viewports that cannot fit 48dp must **state** the shortfall rather than hide it.

### 3.5 Elevation and depth

- Depth communicates hierarchy (iOS “Depth”, MD1 elevation steps), not decoration.
- Layers are honest: a sheet sits on a desk; a warning sheet steps up and dims the desk.
- No elevation theatre that implies authority the store does not grant.
- Popit-style surfaces float above the primary job surface and dim or freeze the underlying content only for the duration of the tool session; closing restores the prior state without silent mutation.

---

## 4. Motion

- **One primary easing curve.** Motion provides meaning (feedback, hierarchy, state change), not noise.
- **Reduced motion is the default** and must be reachable from chrome (not buried). When reduced, decorative lifts and non-essential animation are omitted at render time, not merely documented.
- Forbidden as default behaviour: bounce-for-delight, full-screen flash on send, shake as the only error signal, staggered confetti on ordinary success.
- Animation from interactions must respect WCAG **2.3.3** (AAA, selective) and never cause seizures (**2.3.1**).
- SpringBoard-style “wiggle” for rearrange mode is permitted only as a clear edit-mode signal and must be suppressible under reduced motion.
- Popit open/close may use a short pop or scale-in; it must not block input longer than needed to present the tool surface.

If the runtime has no animation tracks, the design document must not claim motion that does not exist. Register motion work as open or done; do not decorate the specification.

---

## 5. Interaction model

### 5.1 Numbers and locators

A human-readable number may **locate** a record (e.g. a civilian directory entry). It must **never authorise**.

```text
THE NUMBER LOCATES.  IT DOES NOT AUTHORISE.
Only an open CONTRACT + TICKET (or authorised equivalent) opens a session or grants capability.
```

- Assigned centrally and stored; not derived from a hash of the record if that would renumber on rename.
- Check digit (or equivalent) so mistypes fail by arithmetic before lookup.
- Registry responses carry version and digest so the UI can refuse a mismatch.

### 5.2 Primary vs escalation actions

- Primary constructive actions (Send, Confirm ordinary work) use the ink / primary action style.
- Escalation and never-granted actions use warning style, **own row**, never the same visual tier as Send.
- Destructive or irreversible actions require confirmation (forgiveness + user control).

### 5.3 Feedback

- Immediate acknowledgement of input.
- Progress for long operations.
- Explicit refused / blocked states that name what is missing (e.g. “Request contract”) rather than a blank or decorative empty card.
- Clear and end-session paths require confirmation when they discard work.

### 5.4 Keyboard and pointer

- All functionality available from keyboard unless impossible by nature (**2.1.1**).
- No keyboard trap (**2.1.2**).
- Focus order matches meaning (**2.4.3**).
- Focus is at least partially visible and not obscured by author content (**2.4.11** AA).
- Dragging has a non-drag alternative (**2.5.7** AA).
- Target size minimum observed (**2.5.8** AA).

### 5.5 Predictability and help

- Consistent navigation and help placement (**3.2.6** Consistent Help).
- No unexpected context changes on focus alone without a mechanism to control them.
- Errors are identified, described in text, and suggestions offered where feasible (**3.3.x**).

### 5.6 Direct manipulation (Popit cursor discipline)

When a surface offers object-level editing (layout, layer, attachment, property tweak):

- Selection, move, scale, rotate, layer change, duplicate, and delete are first-class and reversible where the contract allows.
- Multi-select via hold-and-drag selection box is permitted; the selection acts as a unit until ungrouped or detached.
- “Glue / attach” and “detach” are explicit operations; they never silently become permanent without an evidence-backed write.
- While a tool session is open, underlying live behaviour may be frozen or outlined; closing the tool resumes prior behaviour without inventing new state.
- Every editable object may expose a **tweak / info page** that lists only fields the store and contract already admit; unknown fields are refused, not invented.
- Mode scope is enforced: tools available in a create-style contract are absent or disabled under a play/read-only contract. Absence is preferred over a disabled control that looks like a grant.

---

## 6. Accessibility (WCAG 2.2 mapped to the stack)

### 6.1 Required floor (Level AA)

Any shipped surface must be designed and tested to meet all applicable Level A and Level AA success criteria of WCAG 2.2, including for non-web software via WCAG2ICT guidance where the runtime is not a browser document.

Critical clusters:

- **1.1** Text alternatives for non-text content  
- **1.3** Info and relationships, meaningful sequence, sensory characteristics not as sole means  
- **1.4** Contrast, resize text, reflow, non-text contrast, text spacing, content on hover/focus  
- **2.1** Keyboard  
- **2.2** Enough time  
- **2.3** Seizures and physical reactions  
- **2.4** Navigable (bypass blocks, page/screen titled, focus order, link purpose, focus visible / not obscured)  
- **2.5** Input modalities (pointer gestures, target size, dragging alternatives)  
- **3.1** Readable  
- **3.2** Predictable (including consistent help)  
- **3.3** Input assistance (including redundant entry, accessible authentication paths)  
- **4.1** Compatible (name, role, value; status messages)

### 6.2 Texture + scrim and contrast

Composite contrast is a **judgement that must be measured** (sampled pixels or equivalent), not asserted from token names. Region variance that proves material is visible is a different claim from AA contrast ratio on text.

### 6.3 Authentication and cognitive load

Prefer paths that do not rely on memorising or transcribing (WCAG **3.3.8** Accessible Authentication). Until real auth exists, keep identity procedural and never present a string field as verified identity.

### 6.4 Conformance claims

A surface may claim “designed to WCAG 2.2 AA” only when mechanical and judgement checks are recorded. W3C conformance logos do not constitute third-party certification. Do not invent a “verified accessible” badge that the evidence ladder does not support.

---

## 7. Information architecture and content

### 7.1 Hierarchy

- Primary task content first.
- Session / contract / ticket / governor version / identity status visible without a secondary screen when the surface is a governed session (rule 143 family).
- Secondary chrome and thematic lens content never compute control status.

### 7.2 Language

- User-centric, plain language.
- Monospace for identifiers and numbers.
- Error and refusal copy names the policy or missing artefact when possible (a refusal that names the allowed vocabulary is preferred to a silent drop).

### 7.3 Empty, blocked, and refused states

- Empty: still a desk / material surface, not a pure white void that denies the product register.
- Blocked: names the missing authorisation (contract, ticket, evidence).
- Refused: distinct from error; may use warning material; never looks like success.

---

## 8. Build and runtime discipline

### 8.1 Class / style table

If the runtime compiles literal class strings into a style table, **only strings present in the table are styles**. An unknown token that is silently dropped is a defect class: source may pass while the app throws or paints without the intended scrim/border. Build checks must compare used families against the compiler palette and assert every required literal has a style record.

### 8.2 Textures and assets

- Provenance file (artist, licence, source, hash) for every material asset.
- Licence gate: public domain / CC0, or CC BY(-SA) with named author; non-commercial and unrecognised licences refused.
- Power-of-two or otherwise compiler-required geometry enforced at bake time.
- UI copy that refers to assets states what is true (e.g. “real material · provenance in …”) and does not hard-code counts that go stale.

### 8.3 Conformance as measurement

Separate:

| Check type | Means |
|---|---|
| **Mechanical** | String, class, file, or schema either exists or not; re-runnable by anyone |
| **Judgement** | Needs rendered frame, sampled contrast, or human review; scored only when measured and named |

A mechanical pass means the requirement is present in source or build artefacts. It does not mean the surface looks right or meets contrast in the framebuffer. Headless or offscreen render tests catch throws and blank trees; pixel sampling catches contrast.

### 8.4 Reduced motion and preference

Preference is visible in chrome. Default is reduced. Implementation gates class strings or animation tracks at render time.

---

## 9. Overlay, lens, and fiction layers

- Any city / overlay / directory / idle layer is a **lens**: it copies status from the store; it never mutates doctrine, income, grade, promotion, retirement, resume, or metrics.
- Fiction / idle jobs stay pathless (no path to the controlled records above).
- Thematic chrome (Neon, 1940s, scientist, Nature UX greens, Popit neon rim, SpringBoard glass dock, Advanced Warfare exo/technical accent and holographic panels) is flavour only.

---

## 10. Do / Don’t (summary)

**Do**

- One primary job per surface.  
- Real materials on primary surfaces when obtained; scrims under body text.  
- Show `contract_id · ticket_id · governor_version · identity_status` when in a governed session.  
- Solid high-contrast fills for actions.  
- Monospace numbers and ids.  
- Reduced motion default; reachable control.  
- Name refusals and missing artefacts.  
- Meet WCAG 2.2 AA for applicable criteria.  
- Measure composite contrast and frame health.  
- Keep appearance non-authoritative.  
- Scope tool bags and cursor actions to the current contract.  
- Distinguish conversation channel (bubble colour) from trust or grade.  
- Keep Dock, folders, and search as locators, never as capability grants.  
- Prefer diegetic or contextual placement of status (on the relevant object or session chrome) when it improves clarity without inventing authority.  
- Use thin modular panels and high-contrast critical reads for stress surfaces; keep secondary data peripheral.  
- Treat loadout / point-budget UIs as organisational only unless a contract already authorises the underlying capability.

**Don’t**

- Neon-on-black “AI chrome”, gradient text as hierarchy, equal-weight card grids that erase priority.  
- Hide PROCEDURAL identity behind a verified badge.  
- Put never-granted actions on the same row or tier as Send.  
- Render a number the registry did not supply.  
- Let texture noise drop body text below AA.  
- Delete a scrim so the texture “deserves to be seen”.  
- Claim motion, contrast, or accessibility that has not been measured.  
- Let historical or thematic content widen Governor Base.  
- Fail open on governance, schema, or unknown ops.  
- Treat a hearted / personal bag, a sticker, or a material placement as income, grade, or promotion.  
- Let a multitasking drawer or Notification Center invent status the store does not hold.  
- Equate blue/green (or any) bubble colour with verified identity or elevated privilege.  
- Treat holographic, exo, or “advanced technical” chrome as proof of clearance, income, or Governor capability.  
- Let a virtual test range or loadout preview silently commit a ticket or spend resources without explicit confirmation.  
- Use threat-ping or highlight FX to invent entities the store does not report.

---

## 11. Definition of done (design)

A surface is design-complete for shipping under this doctrine when:

1. Purpose, agency, and responsibility are explicit in the authorised contract/ticket.  
2. All applicable WCAG 2.2 A/AA criteria are designed for and tested (mechanical + judgement where required).  
3. Identity is shown as procedural (or measured auth status); no false verified chrome.  
4. Never-granted actions are absent or human-gated and visually demoted.  
5. Texture + scrim discipline holds; composite contrast measured or explicitly not-obtained.  
6. Build style table contains every class the UI can emit; unknown tokens fail closed.  
7. Reduced motion default is implemented, not only documented.  
8. Empty / blocked / refused states are distinct and informative.  
9. No path exists from chrome, fiction, overlay, Popit bag, sticker, or material into income, grade, promotion, retirement, resume, metrics, or doctrine.  
10. Conformance report separates mechanical passes from judgement items still open.  
11. SpringBoard-style home, Dock, folders, Spotlight, multitasking, and Notification Center behaviours (when present) are pure navigation and reporting surfaces.  
12. Messages-style composition and bubble semantics encode channel class only; they never encode trust grade or capability.  
13. Any Popit-style tool surface is contract-scoped, reversible where required, and closes without silent state invention.  
14. Advanced Warfare–style diegetic HUD, loadout budgets, operator presentation, and technical accent language remain non-authoritative; highlights and pings only reflect store-supplied state.

---

## 12. SpringBoard patterns (iOS 6 home, Dock, folders, search, multitasking, notifications)

These patterns are adopted as **navigation and organisation metaphors**. They do not create Governor Base capability, open contracts, or resolve open decisions. Status badges and labels are copied from the store.

### 12.1 Home grid and pages

- Icons (or tiles) arranged in a regular grid; multiple pages swipe horizontally.
- Contents are **loaded**, not compiled into the binary (TouchWiz lineage + SpringBoard).
- Last-viewed page is restored on return from an app or session where the product allows it.
- Wallpaper is decorative; it never carries control status or authority chrome.

### 12.2 Dock

- Fixed strip at the bottom (or product-equivalent edge) that remains visible across home pages.
- May hold a small number of primary locators and, where the product supports it, folders.
- Dock items are shortcuts to surfaces already authorised by contract or standing operator policy; placing an item in the Dock does not grant new capability.

### 12.3 Folders

- Created by dropping one locator onto another (or an explicit “new folder” action).
- Open folder shows a limited grid; in the iOS 6 model pages inside folders were constrained — products may extend page count only if measured and declared.
- Closing a folder returns to the home grid; the Dock may remain usable or dimmed consistently with the product’s stated rule (iOS 6 dimmed the Dock under an open folder; either choice must be documented and stable).
- Folders organise locators; they never organise authority.

### 12.4 Wiggle / rearrange mode

- Entered by long-press or equivalent explicit gesture.
- Icons jiggle (or an equivalent reduced-motion indicator) to signal edit mode.
- Drag to reorder, to create/remove folders, or to move to Dock / another page.
- Exit by explicit Done / Home / equivalent; no silent exit that leaves partial moves unconfirmed.
- Under reduced motion, replace continuous wiggle with a static “edit” badge or outline.

### 12.5 Spotlight / search

- Invoked by swipe-down on the home grid (or left of first page, or product-equivalent).
- Searches loaded locators, titles, and declared metadata; results may show containing folder when known.
- Search never invents records; empty results state the query and the scope searched.
- Results are locators only; activation still requires the normal contract/ticket path for any governed session.

### 12.6 Multitasking drawer (iOS 6 style)

- Invoked by double-activation of the home/primary navigation control (or product-equivalent).
- Horizontal row of recent surfaces (classically ~4 visible, swipe for more).
- Selection restores that surface; swipe-off or explicit close may remove from the recent list without claiming the process is destroyed if the runtime does not support true kill.
- The drawer reports recency; it does not report “running with elevated privilege” or invent ticket state.

### 12.7 Notification Center

- Invoked by swipe-down from the top edge (or product-equivalent).
- Aggregates alerts, messages, and system notices **copied from the store or declared channels**.
- Actions on a notification open the corresponding surface or escalate; they do not silently authorise never-granted actions.
- Clear / manage actions are explicit and reversible where the store allows.
- Do Not Disturb (or equivalent quiet mode) is a user preference, not a capability grant; its indicator is status only.

### 12.8 Lock / idle surface (optional)

- May show time, date, and a constrained set of notifications.
- Camera or other quick-launch gestures (e.g. swipe-up) open only surfaces already permitted under current Governor Base.
- Unlock / resume does not treat a string field as verified identity.

### 12.9 SpringBoard mapping under AGENTS.md

| iOS 6 element | Product meaning | Forbidden reading |
|---|---|---|
| Icon / tile | Locator to a surface or record | Capability grant |
| Badge | Count or state copied from store | Trust grade or promotion |
| Folder | Organisational group | Authority domain |
| Dock | Persistent shortcuts | Elevated privilege strip |
| Spotlight | Search over loaded content | Adjudication of open decisions |
| Multitasking row | Recent surfaces | Proof of concurrent authorised work |
| Notification Center | Aggregated alerts | Silent contract creation |
| Wallpaper / theme | Flavour | Doctrine or identity proof |

---

## 13. Messages patterns (iOS 6 Messages / iMessage composition)

Adopted as the **conversation surface** pattern for human–agent and human–human channels under AGENTS.md civilian-chat and related rules (including proposed rules 136–144). Appearance of a conversation never opens a session by itself; session open still requires Contract + Ticket (or proposal ticket) where doctrine requires it.

### 13.1 Conversation list

- Chronological or pinned list of threads.
- Each row shows peer/channel label, last snippet, timestamp, and unread indicator when the store supplies them.
- Rows are locators; opening a row enters the thread surface under the same session rules as the parent surface.

### 13.2 Thread surface

- Vertical timeline of turns.
- **Bubble colour encodes channel class**, not trust or grade:
  - Distinct hue for internet/data-style peer channel (historically “blue” iMessage).
  - Distinct hue for carrier/SMS-style channel (historically “green”).
  - Distinct treatment for system/agent turns.
- Colour is a legibility and classification aid; it must never be presented as “verified identity” or “elevated clearance.”
- Delivery / read indicators appear only when the store or channel protocol supplies them; they are not invented client-side.

### 13.3 Composition

- Single composition field with clear Send (primary ink style).
- Optional attachment affordance for media the contract permits; never-granted attachment types escalate or refuse with a named reason.
- Typing indicators may be shown when the peer channel reports them; absence is not treated as “idle authority.”
- Send is the primary constructive action; escalation and never-granted actions stay on a separate row or sheet.

### 13.4 Group threads

- Named group, member list, and add/remove only under explicit authorisation.
- System messages for membership changes are labelled as such.
- Group does not create a new governance plane.

### 13.5 Share and entry points

- Share sheet / “send to Messages” from other surfaces may pre-fill a draft; it does not auto-send without user confirmation.
- Entry from Notification Center or Dock follows the same session rules as any other locator.

### 13.6 Messages mapping under AGENTS.md

| iOS 6 / Messages element | Product meaning | Forbidden reading |
|---|---|---|
| Blue / green (or product) bubble | Channel class | Verified identity, grade, or clearance |
| Typing indicator | Peer activity signal | Proof the peer is authorised |
| Delivered / Read | Protocol status when supplied | Evidence-grade claim without store record |
| Attachment | Media allowed by contract | Silent upload of credentials or money movement |
| Group | Multi-party thread | New authority domain |
| Composition Send | Primary constructive action | Auto-act without human initiation |

Chat sessions remain non-authoritative by construction (rule 136 family): message records may not create income, grade, promotion, retirement, resume claims, metrics, or doctrine changes.

---

## 14. Popit patterns (LittleBigPlanet contextual tool surface)

Popit is adopted as a **contextual, mode-scoped tool surface** that pops near the focus of attention. It is not a second Governor Base. Tools, bags, and decorations are available only to the extent the current contract and ticket already allow.

### 14.1 Open / close behaviour

- Explicit control opens the Popit surface (button, gesture, or menu equivalent).
- It appears above the primary job surface, optionally with a soft rim or glow that is flavour only.
- Underlying content may be dimmed or physics/behaviour frozen for the duration of the tool session.
- Close restores the prior view and behaviour; no silent writes on close.

### 14.2 Category bags (illustrative, contract-scoped)

| Bag / panel | Role under this doctrine | Must not |
|---|---|---|
| **Tools** | Actions and gadgets the current contract admits (cursor, attach, detach, tweak, layer, etc.) | Invent ops outside Governor / contract vocabulary |
| **Goodies / materials** | Inventory of placeable materials and objects already evidenced or granted by the session | Create materials without evidence; path into income or grade |
| **Personal / hearted** | User-marked favourites for quick access | Treat “heart” as reputation, grade, or promotion signal |
| **Stickers & decorations** | Cosmetic and annotation layers | Imply structural or legal change; path into metrics |
| **Character / appearance** | Avatar or presentation customisation | Claim identity verification or capability change |
| **Info / tweak** | Per-object property sheet limited to declared fields | Expose or invent fields the schema forbids |

Availability of each bag is **mode- and contract-dependent**. A read-only or play-scoped contract omits create-only bags rather than showing them disabled as if granted.

### 14.3 Popit cursor (direct manipulation)

- Select object → move, scale, rotate, change layer/depth, duplicate, delete, attach (glue), detach.
- Multi-select via hold-and-drag region where the product supports it.
- Tweak menu opens on explicit control; lists only schema-admitted properties.
- Outline or freeze mark shows objects under edit; closing Popit clears the mark and resumes behaviour.
- All mutations that affect the store require the normal evidence-linked write path; the cursor is a UI affordance, not a silent writer.

### 14.4 Stickers, decorations, and materials

- Placement is visual and, where the product models physics, may react to the environment.
- Stickers and decorations are annotations; they do not alter legal description, ownership, grade, or ticket state.
- Materials drawn on a surface follow the material-drawing discipline (closed elemental vocabulary, manufacturing origin when required, no invented performance numbers). Drawing a material never implies a grade, R-value, or strength claim unless the store already holds a verified number.

### 14.5 Hearting / personal list

- A “heart” or favourite mark is a personal organisation aid.
- It may filter the Personal bag; it must not feed resume claims, promotion strength, income, or metrics.
- Hearting is reversible and local to the user’s organisational view unless the contract explicitly shares favourites as non-authoritative data.

### 14.6 Popit mapping under AGENTS.md

| Popit element | Product meaning | Forbidden reading |
|---|---|---|
| Popit open | Contextual tool session | New authority plane |
| Tools bag | Contract-scoped actions | Unlimited create power |
| Goodies / materials | Inventory under evidence | Free mint of assets or currency |
| Personal / hearted | Favourite filter | Reputation or grade |
| Stickers & decorations | Cosmetic / annotation | Structural or legal change |
| Cursor mutate | Direct manipulation UI | Silent store write without ticket/evidence |
| Tweak page | Declared property editor | Schema expansion by the client |
| Character customisation | Presentation | Verified identity or capability |

### 14.7 Alignment with material and house drawing doctrine

Where the product implements material drawing (house.md / periodic-table plans under AGENTS.md appendices):

- Callouts and layer order remain exclusive drawing authority of the house doctrine.
- Popit may expose only callouts and layers already in the registry.
- Manufacturing-process and elemental composition data are read-only annotations when present; they never become claims without evidence ids.
- Absence suite still holds: no path from material or decoration into income, ticket, grade, promotion, retirement, resume, metric, or bill.

---

## 15. Advanced Warfare patterns (Call of Duty: Advanced Warfare — Sledgehammer Games)

Adopted as **near-future technical product language** and **competitive clarity patterns**. Art direction and UI from Advanced Warfare supply grounded futurism and diegetic status placement; they do not supply combat authority, exo powers, or any path into Governor Base capability, income, grade, or promotion.

### 15.1 Art direction triad (Joe Salud / Sledgehammer)

Three principles guided Advanced Warfare’s look; they map cleanly onto this doctrine:

| Principle | Meaning in AW | Meaning under AGENTS.md / UNIFIED_DESIGN |
|---|---|---|
| **Advanced technical look** | Exos, modular armour, clean industrial surfaces, holographic and panel UI | Technical materials, thin modular chrome, diegetic panels; still non-authoritative |
| **Legitimately military** | Functional military silhouettes, readable kit, no pure fantasy armour | Serious-task surfaces stay unobtrusive and predictable (HIG aesthetic integrity); no decorative “war badge” that implies clearance |
| **Relatability / plausible reality** | Reconfigured real-world tech into near future; documentary feel (District 9 / Hurt Locker influences); PBR and captured skies | Prefer recognisable real materials and behaviours; first/second/third read of any object must not invent authority; grounded > pure neon sci-fi |

**Plausible reality (first / second / third read):** At distance, silhouette and material family; at arm’s length, modular detail and accent; on inspection, micro-detail and wear. None of these reads may be used to claim verified identity, grade, or capability the store does not hold.

### 15.2 Diegetic and contextual HUD

- Prefer placing critical status **on the relevant object or session chrome** when it improves clarity (AW placed ammo contextually on the weapon — a first in the series — while keeping the read minimalistic and advanced).
- Objectives, waypoints, and secondary data may use holographic or panel treatments that feel embedded in the environment or equipment.
- Custom layouts per context (vehicle, special tool, session type) are encouraged when the product has distinct modes; each layout still only reports store-backed state.
- The HUD is a reporting layer, not an adjudication layer. Removing or minimising floating chrome in favour of diegetic placement is allowed when measured contrast and accessibility still pass.

### 15.3 Competitive styleguide (multiplayer / high-stress surfaces)

From AW’s multiplayer UI styleguide and briefing-style motion graphics:

- **Thin line work**, modular rectangular panels, restrained grids — complexity without boxiness.
- **High-contrast critical information** at primary focus; secondary data peripheral.
- **Accent colour** (commonly amber/orange or cyan against cool neutrals) for active or critical state only; accent is status, not grant.
- Briefing / infographic motion may lead the eye through ordered reveals; it must not invent entities or outcomes.
- Instant readability under stress beats decorative density. Silence is not feedback; clutter is not clarity.

### 15.4 Loadout, point budget, and virtual test range

- **Point-budget / Pick-style loadouts** (AW’s Pick 13) are organisational UIs: the human allocates limited slots among options the contract already admits. Spending a point in the UI does not create capability the Governor has not authorised.
- **Virtual test / firing range** pattern: preview behaviour of a selection without committing a live ticket or consuming real resources. Explicit confirm is required before any store write or ticket open.
- Operator / character customisation (heads, gear, exo silhouette in AW) is **presentation only** — same rule as Popit character customisation and Messages identity: PROCEDURAL until measured auth exists; cosmetics never raise support level or open contracts.

### 15.5 Status highlighting and “ping”

- Threat detection, Exo Ping–style highlights, and environmental callouts may illuminate entities or events **only when the store or an authorised sensor feed reports them**.
- Highlight colour and animation are feedback, not evidence creation. Fabricating a ping for an unrecorded entity is a defect.
- Cooldown, battery, or resource meters (exo battery metaphor) map to declared resource fields on the contract or ticket; they never invent a balance the ledger does not hold (C09-style discipline: no stored balance invention).

### 15.6 Materials and industrial language

- Prefer matte technical panels, restrained metal, and honest wear over pure emissive sci-fi fill.
- Accent strips and edge lighting may mark active modules; they remain flavour under rules 112 / 120 / 136–143.
- Where the product also uses house.md / material-drawing doctrine, AW technical language does not override closed callouts or elemental registries; it supplies surface treatment only.

### 15.7 Advanced Warfare mapping under AGENTS.md

| AW element | Product meaning | Forbidden reading |
|---|---|---|
| Exo / advanced technical chrome | Near-future material and panel language | Clearance, grade, or Governor capability |
| Diegetic ammo / status on object | Contextual status placement | Authority to fire or act without ticket |
| Holographic objective / briefing | Embedded status and ordered reveal | Evidence-grade claim or new doctrine |
| Accent (amber/cyan) highlight | Active or critical measured state | Verified identity or promotion |
| Pick / point-budget loadout | Organisation of already-admitted options | Creation of new ops or spend without contract |
| Virtual test range | Preview without commit | Silent ticket open or resource spend |
| Operator customisation | Presentation / cosmetic | Resume inflation or identity verification |
| Threat ping / highlight FX | Store- or sensor-backed highlight | Invented entities or hostile labels without evidence |
| Multiplayer styleguide clarity | High-stress readability | Combat simulation as product law |

### 15.8 Alignment with existing patterns

- **With SpringBoard:** Home / Dock / folders remain locators; AW technical panels may skin them but must not turn them into a loadout that grants capability.
- **With Messages:** Bubble channel class stays distinct from AW accent colour; accent is for system critical state, not peer trust.
- **With Popit:** Tools and materials bags stay contract-scoped; AW diegetic placement may put a bag’s status on the tool surface without widening the bag’s powers.
- **With material drawing:** Manufacturing origin and elemental grounding still required where those plans apply; AW “advanced technical look” does not invent performance numbers.

---

## 16. Document maintenance

- Corrections to this file are **append-only** with explicit correction objects (old value, new value, reason, authority), consistent with AGENTS.md.  
- New platform-specific tokens or class maps may be added as product appendices; they may not weaken §0–§2.  
- Open questions in AGENTS.md remain open; this document does not resolve them.  
- Popit, SpringBoard, Messages, and Advanced Warfare patterns added here are **design patterns under existing doctrine**, not new binding rules numbered in AGENTS.md §17. Any elevation to binding rule status requires the normal Governor / contract process.

---

## 17. Quick reference: principle crosswalk

| Concern | AGENTS.md | DESIGN.md / product | WCAG 2.2 | iOS 6 / classic HIG | Popit (LBP) | Advanced Warfare |
|---|---|---|---|---|---|---|
| Who may act | Governor Base > Contract > Ticket | Appearance grants no authority | Operable, understandable auth paths | User control, agency | Tools only within current mode/contract | Loadout points organise admitted options only |
| Truth | Evidence record | Surfaces report, do not adjudicate | Robust name/role/state | Feedback, honesty of materials | Tweak pages show declared fields only | Diegetic HUD and pings copy store/sensor state |
| Primary job | Mission in contract | Purpose | — | Focus on primary task, deference to content | Popit is secondary; closes to primary job | Critical read at centre; secondary peripheral |
| Touch / pointer | — | Target sizes stated | 2.5.7, 2.5.8 | Fingertip-size targets | Cursor + target size for bags | High-stress targets remain AA-compliant |
| Contrast / text | — | Scrim + AA targets | 1.4.3, 1.4.11, … | Clarity, legibility | Bubble and bag text on scrim | Accent on critical only; measured composite |
| Motion | — | One curve; reduced default | 2.2.2, 2.3.1, 2.3.3 | Feedback without noise | Pop open/close; optional freeze outline | Briefing reveals; no seizure or decorative flash |
| Identity | Procedural until measured | Field on plaque | 3.3.8 accessible auth | Responsibility, no false trust | Character panel is presentation only | Operator customisation is presentation only |
| History / theme | Never authorises | Lens only | — | Metaphor without false authority | Stickers/decorations are lens/annotation | Plausible-future chrome is flavour only |
| Errors | Named refusal preferred | Named missing artefact | 3.3.x input assistance | Forgiveness, feedback | Unknown tool/callout refused with vocabulary | Unknown loadout option refused with vocabulary |
| Organisation | — | Numbers locate | — | SpringBoard grid, Dock, folders, Spotlight | Bags organise tools; do not grant power | Point budget + virtual test range; no silent commit |
| Conversation | Chat non-authoritative (136+) | Channel ≠ trust | — | Messages bubbles, composition, receipts | Text chat panel is communication only | Accent ≠ channel; channel ≠ trust |

---

**End of UNIFIED_DESIGN.md**

This document is the single design entry point for any application compiled against AGENTS.md. Product-specific DESIGN.md files (e.g. civilian-chat) specialise tokens, class maps, and viewports; they do not override the constraints in §0–§2 or the WCAG AA floor. SpringBoard, Messages, Popit, and Advanced Warfare sections supply concrete interaction and art-direction patterns; they remain subordinate to AGENTS.md precedence and to the non-authority of appearance.
