# Skills

Skills are reusable, composable workflows invoked with `/<skill-name>`. They can chain other skills and maintain context across steps.

## Skill Categories

### Meta
| Skill | When to Use |
|-------|-------------|
| **using-superpowers** | Injected at session start - how to find and use skills |

### Planning & Design
| Skill | When to Use |
|-------|-------------|
| **brainstorming** | BEFORE any creative work - explores ideas into designs |
| **writing-plans** | After design - creates bite-sized implementation tasks |
| **interview** | Gathering detailed requirements via Q&A |

### Execution
| Skill | When to Use |
|-------|-------------|
| **using-git-worktrees** | Start feature work - create isolated workspace |
| **executing-plans** | Execute plan with human checkpoints (parallel session) |
| **subagent-driven-development** | Execute plan with auto-review (same session, faster) |
| **dispatching-parallel-agents** | Multiple independent failures - debug in parallel |
| **finishing-a-development-branch** | Complete work: merge, PR, keep, or discard |

### Quality System
| Skill | When to Use |
|-------|-------------|
| **test-driven-development** | Always - write failing test before any production code |
| **verification-before-completion** | Before ANY completion claim - evidence before assertions |
| **requesting-code-review** | After implementing - dispatch reviewer subagent |
| **receiving-code-review** | When handling feedback - verify before implementing |

### Debugging & Review
| Skill | When to Use |
|-------|-------------|
| **systematic-debugging** | ANY bug/failure - four phases before proposing fixes |
| **dispatching-parallel-agents** | Multiple independent failures - debug in parallel |
| **review-pr** | Comprehensive PR review |

---

## The Complete Development Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           PLANNING PHASE                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  /brainstorming ──→ /writing-plans                                       │
│       │                   │                                              │
│  Q&A one at a time   TDD + exact code                                   │
│  Propose approaches  Exact file paths                                    │
│  Validate sections   Bite-sized tasks                                    │
│       │                   │                                              │
│       ▼                   ▼                                              │
│  *-design.md         *-plan.md                                          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          SETUP PHASE                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  /using-git-worktrees                                                    │
│                                                                          │
│  1. Find/create worktree directory (.worktrees/ preferred)               │
│  2. Verify directory is gitignored (safety)                              │
│  3. Create worktree with new branch                                      │
│  4. Install dependencies (auto-detect)                                   │
│  5. Run tests (verify clean baseline)                                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          EXECUTION PHASE                                 │
│                     (Choose one path)                                    │
├──────────────────────────────┬──────────────────────────────────────────┤
│                              │                                           │
│  /executing-plans            │  /subagent-driven-development            │
│  (parallel session)          │  (same session)                          │
│                              │                                           │
│  • Batch of 3 tasks          │  • Fresh subagent per task               │
│  • Human checkpoint          │  • Two-stage auto-review:                │
│  • Report → feedback         │    1. Spec compliance                    │
│  • Next batch                │    2. Code quality                       │
│                              │                                           │
│  Best for:                   │  Best for:                               │
│  • Risky changes             │  • Independent tasks                     │
│  • Learning codebase         │  • Speed                                 │
│                              │                                           │
└──────────────────────────────┴──────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │   Quality Gates (per task)    │
                    ├───────────────────────────────┤
                    │                               │
                    │  /test-driven-development     │
                    │  (implementer uses TDD)       │
                    │           │                   │
                    │           ▼                   │
                    │  /requesting-code-review      │
                    │  (dispatch reviewer)          │
                    │           │                   │
                    │           ▼                   │
                    │  /receiving-code-review       │
                    │  (handle feedback)            │
                    │                               │
                    └───────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          COMPLETION PHASE                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  /finishing-a-development-branch                                         │
│                                                                          │
│  1. Verify tests pass                                                    │
│  2. Present options:                                                     │
│     • Merge locally                                                      │
│     • Push + create PR                                                   │
│     • Keep as-is                                                         │
│     • Discard                                                            │
│  3. Execute choice                                                       │
│  4. Cleanup worktree                                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Quality System Deep Dive

### Test-Driven Development

**Iron Law:** No production code without a failing test first.

```
RED → GREEN → REFACTOR → REPEAT
 │       │        │
 │       │        └── Clean up (stay green)
 │       └── Minimal code to pass
 └── Write failing test, watch it fail
```

See: `test-driven-development/SKILL.md` and `testing-anti-patterns.md`

### Two-Stage Code Review

Used by `/subagent-driven-development`:

```
Implementer completes task
         │
         ▼
┌─────────────────────┐
│ Spec Compliance     │  "Did they build what was requested?"
│ Review              │  • Missing requirements?
│                     │  • Extra/unneeded work?
│                     │  • Misunderstandings?
└─────────────────────┘
         │
         ▼ (only if spec compliant)
┌─────────────────────┐
│ Code Quality        │  "Is it well-built?"
│ Review              │  • Clean code?
│                     │  • Proper tests?
│                     │  • Good architecture?
└─────────────────────┘
```

### Receiving Code Review

**Core principle:** Verify before implementing. Technical correctness over social comfort.

**Forbidden:**
- "You're absolutely right!" (performative)
- Blind implementation without verification
- Accepting suggestions that break things

