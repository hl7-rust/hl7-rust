import type { PageLoad } from './$types';
import { postBySlug } from '$lib/data/news';

export const load: PageLoad = () => ({ title: postBySlug('serde-support-for-hl7').title });
