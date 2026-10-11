---
title: The shell's contract is a shared npm package
status: superseded
superseded-by: decisions/ahpw-owns-its-embedding-contract.md
date: 2026-10-11
refs:
  - file:///github/ahp-desk/.project/decisions/a-ui-takes-its-hosts-from-a-command-and-an-event.md - the contract that the package wraps
---

## Context

ahpw and ahpapp both talk to the shell.
They read the hosts, the window's host, and they drive the window controls.
The contract can be one package that both UIs import, or a document that each UI implements.

## Decision

The TypeScript side of the contract is one npm package, `desk-client`, published from `packages/desk-client/` in ahp-desk.
ahpw and ahpapp depend on it.
Source: Softov, asked "Where does the TypeScript side of the shell contract live, for ahpw and ahpapp to use?": "Shared npm package (Recommended)".

## Consequences

The contract has one definition, with its types.
A change to the contract is a new package version, and both UIs move to it.
ahp-desk publishes an npm package beside its installers.

## Options

- **A spec, and each UI owns its adapter.** Lost: two copies of the same types, which can drift apart.
