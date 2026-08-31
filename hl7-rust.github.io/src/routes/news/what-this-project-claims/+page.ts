import type { PageLoad } from './$types';
import { postBySlug } from '$lib/data/news';

export const load: PageLoad = () => ({ title: postBySlug('what-this-project-claims').title });
