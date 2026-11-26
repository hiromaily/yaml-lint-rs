---
name: review-agent-work
description: Review completed or interrupted AI work in this repository when asked for a retrospective, mistakes, incorrect assumptions, or ways to improve the agent's workflow.
---

# Review agent work

Use the user's requests and corrections, observable assistant decisions, tool results, and repository or PR state to reconstruct the work. Review the requested scope, including follow-up and cleanup when relevant. Use the available conversation and read-only checks to fill gaps. State what the evidence leaves unresolved. Describe observable decisions and stated assumptions without reconstructing private reasoning.

1. Identify the outcome the user originally wanted, what they later clarified, and what was actually delivered at each handoff. Judge each decision using the information and authorization available at that point.
2. Trace material findings from the observed decision or omission to its impact. Distinguish a mistaken inference, execution or verification gap, and communication gap where the distinction changes the remedy. Separate confirmed issues from plausible risks and useful recoveries. Avoid treating every failed command as a mistake when recovery was timely and the deliverable was unaffected.
3. For Git handoffs, compare the PR and merge state with the local branch, tracked and untracked changes, and any relevant stash or worktree. For each remaining change, determine its purpose, how far it progressed, and where it ended up. A clean status after stashing only verifies the working tree. When judging a decision to set work aside, check whether there was already authorization to complete it and whether a real blocker existed.
4. Lead with the highest-impact finding. Tie each finding to a specific request, observed action, command result, or artifact. Explain the consequence and how a future agent can verify that its change in behavior worked. When recommending a repository-level change, identify the file or workflow where it belongs. Name any work that still needs a decision or verification.

Present findings in the user's language. Keep the retrospective focused on actions and outcomes. A review request authorizes inspection and reporting. Apply fixes or publish the review when the user requests those actions.
