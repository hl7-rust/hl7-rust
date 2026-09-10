import type { PageLoad } from './$types';
import { crateBySlug } from '$lib/data/crates';

export const load: PageLoad = () => ({ title: crateBySlug('serde-hl7-v3').name });
