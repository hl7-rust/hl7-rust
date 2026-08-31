import type { PageLoad } from './$types';
import { crateBySlug } from '$lib/data/crates';

export const load: PageLoad = () => ({ title: crateBySlug('hl7-3-derive').name });
