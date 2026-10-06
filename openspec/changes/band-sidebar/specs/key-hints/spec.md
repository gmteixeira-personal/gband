## REMOVED Requirements

### Requirement: Hints segment plugin
**Reason**: The hints segment is removed with the status line.
**Migration**: Open the key list with `prefix ?` to read the bindings of navigation mode.

### Requirement: Hints of the active table
**Reason**: The hints segment is removed with the status line.
**Migration**: The key list lists the bindings of the `prefix` table.

### Requirement: Key form
**Reason**: The key form is used by the key list, the key style chooser and `gband.keyform`, not by hints alone.
**Migration**: Moved to the key-list capability's "Key form".

### Requirement: Hint labels
**Reason**: The short labels and the `labels` option existed only for the hints segment.
**Migration**: The key list shows each binding's description. Give a binding `desc` to change its line.

### Requirement: Fit to the available width
**Reason**: The hints segment is removed with the status line.
**Migration**: None.

### Requirement: Hint groups
**Reason**: `KeyHintKey` and `KeyHintLabel` are removed with the hints segment.
**Migration**: Style the key list with `KeyListKey` and `KeyListMuted`.

### Requirement: Default setup
**Reason**: The default configuration no longer sets up the hints segment.
**Migration**: None.

### Requirement: Mouse bindings are not hinted
**Reason**: The hints segment is removed with the status line.
**Migration**: The key list leaves out mouse bindings, as the key-list capability's "Mouse bindings in the key list" defines.
