import type { PageLoad } from './$types';

// The home page's title is the full string <title> wants — see
// $lib/data/site-title.ts — because siteTitle() leaves path "/" alone
// rather than appending the site-name suffix every other page gets.
export const load: PageLoad = () => ({ title: 'HL7 Rust — HL7 v2 and v3 for Rust' });
