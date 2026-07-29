import init, {
  ClipForgeProject,
} from "../generated/clipforge-wasm/clipforge_web_bindings";

let initialization: Promise<unknown> | undefined;

export function initializeBindings(): Promise<unknown> {
  initialization ??= init();
  return initialization;
}

export { ClipForgeProject };
