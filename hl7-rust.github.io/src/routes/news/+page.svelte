<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import { POSTS, formatDate } from '$lib/data/news';
</script>

<DocPage
  title="News"
  lede="Releases, decisions, and write-ups. Infrequent by design — a post here means something changed that a user of these crates would want to know."
>
  <ul class="posts">
    {#each POSTS as post (post.slug)}
      <li class="post">
        <p class="post-date">{formatDate(post.date)}</p>
        <h2><a href={`/news/${post.slug}/`}>{post.title}</a></h2>
        <p>{post.summary}</p>
      </li>
    {/each}
  </ul>
</DocPage>

<style>
  .posts {
    list-style: none;
    margin: 2rem 0 0;
    padding: 0;
    display: grid;
    gap: 2.5rem;
  }

  .post-date {
    margin: 0 0 0.25rem;
    font-size: 0.8125rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    opacity: 0.7;
  }

  .post h2 {
    margin: 0 0 0.5rem;
  }

  .post p:last-child {
    margin-bottom: 0;
  }
</style>
