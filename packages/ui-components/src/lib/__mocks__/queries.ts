import {
  QueryClient,
  createInfiniteQuery,
  createQuery,
} from "@tanstack/svelte-query";

// A helper function to create a resolvable mock query.
// This gives us more control over when each query resolves.
export const createResolvableMockQuery = <T>(
  queryFn: (() => T) | ((pageParam: number) => T),
) => {
  const resolveQueue: Array<() => void> = [];
  let currentPromise: Promise<void>;

  const createNewPromise = () => {
    currentPromise = new Promise<void>((res) => {
      resolveQueue.push(res);
    });
  };

  createNewPromise(); // Initialize the first promise

  const resolvableQuery = async (pageParam?: number) => {
    const mockData = queryFn(pageParam as number);
    await currentPromise;
    createNewPromise(); // Create a new promise for the next call
    return mockData;
  };

  const resolve = () => {
    const resolver = resolveQueue.shift();
    if (resolver) {
      resolver();
    }
  };

  return { queryFn: resolvableQuery, resolve };
};

// A helper function to create an infinite Tanstack query that resolves when you call
// the `resolve` function.
export const createResolvableInfiniteQuery = (
  _queryFn: (pageParam: number) => unknown,
  getNextPageParam: (
    _lastPage: unknown,
    _allPages: unknown[],
    lastPageParam: number,
  ) => number | undefined = (
    _lastPage: unknown,
    _allPages: unknown[],
    lastPageParam: number,
  ) => lastPageParam + 1,
) => {
  const { queryFn, resolve } = createResolvableMockQuery(_queryFn);
  const refetch = vi.fn();
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return queryFn(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam,
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  return { query, resolve, refetch };
};

// A helper function to create a regular Tanstack query that resolves when you call resolve
export const createResolvableQuery = <T>(_queryFn: () => T) => {
  const { queryFn, resolve } = createResolvableMockQuery<T>(_queryFn);

  const query = createQuery(
    {
      queryKey: [],
      queryFn: () => {
        return queryFn();
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  return { query, resolve };
};