**Required:**
- Restate technical requirement
- Verify against codebase reality
- Push back with reasoning if wrong

See: `receiving-code-review/SKILL.md`

### Verification Before Completion

**Iron Law:** No completion claims without fresh verification evidence.

```
BEFORE claiming "done", "fixed", "passes":
  1. IDENTIFY what command proves this
  2. RUN the command (fresh, complete)
  3. READ full output, check exit code
  4. VERIFY output confirms claim
  5. ONLY THEN make the claim
```

**Red flags:**
- "Should pass now" / "Looks correct" (no evidence)
- Expressing satisfaction before running verification
- Trusting agent success reports without checking

See: `verification-before-completion/SKILL.md`

### Parallel Agent Debugging

When facing multiple independent failures:

```
3 test files failing with different causes?
         │
         ▼
    Are they independent?
         │
    Yes ─┴─→ Dispatch one agent per problem domain
              │
              ├─ Agent 1 → Fix file A (abort logic)
              ├─ Agent 2 → Fix file B (batch completion)
              └─ Agent 3 → Fix file C (race conditions)
              │
              ▼
         Review + integrate all fixes
```

**Don't use when:** Failures are related, need full system context, agents would conflict.

See: `dispatching-parallel-agents/SKILL.md`

### Systematic Debugging

**Iron Law:** No fixes without root cause investigation first.

```
Phase 1: ROOT CAUSE INVESTIGATION
  • Read error messages carefully (don't skip!)
  • Reproduce consistently
  • Check recent changes
  • Gather evidence in multi-component systems
  • Trace data flow backward

Phase 2: PATTERN ANALYSIS
  • Find working examples
  • Compare against references
  • Identify differences

Phase 3: HYPOTHESIS TESTING
  • Form single hypothesis
  • Test minimally (one variable)
  • Verify before continuing

Phase 4: IMPLEMENTATION
  • Create failing test (TDD)
  • Implement single fix
  • Verify fix
  • If 3+ fixes failed → question architecture
```

**Supporting techniques:**
- `root-cause-tracing.md` - Trace bugs backward through call stack
- `condition-based-waiting.md` - Replace arbitrary timeouts with condition polling
- `defense-in-depth.md` - Validate at every layer to make bugs impossible

See: `systematic-debugging/SKILL.md`

---

## Execution Path Decision

```
Have a plan ready to execute?
         │
         ▼
Want human review between batches?
    │
    ├── Yes → /executing-plans
    │         • Parallel session
    │         • Batches of 3 tasks
    │         • You review each batch
    │
    └── No  → /subagent-driven-development
              • Same session
              • Auto-review (spec + quality)
              • Faster iteration
```

| Factor | `/executing-plans` | `/subagent-driven-development` |
|--------|-------------------|-------------------------------|
| Session | Parallel (new window) | Same session |
| Human review | Every 3 tasks | Only if subagent asks |
| Review type | Human checkpoints | Auto (spec + quality) |
| Speed | Slower (waits for you) | Faster (continuous) |
| Best for | Risky, learning | Independent, speed |

---

## Directory Structure

```
common/skills/
├── README.md                          # This file
├── using-superpowers/                 # Meta: how to use skills
├── brainstorming/                     # Idea → Design
├── writing-plans/                     # Design → Tasks
├── interview/                         # Requirements gathering
├── using-git-worktrees/               # Isolated workspace setup
├── executing-plans/                   # Tasks → Code (human checkpoints)
├── subagent-driven-development/       # Tasks → Code (auto-review)
├── test-driven-development/           # TDD methodology
├── verification-before-completion/    # Evidence before claims
├── requesting-code-review/            # Dispatch reviewer
├── receiving-code-review/             # Handle feedback
├── systematic-debugging/              # Four-phase debugging
├── dispatching-parallel-agents/       # Parallel debugging
├── finishing-a-development-branch/    # Completion workflow
├── review-pr/                         # PR review
└── using-prog/                        # Progress tracking
```

---

## Quick Reference

| Phase | Skill | Output |
|-------|-------|--------|
| Explore | `/brainstorming` | `*-design.md` |
| Plan | `/writing-plans` | `*-plan.md` with bite-sized tasks |
| Setup | `/using-git-worktrees` | Isolated workspace with clean baseline |
| Build | `/executing-plans` or `/subagent-driven-development` | Working code |
| Quality | `/test-driven-development` | Tests first, then code |
| Verify | `/verification-before-completion` | Evidence before claims |
| Review | `/requesting-code-review` | Reviewer feedback |
| Feedback | `/receiving-code-review` | Verified fixes |
| Debug | `/systematic-debugging` | Root cause found, fix verified |
| Parallel Debug | `/dispatching-parallel-agents` | Multiple independent fixes |
| Complete | `/finishing-a-development-branch` | Merged/PR'd/kept + worktree cleanup |

---

## Skills to Add

- **api-design** — Design API with OpenAPI spec
- **db-schema** — Design schema with migrations
- **trace-request** — Trace through distributed system
- **oncall-investigate** — Production incident workflow
- **dockerfile** — Create optimized Dockerfile
- **ci-pipeline** — Design CI/CD pipeline

---

## Related

For project-specific skills, see:
- `web-app/skills/` — React, Next.js, TanStack Router patterns
