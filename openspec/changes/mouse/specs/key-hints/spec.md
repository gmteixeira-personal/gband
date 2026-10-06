## ADDED Requirements

### Requirement: Mouse bindings are not hinted
The hints segment SHALL give no hint to a binding whose key is a mouse name, as the configuration capability defines mouse names.

#### Scenario: Navigation mode hints
- **WHEN** the default configuration is in use and navigation mode is active
- **THEN** no hint shows `leftmouse`, `rightmouse`, `middlemouse` or a wheel name
