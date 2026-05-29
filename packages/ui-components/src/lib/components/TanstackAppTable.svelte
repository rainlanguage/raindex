<script lang="ts" generics="DataItem, InputData = DataItem[]">
  import { invalidateTanstackQueries } from "$lib/queries/queryClient";
  import Refresh from "./icon/Refresh.svelte";
  import type {
    CreateInfiniteQueryResult,
    InfiniteData,
  } from "@tanstack/svelte-query";
  import {
    Button,
    Table,
    TableBody,
    TableBodyRow,
    TableHead,
  } from "flowbite-svelte";
  import {
    afterUpdate,
    createEventDispatcher,
    onDestroy,
    onMount,
  } from "svelte";
  import { useQueryClient } from "@tanstack/svelte-query";
  import {
    createWindowVirtualizer,
    type SvelteVirtualizer,
  } from "@tanstack/svelte-virtual";
  import type { Readable } from "svelte/store";
  import type { VirtualItem } from "@tanstack/virtual-core";

  const queryClient = useQueryClient();

  const dispatch = createEventDispatcher();

  export let queryKey: string;
  export let query: CreateInfiniteQueryResult<
    InfiniteData<InputData, unknown>,
    Error
  >;
  export let emptyMessage: string = "None found";
  export let rowHoverable = true;
  // Selector to extract DataItem[] from each page of type InputData
  export let dataSelector: (pageData: InputData) => DataItem[] = (pageData) =>
    Array.isArray(pageData) ? (pageData as unknown as DataItem[]) : [];

  type VirtualizationConfig = {
    enabled?: boolean;
    estimatedRowHeight?: number;
    overscan?: number;
  };
  export let virtualization: VirtualizationConfig = {};

  $: enableVirtualization = virtualization.enabled ?? true;
  $: estimatedRowHeight = virtualization.estimatedRowHeight ?? 56;
  $: virtualizationOverscan = virtualization.overscan ?? 8;

  let measuredRowHeight: number | null = null;
  $: rowHeight = measuredRowHeight ?? estimatedRowHeight;

  // Transform the query data by applying dataSelector to each page only when the page
  // reference or selector changes. This keeps the array reference stable so large tables
  // don't re-render on every fetch status update.
  let transformedPages: DataItem[][] = [];
  let lastPagesRef: InputData[] | undefined;
  let lastSelector = dataSelector;
  $: {
    const currentData = $query.data;
    const currentPages = currentData?.pages;
    const selectorChanged = lastSelector !== dataSelector;
    if (!currentPages) {
      transformedPages = [];
      lastPagesRef = undefined;
      lastSelector = dataSelector;
    } else if (currentPages !== lastPagesRef || selectorChanged) {
      transformedPages = currentPages.map((page) => dataSelector(page));
      lastPagesRef = currentPages;
      lastSelector = dataSelector;
    }
  }

  // Helper derived values kept out of the main script block for readability
  $: flattenedRows = transformedPages.flat();
  $: totalRows = flattenedRows.length;
  $: hasData = totalRows > 0;
  $: isInitialLoading =
    totalRows === 0 && ($query.isLoading || $query.isFetching);

  // Virtualization (TanStack Virtual) tied to window scroll so the table keeps its natural height.
  const hasWindow = typeof window !== "undefined";
  let tableContainerElement: HTMLDivElement | null = null;
  let tableOffsetTop = 0;
  let virtualizerStore: Readable<
    SvelteVirtualizer<Window, HTMLElement>
  > | null = null;
  let virtualizer: SvelteVirtualizer<Window, HTMLElement> | null = null;
  let unsubscribeVirtualizer: (() => void) | null = null;
  let virtualizationActive = false;

  function updateTableOffset() {
    if (!enableVirtualization || !hasWindow || !tableContainerElement) {
      tableOffsetTop = 0;
      return;
    }
    const rect = tableContainerElement.getBoundingClientRect();
    tableOffsetTop = rect.top + window.scrollY;
  }

  onMount(() => {
    if (!hasWindow) {
      return;
    }
    virtualizerStore = createWindowVirtualizer<HTMLElement>({
      count: totalRows,
      estimateSize: () => rowHeight,
      overscan: virtualizationOverscan,
      scrollMargin: tableOffsetTop,
      getItemKey: (index) => index,
      enabled: enableVirtualization && totalRows > 0,
    });
    unsubscribeVirtualizer = virtualizerStore.subscribe((instance) => {
      virtualizer = instance;
    });
    updateTableOffset();
    return () => {
      unsubscribeVirtualizer?.();
    };
  });

  afterUpdate(() => {
    if (enableVirtualization) {
      updateTableOffset();
    }
  });

  $: if (virtualizer) {
    virtualizer.setOptions({
      count: totalRows,
      estimateSize: () => rowHeight,
      overscan: virtualizationOverscan,
      scrollMargin: tableOffsetTop,
      getItemKey: (index) => index,
      enabled: enableVirtualization && totalRows > 0,
    });
  }

  $: virtualizationActive = enableVirtualization && Boolean(virtualizer);

  let virtualItems: VirtualItem[] = [];
  let totalSize = 0;
  let topPadding = 0;
  let bottomPadding = 0;
  let scrollMargin = 0;
  $: {
    const hasRows = totalRows > 0;
    scrollMargin = virtualizationActive ? tableOffsetTop : 0;
    if (virtualizationActive && virtualizer && hasRows) {
      virtualItems = virtualizer.getVirtualItems();
      totalSize = virtualizer.getTotalSize();
      const firstItem = virtualItems[0];
      const lastItem = virtualItems[virtualItems.length - 1];
      topPadding = firstItem ? Math.max(0, firstItem.start - scrollMargin) : 0;
      bottomPadding = lastItem
        ? Math.max(0, totalSize - (lastItem.end - scrollMargin))
        : Math.max(0, totalSize);
    } else {
      virtualItems = [];
      totalSize = 0;
      topPadding = 0;
      bottomPadding = 0;
    }
  }

  onDestroy(() => {
    unsubscribeVirtualizer?.();
  });

  afterUpdate(() => {
    if (!virtualizationActive || !virtualizer || !tableContainerElement) {
      return;
    }
    const rows = Array.from(
      tableContainerElement.querySelectorAll(
        'tbody tr[data-virtual-row="true"]',
      ),
    ) as HTMLElement[];
    if (!rows.length) {
      return;
    }
    for (const row of rows) {
      virtualizer.measureElement(row);
    }
    if (measuredRowHeight === null) {
      const sampleHeight = rows[0].getBoundingClientRect().height;
      if (sampleHeight > 0) {
        measuredRowHeight = sampleHeight;
      }
    }
  });
