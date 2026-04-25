---
title: Introduction
description: What Ought is and why it exists.
order: 1
---

Ought is a spec-driven verification system. You write what your system **ought** to do as plain markdown — the intent. An LLM reads your spec and your source, generates tests for every clause, and `ought run` reports each result back against the spec.

When agents write more of the code, the work shifts upstream to intent and downstream to verification. Ought is the spec-driven verification half of that loop.

## The problem

Test intent and test implementation are fused together in code. The assertion `assert_eq!(response.status(), 401)` buries the intent — _"invalid credentials must return 401"_ — inside mechanical setup and plumbing. When requirements change, you rewrite test code instead of updating a sentence.

Ought separates the two: intent stays in the spec; the LLM handles the mechanical work.

## What's different

Traditional test frameworks give you a way to write assertions about source code. Ought adds a layer above that: a spec that says, in plain language, what the assertions are _for_. Every test is traceable back to a clause in a spec, and every clause in a spec is traceable forward to the tests that enforce it.

Intent → test → result → intent. The loop closes automatically.

## Two starting points

Greenfield projects describe intent first, then `ought generate` writes the tests. Existing codebases run `ought extract` to draft specs from current behavior, then iterate.

## Where to go next

- [Installation](/products/ought/docs/installation) — get the CLI on your machine
- [Quick start](/products/ought/docs/quickstart) — write your first spec and run it
- [Writing specs](/products/ought/docs/writing-specs) — the spec file format in detail
