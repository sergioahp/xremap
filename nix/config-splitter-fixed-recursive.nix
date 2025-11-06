{ lib }:

# Xremap Config Splitter for Nix (FIXED - Handles Nested Remaps)
#
# This version correctly handles nested remaps by recursively removing
# descriptions at all levels

with lib;

rec {
  # Extract description from an action attribute set
  # FIXED: Now recursively processes nested remaps
  extractDescription = action:
    if isAttrs action && action ? description then
      let
        clean = removeAttrs action [ "description" ];
        # If only 'action' remains after removing description, unwrap it
        unwrapped = if (length (attrNames clean)) == 1 && clean ? action
                    then clean.action
                    else clean;
        # FIXED: Recursively process nested remaps
        finalAction = if isAttrs unwrapped && unwrapped ? remap
                      then unwrapped // { remap = processRemapRecursive unwrapped.remap; }
                      else unwrapped;
      in {
        hasDescription = true;
        description = action.description;
        cleanAction = finalAction;
      }
    else if isAttrs action && action ? remap then
      # No description but has nested remap - still need to process it
      {
        hasDescription = false;
        description = null;
        cleanAction = action // { remap = processRemapRecursive action.remap; };
      }
    else
      {
        hasDescription = false;
        description = null;
        cleanAction = action;
      };

  # NEW: Recursively process a remap attribute set, removing descriptions at all levels
  processRemapRecursive = remap:
    mapAttrs (key: value:
      let extracted = extractDescription value;
      in extracted.cleanAction
    ) remap;

  # Process a single remap entry (key -> action)
  processRemapEntry = key: action: let
    extracted = extractDescription action;
  in {
    binding = key;
    cleanAction = extracted.cleanAction;
    description = extracted.description;
  };

  # Process a remap attribute set
  processRemap = remap: let
    entries = mapAttrsToList processRemapEntry remap;
    cleanRemap = listToAttrs (map (e: nameValuePair e.binding e.cleanAction) entries);
    bindings = map (e: {
      inherit (e) binding description;
      action = e.cleanAction;
    }) entries;
  in {
    inherit cleanRemap bindings;
  };

  # Process a keymap or modmap item
  processKeymapItem = item: let
    remapResult = if item ? remap then processRemap item.remap else { cleanRemap = {}; bindings = []; };

    # Extract metadata
    name = item.name or null;
    mode = item.mode or null;
    application = item.application or null;
    window = item.window or null;
    device = item.device or null;
    exactMatch = item.exact_match or null;

    # Add metadata to each binding
    enrichedBindings = map (binding:
      binding // {
        inherit name mode application window device;
        exact_match = exactMatch;
      } // (
        # Remove null values
        filterAttrs (n: v: v != null) {
          inherit name mode application window device;
          exact_match = exactMatch;
        }
      )
    ) remapResult.bindings;

    # Create clean item without descriptions
    cleanItem = (removeAttrs item [ "remap" ]) // {
      remap = remapResult.cleanRemap;
    };
  in {
    inherit cleanItem;
    bindings = enrichedBindings;
  };

  # Process a list of keymap/modmap items
  processItems = items: let
    results = map processKeymapItem items;
    cleanItems = map (r: r.cleanItem) results;
    allBindings = concatMap (r: r.bindings) results;
  in {
    inherit cleanItems allBindings;
  };

  # Main function: Split config into clean config and dmenu bindings
  splitConfig = config: let
    # Process modmap
    modmapResult = if config ? modmap then
      processItems config.modmap
    else
      { cleanItems = []; allBindings = []; };

    # Process keymap
    keymapResult = if config ? keymap then
      processItems config.keymap
    else
      { cleanItems = []; allBindings = []; };

    # Combine all bindings
    allBindings = modmapResult.allBindings ++ keymapResult.allBindings;

    # Create clean config
    cleanConfig = (removeAttrs config [ "modmap" "keymap" ]) //
      optionalAttrs (config ? modmap) { modmap = modmapResult.cleanItems; } //
      optionalAttrs (config ? keymap) { keymap = keymapResult.cleanItems; };

    # Create dmenu bindings
    dmenuBindings = {
      bindings = allBindings;
      count = length allBindings;
    };
  in {
    inherit cleanConfig dmenuBindings;
  };

  # Utility: Filter bindings by mode
  filterByMode = mode: bindings:
    filter (b: b.mode or "default" == mode) bindings.bindings;

  # Utility: Filter bindings by application
  filterByApplication = app: bindings:
    filter (b:
      if b ? application then
        if b.application ? only then
          elem app (toList b.application.only)
        else if b.application ? not then
          !(elem app (toList b.application.not))
        else
          true
      else
        true
    ) bindings.bindings;

  # Utility: Group bindings by name (category)
  groupByName = bindings:
    groupBy (b: b.name or "General") bindings.bindings;
}
