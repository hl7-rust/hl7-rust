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
description: Work with Health Level Seven (HL7®) v2 messages using the
  hl7-rust workspace — parse, query, validate, edit, and render ER7
  (pipe-delimited); convert between ER7, v2.xml, and JSON; move messages
  over MLLP or SOAP; build a vendor dictionary from XSDs. Use when the task
  involves an HL7 v2 message, an ER7/pipe-delimited clinical message, an
  MSH/PID/OBX segment, an ADT/ORU/ORM message, or converting one of those
  to or from XML/JSON.
---`;

  const recipes = `# Look at an unfamiliar message, with its paths
hl7-v2 --paths samples/vendor.hl7

# Pull one field out of a batch of messages
cat inbox/*.hl7 | hl7-v2 --query PID-3

# Validate, with an exit status a script can branch on
hl7-v2 --check samples/adt_a01.hl7

# Convert to typed JSON, or the official v2.xml
hl7-2-from-er7-into-json samples/orm_o01.hl7
hl7-2-from-er7-into-xml  samples/orm_o01.hl7`;
</script>

<DocPage
  title="Agent skill"
  lede="hl7-skill/ is an Agent Skill for Claude Code and any other tool that reads a SKILL.md: which crate or CLI to reach for, copy-paste recipes, and the gotchas worth knowing before relying on the output."
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
    task — an HL7 v2 message, an ER7 fragment, an MSH/PID/OBX segment, a
    conversion to or from XML or JSON — is one this skill applies to. There
    is nothing to install and nothing to configure.
  </p>

  <h2 id="contents">What is inside it</h2>
  <p>The skill is a map plus recipes, not a restatement of the specs:</p>
  <ul>
    <li>
      A one-table decision guide — which crate or command-line tool a given
      task calls for, from “look at an unfamiliar message” to “teach the
      parser a vendor's dialect”.
    </li>
    <li>Install lines for every binary and the library crates.</li>
    <li>
      Copy-paste recipes for the tasks that come up constantly: querying a
      path, checking validity, editing a field, converting in either
      direction, building a vendor dictionary from XSDs.
    </li>
    <li>
      A short list of things worth knowing before trusting the output — no
      field is ever a JSON number, an explicit HL7 null and an absent field
      are different, nothing here validates a full conformance profile —
      with pointers to <a href="/docs/patient-data/">Patient data</a> and
      <a href="/docs/conformance/">Conformance</a> for the parts that need
      more than one sentence.
    </li>
  </ul>
  <CodeSample language="sh" caption="A sample of what the skill's recipes look like" code={recipes} />

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
    crate at length; the skill exists because an agent mid-task needs a
    decision (“which of these fourteen crates do I reach for right now?”)
    answered in a few lines, not a tour. It is deliberately short and links
    out — to each crate's <code>README.md</code>, to its normative
    <code>spec/index.md</code>, and to this site — rather than duplicating
    them.
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
