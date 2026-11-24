# License

```
SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-SA-4.0
```

That single line is the machine-readable answer, and it is what every
`Cargo.toml` in this workspace carries in its `license` field, byte for
byte. An `OR` expression means the choice is yours: pick one, comply with
that one.

This project is multi-licensed. You may use it under **any one** of the
following licenses, at your option. The full text of every option is
committed in [`LICENSES/`](LICENSES/), one directory per license, named by the
lowercase SPDX identifier, each holding the full text in `index.md`:

| License | SPDX identifier | Full text |
| ------- | --------------- | --------- |
| MIT License | `MIT` | [`LICENSES/mit/index.md`](LICENSES/mit/index.md) · <https://opensource.org/license/mit/> |
| Apache License 2.0 | `Apache-2.0` | [`LICENSES/apache-2.0/index.md`](LICENSES/apache-2.0/index.md) · <https://www.apache.org/licenses/LICENSE-2.0> |
| BSD 3-Clause License | `BSD-3-Clause` | [`LICENSES/bsd-3-clause/index.md`](LICENSES/bsd-3-clause/index.md) · <https://opensource.org/license/bsd-3-clause/> |
| GNU General Public License v2.0 only | `GPL-2.0-only` | [`LICENSES/gpl-2.0-only/index.md`](LICENSES/gpl-2.0-only/index.md) · <https://www.gnu.org/licenses/old-licenses/gpl-2.0.html> |
| GNU General Public License v3.0 only | `GPL-3.0-only` | [`LICENSES/gpl-3.0-only/index.md`](LICENSES/gpl-3.0-only/index.md) · <https://www.gnu.org/licenses/gpl-3.0.html> |
| UK Open Government Licence v3.0 | `OGL-UK-3.0` | [`LICENSES/ogl-uk-3.0/index.md`](LICENSES/ogl-uk-3.0/index.md) · <https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/> |
| Creative Commons Attribution Share Alike 4.0 International | `CC-BY-SA-4.0` | [`LICENSES/cc-by-sa-4.0/index.md`](LICENSES/cc-by-sa-4.0/index.md) · <https://creativecommons.org/licenses/by-sa/4.0/> |

Each identifier above is the SPDX short identifier from the
[SPDX License List](https://spdx.org/licenses/), which is the vocabulary
`cargo`, `cargo-deny`, FOSSA, Black Duck, and every other license scanner
reads. So a scanner run against these crates resolves the expression
without a human having to interpret this file.

Pick the one that fits your project and comply with that one. You do not
need to comply with all seven, and you do not need to tell anyone which you
chose.

## Marking a file

Individual source files here do not carry per-file SPDX headers; the
`Cargo.toml` `license` field is the authority for every file in its
package. If your organisation requires per-file marking on vendored code,
the line to add at the top of a file is:

```
// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-SA-4.0
```

or, if you have chosen one of the seven for your own distribution, that one
alone.

## What this covers, and what it does not

This license covers the source code and documentation in this workspace.
It does not cover the HL7® standards themselves: HL7 v2, HL7 v3, and their
XML schemas are published by Health Level Seven International under their
own terms, and this project implements the standards rather than
redistributing them. The same boundary would apply to the
HL7® FHIR® standard, which this project does not implement.

HL7® and FHIR® are registered trademarks of HL7. We are requesting
permission to use it here. Use of the trademarks does not constitute
endorsement of this library by HL7. A license to use this software is not
a license to use either mark.

Copyright © Joel Parker Henderson <joel@joelparkerhenderson.com>

## Why seven

Healthcare integration code ends up inside organisations with very
different legal constraints: a permissive license suits a vendor
integrating into a proprietary product, while a copyleft license suits a
public-sector project that wants derivatives kept open. Offering the choice
means neither has to ask.

---

HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their use of these trademarks does not constitute an endorsement by HL7.
