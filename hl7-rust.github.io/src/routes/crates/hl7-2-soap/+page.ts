import type { PageLoad } from './$types';
import { crateBySlug } from '#lib/data/crates.js';

export const load: PageLoad = () => ({ title: crateBySlug('hl7-2-soap').name });
