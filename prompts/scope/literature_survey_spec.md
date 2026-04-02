# Literature Survey Spec — `{{lisa_root}}/spiral/pass-0/literature-survey.md`

Survey of candidate methods, organized by topic/phenomenon. This template is used both by
the Literature Survey subagent (to produce the initial draft) and by the scoping agent
(to review and finalize the artifact).

## Template

```markdown
# Literature Survey

## Methods Surveyed

### [Topic/Phenomenon A]

#### [Method 1 Name]
- **Source:** [Author(s), Year, Title, DOI/URL]
- **Approach:** [Brief description]
- **Fidelity:** [Low / Medium / High]
- **Assumptions:** [Key assumptions]
- **Valid range:** [Where it applies]
- **Pros:** [Advantages for our problem]
- **Cons:** [Disadvantages or limitations]
- **Available:** [YES / NEEDS_PAPER — whether full paper is accessible]

[Repeat for each candidate method]

#### Recommended Approach for [Topic A]
[Which method(s) to use and why]

### [Topic/Phenomenon B]
[Same structure]

## Cross-Cutting Methods
[Any methods that span multiple topics]

## Papers Retrieved
[List papers saved to {{lisa_root}}/references/retrieved/ with full citations]

## Papers Needed
[Papers flagged with [NEEDS_PAPER] that the human should provide]
```

## Quality Criteria

- Every method candidate cites a peer-reviewed source (author(s), year, title, DOI/URL)
- Alternatives are documented for each phenomenon
- Papers saved to `{{lisa_root}}/references/retrieved/` have proper citations and key equations
