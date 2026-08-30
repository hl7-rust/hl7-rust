/**
 * The site map, in one place.
 *
 * The header nav, the documentation sidebar, the previous/next footer links,
 * and the sitemap all read this, so a page is added once and appears in all
 * four. Order within a section is reading order — that is what previous/next
 * follows.
 */

import { CRATES } from './crates';
import { POSTS } from './news';

export interface NavLink {
  href: string;
  label: string;
  /** One line shown on index pages and in the sidebar's title attribute. */
  blurb?: string;
}

export interface NavSection {
  title: string;
  href: string;
  blurb: string;
  links: NavLink[];
}

export const SECTIONS: NavSection[] = [
  {
    title: 'Documentation',
    href: '/docs/',
    blurb: 'Install it, run it, and learn the vocabulary the rest of the site assumes.',
    links: [
      {
        href: '/docs/install/',
        label: 'Install',
        blurb: 'Every crate, every feature flag, and the command-line tools.'
      },
      {
        href: '/docs/quickstart/',
        label: 'Quick start',
        blurb: 'A real message parsed, queried, edited, and answered, in about five minutes.'
      },
      {
        href: '/docs/concepts/',
        label: 'Concepts',
        blurb: 'ER7, v2.xml, the dictionary, names versus paths, and the HL7 null.'
      },
      {
        href: '/docs/architecture/',
        label: 'Architecture',
        blurb: 'How fourteen crates fit together, and why the seams fall where they do.'
      },
      {
        href: '/docs/cli/',
        label: 'Command line',
        blurb: 'Every flag of every binary in the workspace.'
      },
      {
        href: '/docs/versions/',
        label: 'Versions and compatibility',
        blurb: 'HL7 releases 2.1–2.9, crate versions, the MSRV policy, and licensing.'
      },
      {
        href: '/docs/conformance/',
        label: 'Conformance',
        blurb: 'What “supports 2.1–2.9” means here, exactly — and everything it does not mean.'
      },
      {
        href: '/docs/comparison/',
        label: 'Compared with the alternatives',
        blurb: 'Interface engines, Java, and the other Rust crates: when each one is the right answer.'
      },
      {
        href: '/docs/benchmarks/',
        label: 'Benchmarks',
        blurb: 'Measured figures, the method behind them, and what they are not evidence of.'
      },
      {
        href: '/docs/patient-data/',
        label: 'Patient data',
        blurb: 'What these crates do with PHI, what they never do, and where a value can escape.'
      },
      {
        href: '/docs/agent-skill/',
        label: 'Agent skill',
        blurb: 'A SKILL.md for Claude Code and other agents: HL7 concepts, terminology, and worked examples.'
      },
      {
        href: '/docs/maintainer-skill/',
        label: 'Maintainer skill',
        blurb: 'A second SKILL.md for changing this workspace itself: the pre-PR checklist and the spec-first rule.'
      }
    ]
  },
  {
    title: 'Guides',
    href: '/guides/',
    blurb: 'One task at a time, with the API calls that do it.',
    links: [
      {
        href: '/guides/parsing/',
        label: 'Parsing',
        blurb: 'The three modes, and which one a given integration wants.'
      },
      {
        href: '/guides/navigating/',
        label: 'Navigating',
        blurb: 'Paths, the tree, repetitions, and reading a value back out.'
      },
      {
        href: '/guides/modifying/',
        label: 'Modifying and building',
        blurb: 'Set, null, append a segment, build an acknowledgement, render ER7.'
      },
      {
        href: '/guides/validating/',
        label: 'Validating',
        blurb: 'Errors versus warnings, strict mode, and what a Z-segment does not break.'
      },
      {
        href: '/guides/dictionaries/',
        label: 'Vendor dictionaries',
        blurb: 'Teach the parser a dialect in JSON — or generate one from a site’s XSDs.'
      },
      {
        href: '/guides/struct-mode/',
        label: 'Struct mode and derive',
        blurb: 'Map paths onto a struct once, and keep an escape hatch for the day it changes.'
      },
      {
        href: '/guides/converting/',
        label: 'Converting formats',
        blurb: 'ER7 to XML or JSON and back, in the library and from the shell.'
      },
      {
        href: '/guides/mllp/',
        label: 'MLLP over TCP',
        blurb: 'Framing, streaming, transports, acknowledgements, and strictness.'
      },
      {
        href: '/guides/soap/',
        label: 'SOAP over HTTP',
        blurb: 'Receiving, sending, faults with the right status, and reading a reply.'
      },
      {
        href: '/guides/hl7-v3/',
        label: 'HL7 v3',
        blurb: 'The RIM backbone, the data types, and the three-level envelope.'
      }
    ]
  },
  {
    title: 'Tutorials',
    href: '/tutorials/',
    blurb: 'Longer, start-to-finish walkthroughs you can follow at a keyboard.',
    links: [
      {
        href: '/tutorials/first-message/',
        label: 'Your first message',
        blurb: 'From `cargo add` to a parsed, queried, validated ORU^R01.'
      },
      {
        href: '/tutorials/vendor-dialect/',
        label: 'Taming a vendor dialect',
        blurb: 'The three stages: look at it, write down what you learned, freeze what is stable.'
      },
      {
        href: '/tutorials/mllp-listener/',
        label: 'An MLLP listener that answers',
        blurb: 'Accept a connection, reassemble frames, and send a correct acknowledgement.'
      },
      {
        href: '/tutorials/round-trip/',
        label: 'A lossless round trip',
        blurb: 'ER7 to XML to ER7, as a shell pipeline and as a test.'
      }
    ]
  },
  {
    title: 'Examples',
    href: '/examples/',
    blurb: 'Short, copyable snippets, grouped by what you are trying to do.',
    links: []
  },
  {
    title: 'Crates',
    href: '/crates/',
    blurb: 'A reference page for each of the fourteen workspace members.',
    links: CRATES.map((crate) => ({
      href: `/crates/${crate.slug}/`,
      label: crate.name,
      blurb: crate.tagline
    }))
  },
  {
    title: 'Help',
    href: '/help/',
    blurb: 'Answers, error messages, and where to ask.',
    links: [
      {
        href: '/help/faq/',
        label: 'FAQ',
        blurb: 'The questions that come up before the first line of code.'
      },
      {
        href: '/help/troubleshooting/',
        label: 'Troubleshooting',
        blurb: 'Symptom, cause, and fix for the failures people actually hit.'
      },
      {
        href: '/help/support/',
        label: 'Support and contributing',
        blurb: 'Where to file, what to include, and how the repositories are laid out.'
      }
    ]
  },
  {
    title: 'News',
    href: '/news/',
    blurb: 'Releases, decisions, and write-ups from the people building this.',
    links: POSTS.map((post) => ({
      href: `/news/${post.slug}/`,
      label: post.title,
      blurb: post.summary
    }))
  },
  {
    title: 'Specs',
    href: '/spec/',
    blurb: 'The normative specification for each crate — the source of truth.',
    links: []
  }
];

/** Header navigation: the top of each section. */
export const PRIMARY_NAV: NavLink[] = SECTIONS.map((section) => ({
  href: section.href,
  label: section.title
}));

/** Every page on the site, in reading order, for previous/next and the sitemap. */
export const ALL_PAGES: NavLink[] = [
  { href: '/', label: 'HL7 Rust' },
  ...SECTIONS.flatMap((section) => [
    { href: section.href, label: section.title, blurb: section.blurb },
    ...section.links
  ])
];

export function sectionFor(pathname: string): NavSection | undefined {
  return SECTIONS.find(
    (section) => pathname === section.href || pathname.startsWith(section.href)
  );
}

export interface Neighbors {
  previous?: NavLink;
  next?: NavLink;
}

export function neighborsFor(pathname: string): Neighbors {
  const index = ALL_PAGES.findIndex((page) => page.href === pathname);
  if (index < 0) return {};
  return { previous: ALL_PAGES[index - 1], next: ALL_PAGES[index + 1] };
}
