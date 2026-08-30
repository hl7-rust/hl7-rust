<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'contents', label: 'What is inside it' },
    { id: 'using-it', label: 'Using it' },
    { id: 'why-a-skill', label: 'Why a skill instead of just the README' }
  ];

  const frontmatter = `---
name: hl7-skill
description: Explain Health Level Seven (HL7®) concepts, terminology,
  and ideas — what a segment, field, component, and repetition are; what
  MSH-9, ER7, the HL7 null, v2.xml, and Z-segments mean; how HL7 v2
  differs from v3 and FHIR® — with worked examples from the hl7-rust
  workspace. Use whenever the task is understanding an HL7 message or
  vocabulary, not only when this repo's code is involved.
---`;

  const example = `PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E
 ^  ^  ^                                       ^
 |  |  |                                       PID-5, an XPN (person name)
 |  |  PID-3, a CX (identifier), whose 4th component
 |  |  is itself an HD with subcomponents split by &
 |  PID-2, not sent
 PID-1, the set ID`;
</script>

<DocPage
  title="Agent skill"
  lede="hl7-skill/ is an Agent Skill for Claude Code and any other tool that reads a SKILL.md: HL7 concepts, terminology, and ideas, with worked examples from this repo — for anyone trying to understand a message, not only for people writing code against this workspace."
  {contents}
>
  <h2 id="what">What it is</h2>
  <p>
    <a href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-skill"><code>hl7-skill/</code></a>
    is a top-level directory in the workspace holding one file,
    <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-skill/SKILL.md"><code>SKILL.md</code></a>,
    in the
    <a href="https://code.claude.com/docs/en/skills">Agent Skills</a>
    format: YAML frontmatter naming the skill and describing when to use
    it, followed by the instructions themselves.
  </p>
  <CodeSample language="yaml" caption="hl7-skill/SKILL.md — frontmatter" code={frontmatter} />
  <p>
    An agent that has this repository checked out, or that a user points at
    it, reads the description and decides for itself whether the current
    task — an unfamiliar HL7 message, a term like <code>MSH-9</code> or
    “ER7”, a question about how v2 differs from v3 — is one this skill
    applies to. There is nothing to install and nothing to configure.
  </p>

  <h2 id="contents">What is inside it</h2>
  <p>
    The skill is a condensed version of this site's
    <a href="/docs/concepts/">Concepts</a> page — a glossary with worked
    examples, not a restatement of any crate's spec:
  </p>
  <ul>
    <li>
      HL7 as a family of standards (v2, v3, the HL7® FHIR® standard) and
      how they differ.
    </li>
    <li>
      ER7's delimiters, and how a message decomposes into segments,
      fields, repetitions, components, and subcomponents.
    </li>
    <li>Paths versus names, the dictionary, and message structures.</li>
    <li>
      The HL7 null versus an absent field, escape sequences, the v2.xml
      and JSON mappings, and why Z-segments degrade rather than reject.
    </li>
    <li>
      A short “try it” section pointing at this workspace's CLI, for
      readers who want to test an idea against a real message.
    </li>
  </ul>
  <CodeSample language="text" caption="One of the worked examples inside it" code={example} label="How a PID segment decomposes" />

  <h2 id="using-it">Using it</h2>
  <p>
    With Claude Code, nothing to do beyond having the repository open —
    skills in a checked-out project are picked up automatically. Other
    agent tools that support the same <code>SKILL.md</code> convention can
    point directly at the file; everything else can still read it as plain
    Markdown, the same as any other page linked from this site.
  </p>

  <h2 id="why-a-skill">Why a skill instead of just the README</h2>
  <p>
    The root <a href="/">README</a> and this site already describe every
    crate at length; the skill exists so an agent that only needs the
    *vocabulary* — what a component is, what the HL7 null means — gets a
    short, self-contained answer instead of a tour of fourteen crates. It
    is deliberately general: the concepts hold for any HL7 v2 system, and
    this workspace supplies the worked examples.
  </p>
  <Callout heading="The specs stay the source of truth">
    <p>
      Where the skill and a crate's <code>spec/index.md</code> disagree,
      the spec wins — the same rule that governs every other document in
      this workspace. See
      <a href="/docs/#sources">Where the authority lives</a>.
    </p>
  </Callout>
</DocPage>