</script>

<div data-testid="title" class="flex h-16 w-full items-center justify-end">
  <slot name="info" />
  <slot name="timeFilter" />
  <slot name="title" />
  <Refresh
    class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
    data-testid="refreshButton"
    spin={$query.isLoading || $query.isFetching}
    on:click={async () => {
      if (queryKey) {
        invalidateTanstackQueries(queryClient, [queryKey]);
      }
    }}
  />
</div>
{#if isInitialLoading}
  <div
    data-testid="tableLoadingSkeleton"
    class="animate-pulse overflow-hidden rounded-lg border dark:border-gray-700"
  >
    <div
      class="grid grid-cols-5 gap-4 border-b bg-gray-50 p-4 dark:border-gray-700 dark:bg-gray-800"
    >
      {#each [0, 1, 2, 3, 4] as columnIndex (columnIndex)}
        <div class="h-3 rounded bg-gray-200 dark:bg-gray-700" />
      {/each}
    </div>
    {#each [0, 1, 2, 3, 4, 5, 6, 7] as rowIndex (rowIndex)}
      <div
        class="grid grid-cols-5 gap-4 border-b p-4 last:border-b-0 dark:border-gray-700"
      >
        <div class="h-4 rounded bg-gray-200 dark:bg-gray-700" />
        <div class="h-4 rounded bg-gray-200 dark:bg-gray-700" />
        <div class="h-4 rounded bg-gray-200 dark:bg-gray-700" />
        <div class="h-4 rounded bg-gray-200 dark:bg-gray-700" />
        <div class="h-4 rounded bg-gray-200 dark:bg-gray-700" />
      </div>
    {/each}
  </div>
{:else if totalRows === 0}
  <div
    data-testid="emptyMessage"
    class="text-center text-gray-900 dark:text-white"
  >
    {emptyMessage}
  </div>
{:else if hasData}
  <div
    class="cursor-pointer overflow-x-auto rounded-lg border dark:border-none"
    data-testid="tanstackTableContainer"
    bind:this={tableContainerElement}
  >
    <Table
      divClass="min-w-full"
      class="w-full table-fixed"
      hoverable={rowHoverable}
    >
      <TableHead data-testid="head">
        <slot name="head" />
      </TableHead>
      <TableBody>
        {#if virtualizationActive && topPadding > 0}
          <tr aria-hidden="true">
            <td
              colspan="1000"
              class="border-0 p-0"
              style={`height:${topPadding}px;`}
            />
          </tr>
        {/if}
        {#if virtualizationActive}
          {#each virtualItems as virtualItem (virtualItem.key)}
            <TableBodyRow
              class="whitespace-nowrap"
              data-testid="bodyRow"
              data-virtual-row="true"
              on:click={() => {
                dispatch("clickRow", {
                  item: flattenedRows[virtualItem.index],
                });
              }}
            >
              <slot name="bodyRow" item={flattenedRows[virtualItem.index]} />
            </TableBodyRow>
          {/each}
        {:else}
          {#each transformedPages as page}
            {#each page as item}
              <TableBodyRow
                class="whitespace-nowrap"
                data-testid="bodyRow"
                on:click={() => {
                  dispatch("clickRow", { item });
                }}
              >
                <slot name="bodyRow" {item} />
              </TableBodyRow>
            {/each}
          {/each}
        {/if}
        {#if virtualizationActive && bottomPadding > 0}
          <tr aria-hidden="true">
            <td
              colspan="1000"
              class="border-0 p-0"
              style={`height:${bottomPadding}px;`}
            />
          </tr>
        {/if}
      </TableBody>
    </Table>
  </div>
  <div class="mt-2 flex justify-center">
    <Button
      data-testid="loadMoreButton"
      size="xs"
      color="dark"
      on:click={() => $query.fetchNextPage()}
      disabled={!$query.hasNextPage || $query.isFetchingNextPage}
    >
      {#if $query.isFetchingNextPage}
        Loading more...
      {:else if $query.hasNextPage}
        Load More
      {:else}Nothing more to load{/if}
    </Button>
  </div>
{/if}
