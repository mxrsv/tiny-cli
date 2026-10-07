import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { desktop, ipc } from "../lib/ipc";
import { recoverable } from "../lib/format";
import { useUi } from "../stores/ui-store";
import type { ErrorPayload, Progress, SmartScan } from "../types/ipc";
export function useLatestScan() {
  return useQuery({ queryKey: ["scan", "result"], queryFn: ipc.latestScan });
}
export function useMonitor() {
  return useQuery({
    queryKey: ["system", "monitor"],
    queryFn: ipc.monitor,
    refetchInterval: 10000,
  });
}
export function usePermissions() {
  return useQuery({
    queryKey: ["system", "permissions"],
    queryFn: ipc.permissions,
  });
}
export function useSmartScan() {
  const client = useQueryClient();
  return useMutation({
    mutationKey: ["scan", "run"],
    mutationFn: ipc.smartScan,
    onMutate: () =>
      useUi.getState().setProgress({
        operation: "scan",
        completed: 0,
        total: null,
        message: "Reading your Mac…",
      }),
    onSuccess: (scan) => {
      client.setQueryData(["scan", "result"], scan);
      selectSafe(scan);
      void client.invalidateQueries({ queryKey: ["system", "monitor"] });
    },
    onSettled: () => useUi.getState().setProgress(null),
  });
}
export function selectSafe(scan: SmartScan) {
  useUi
    .getState()
    .selectPaths(
      scan.discovery.groups
        .filter((g) => g.risk === "safe" && recoverable(g.id))
        .flatMap((g) => g.items.map((i) => i.path)),
    );
}
export function useEngineEvents() {
  const client = useQueryClient();
  useEffect(() => {
    if (!desktop) return;
    let closed = false;
    const cleanups: (() => void)[] = [];
    const subscriptions = [
      ...["scan:error", "clean:error", "space-lens:error"].map((event) =>
        listen<ErrorPayload>(event, ({ payload }) => {
          if (payload.code !== "operation_busy")
            useUi.getState().setProgress(null);
          useUi.getState().setEngineError(payload.message);
        }),
      ),
      ...["scan:progress", "clean:progress", "space-lens:progress"].map(
        (event) =>
          listen<Progress>(event, ({ payload }) => {
            useUi.getState().setEngineError(null);
            useUi.getState().setProgress(payload);
          }),
      ),
      listen<SmartScan>("scan:done", ({ payload }) => {
        client.setQueryData(["scan", "result"], payload);
        selectSafe(payload);
        useUi.getState().setProgress(null);
        void client.invalidateQueries({ queryKey: ["system", "monitor"] });
      }),
      listen("clean:done", () => {
        client.setQueryData(["scan", "result"], null);
        useUi.getState().selectPaths([]);
        useUi.getState().setProgress(null);
        void client.invalidateQueries({ queryKey: ["safety"] });
        void client.invalidateQueries({ queryKey: ["system", "monitor"] });
      }),
      listen("space-lens:done", () => useUi.getState().setProgress(null)),
    ];
    for (const subscription of subscriptions)
      void subscription
        .then((unlisten) => {
          if (closed) unlisten();
          else cleanups.push(unlisten);
        })
        .catch(() => {
          useUi.getState().setProgress(null);
        });
    return () => {
      closed = true;
      cleanups.forEach((unlisten) => unlisten());
    };
  }, [client]);
}
