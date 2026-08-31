import type { PageLoad } from './$types';
import { crateBySlug } from '$lib/data/crates';

export const load: PageLoad = () => ({ title: crateBySlug('hl7-2-from-er7-into-json').name });
