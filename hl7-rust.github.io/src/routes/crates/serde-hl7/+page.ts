import type { PageLoad } from './$types';
import { crateBySlug } from '#lib/data/crates.js';

export const load: PageLoad = () => ({ title: crateBySlug('serde-hl7').name });
